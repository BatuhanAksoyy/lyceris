use reqwest::{Client, IntoUrl, Response};
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;

/// A struct to hold optional fetch request parameters.
#[derive(Default)]
pub struct FetchOptions<B: Serialize> {
    pub method: reqwest::Method,
    pub headers: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
    pub body: Option<B>,
}

pub async fn fetch<T: DeserializeOwned>(
    url: impl IntoUrl,
    client: Option<&Client>,
) -> crate::Result<T> {
    // Call the fetch function with default options
    let default_client = Client::default();
    fetch_with_options::<T, ()>(url, None, client.unwrap_or(&default_client)).await
}

/// Performs a customizable fetch request.
///
/// This function allows you to send HTTP requests with various options, including
/// the HTTP method, headers, query parameters, and body. If no options are provided,
/// it defaults to a GET request that returns JSON.
///
/// # Parameters
///
/// - `url`: A string slice representing the URL to send the request to.
/// - `options`: An optional `FetchOptions` struct that contains parameters for the request.
///   If not provided, the function defaults to a GET request with no additional parameters.
///
/// # Type Parameters
///
/// - `T`: The type to which the response will be deserialized. This type must implement
///   the `DeserializeOwned` trait from the `serde` crate.
/// - `B`: The type of the body to be sent with the request. This type must implement
///   the `Serialize` trait from the `serde` crate.
///
/// # Returns
///
/// Returns a `Result<T, Error>`, where `T` is the type to which the response will be deserialized.
/// On success, it returns `Ok(T)`, where `T` is the deserialized response. On failure,
/// it returns an `Err(Error)` containing the error information.
///
/// # Errors
///
/// The function can fail in several ways, including but not limited to:
/// - Network errors when sending the request.
/// - Non-success HTTP status codes (e.g., 404 Not Found).
/// - Errors during the deserialization of the response body.
pub async fn fetch_with_options<T: DeserializeOwned, B: Serialize + Default>(
    url: impl IntoUrl,
    options: Option<FetchOptions<B>>,
    client: &Client,
) -> crate::Result<T> {
    let options = options.unwrap_or_default(); // Use default options if none provided

    let mut request_builder = client.request(options.method.clone(), url);

    // Add headers if provided
    for (key, value) in options.headers {
        request_builder = request_builder.header(key, value);
    }

    // Add query parameters if provided
    for (key, value) in options.query_params {
        request_builder = request_builder.query(&[(key, value)]);
    }

    // Add body if provided
    if let Some(b) = options.body {
        request_builder = request_builder.json(&b);
    }

    // Send the request and await the response
    let response: Response = request_builder.send().await?;

    // Deserialize the response body
    Ok(response.json::<T>().await?)
}
