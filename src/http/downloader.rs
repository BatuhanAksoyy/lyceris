use futures::{stream, StreamExt};
use reqwest::{Client, IntoUrl};
use std::{
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{
    fs::{create_dir_all, File},
    io::AsyncWriteExt,
    sync::Mutex,
    time::timeout,
};

use crate::{
    error::Error,
    minecraft::emitter::{Emit, Emitter, Event},
    util::retry::retry,
};

/// Downloads a file from the specified URL and saves it to the given destination.
///
/// This function performs an asynchronous HTTP GET request to the provided URL,
/// streams the response body, and writes the content to a file at the specified
/// destination. It also provides progress updates through a callback function.
///
/// # Parameters
///
/// - `url`: The URL of the file to download. It can be any type that implements
///   the `IntoUrl` trait, such as a string slice or a `String`.
/// - `destination`: A `PathBuf` representing the path where the downloaded file
///   will be saved.
/// - `progression_callback`: A mutable closure that takes two `u64` parameters:
///   the number of bytes downloaded so far and the total size of the file. This
///   callback is called after each chunk of data is written to the file, allowing
///   the caller to track the download progress.
///
/// # Returns
///
/// This function returns a `Result<u64, DownloaderError>`. On success, it returns
/// `Ok(u64)`. If an error occurs during the download process, it returns an
/// `Err` containing a `DownloaderError` that describes the failure.
///
/// # Errors
///
/// The function can fail in several ways, including but not limited to:
/// - Network errors when making the HTTP request.
/// - Non-success HTTP status codes (e.g., 404 Not Found).
/// - Errors when creating or writing to the file.
pub async fn download<P: AsRef<Path>>(
    url: impl IntoUrl,
    destination: P,
    emitter: Option<&Emitter>,
    client: Option<&Client>,
) -> crate::Result<u64> {
    // Send a get request to the given url.
    let default_client = Client::default();
    let client = client.unwrap_or(&default_client);
    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        return Err(Error::Download(response.status().to_string()));
    }

    // Get the total size of the file to use at progression
    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;

    if let Some(parent) = destination.as_ref().parent() {
        if !parent.is_dir() {
            create_dir_all(parent).await?;
        }
    }

    // Create a file to write the downloaded content
    let mut file = File::create(&destination).await?;

    // Stream the response body
    let mut stream = response.bytes_stream();

    let mut last_data_received;

    while let Some(chunk_result) = timeout(Duration::from_secs(10), stream.next()).await? {
        match chunk_result {
            Ok(chunk) => {
                // Reset the timer when data is received
                last_data_received = Instant::now();
                downloaded += chunk.len() as u64;

                // Write chunk to the file
                file.write_all(&chunk).await?;

                // Emit progress event
                emitter
                    .emit(
                        Event::SingleDownloadProgress,
                        (
                            destination.as_ref().to_string_lossy().into_owned(),
                            downloaded,
                            total_size,
                        ),
                    )
                    .await;
            }
            Err(_) => {
                // Timeout occurred (no chunk received in 3 seconds)
                return Err(Error::Download(
                    "Connection dead, no data for 3 seconds.".to_string(),
                ));
            }
        }

        // Check if no data has been received in the last 3 seconds
        if last_data_received.elapsed() > Duration::from_secs(10) {
            return Err(Error::Download(
                "Connection dead, no data for 3 seconds.".to_string(),
            ));
        }
    }

    Ok(total_size)
}

/// Downloads multiple files from the specified URLs and saves them to the given destinations.
///
/// This function takes a vector of tuples, where each tuple contains a URL and a destination path.
/// It downloads all files in parallel and provides progress updates through a callback function.
///
/// # Parameters
///
/// - `downloads`: A vector of tuples containing the URLs and their corresponding destination paths.
/// - `progression_callback`: A mutable closure that takes four `u64` parameters:
///   the number of bytes downloaded so far for the current file, the total bytes downloaded so far,
///   the current file index, and the total number of files. This callback is called after each chunk of data
///   is written to the file, allowing the caller to track the download progress.
///
/// # Returns
///
/// This function returns a `Result<(), HttpError>`. On success, it returns `Ok(())`. If an error occurs
/// during the download process, it returns an `Err` containing a `HttpError` that describes the failure.
pub async fn download_multiple<U, P>(
    downloads: Vec<(U, P)>,
    emitter: Option<&Emitter>,
    client: Option<&Client>,
) -> crate::Result<()>
where
    U: IntoUrl + Send,               // URL type that implements IntoUrl
    P: AsRef<Path> + Send + 'static, // Path type
{
    let total_files = downloads.len();
    let total_downloaded = Arc::new(Mutex::new(0));
    let tasks = downloads.into_iter().map(|(url, destination)| {
        let total_downloaded = Arc::clone(&total_downloaded);
        async move {
            // Retry download logic
            let result = retry(
                || async { download(url.as_str(), destination.as_ref(), emitter, client).await },
                Result::is_ok,
                3,
                Duration::from_secs(5),
            )
            .await;

            // Check if the download was successful
            match result {
                Ok(_) => {
                    // Update the progress counter
                    let mut downloaded = total_downloaded.lock().await;
                    *downloaded += 1;

                    emitter
                        .emit(
                            Event::MultipleDownloadProgress,
                            (
                                destination.as_ref().to_string_lossy().into_owned(),
                                *downloaded as u64,
                                total_files as u64,
                            ),
                        )
                        .await;

                    Ok::<(), Error>(())
                }
                Err(e) => {
                    // Return the error immediately
                    Err(e)
                }
            }
        }
    });

    // Create a stream of tasks with limited concurrency
    let mut stream = stream::iter(tasks).buffered(10); // Limit concurrency here

    // Poll the stream and handle results
    while let Some(result) = stream.next().await {
        result?;
    }

    Ok(())
}
