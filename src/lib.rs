pub mod auth;
pub mod error;
pub mod http;
pub mod integration;
pub mod json;
pub mod minecraft;
pub mod util;

// Re-export commonly used items for easier access
pub use auth::AuthMethod;
pub use error::Error;
pub use http::downloader::{download, download_multiple};
pub use json::version::meta::vanilla::{Library, VersionMeta};
pub use minecraft::config::Config;
pub use minecraft::{install::install, launch::launch};
pub use util::json::{read_json, write_json};

/// A type alias for results returned by library functions.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod test {
    use std::{path::PathBuf, thread::park};

    use crate::{
        integration::modrinth::install_modrinth_pack,
        minecraft::{config::ConfigBuilder, emitter::{Emitter, Event}, install::FileType, loader, manager},
    };

    #[tokio::test]
    async fn test() {
        let emitter = Emitter::default();

        // Single download progress event send when
        // a file is being downloaded.
        // emitter
        //     .on(
        //         Event::SingleDownloadProgress,
        //         |(path, current, total): (String, u64, u64)| {
        //             println!("Downloading {} - {}/{}", path, current, total);
        //         },
        //     )
        //     .await;
    
        // Multiple download progress event send when
        // multiple files are being downloaded.
        // Java, libraries and assets are downloaded in parallel and
        // this event is triggered for each file.
        emitter
            .on(
                Event::MultipleDownloadProgress,
                |(_, current, total, _): (String, u64, u64, String)| {
                    println!("Downloading {}/{}", current, total);
                },
            )
            .await;

        install_modrinth_pack(
            PathBuf::from("C:\\Users\\Batuhan\\Desktop\\WORKS\\lyceris\\target\\test-modrinth"),
            "cqaC80tF".to_string(),
            Some(&emitter),
        )
        .await
        .unwrap();
    }
}
