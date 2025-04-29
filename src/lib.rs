pub mod auth;
pub mod error;
pub mod http;
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

    use crate::minecraft::{config::ConfigBuilder, emitter::Emitter, loader, manager};

    #[tokio::test]
    async fn test() {
        let first_config = ConfigBuilder::new(
            PathBuf::from("C:\\Users\\Batuhan\\AppData\\Roaming\\.minecraft"),
            "1.16.5".to_string(),
            crate::AuthMethod::Offline {
                username: "Miate".to_string(),
                uuid: None,
            },
        )
        .build();

        let second_config = ConfigBuilder::new(
            PathBuf::from("C:\\Users\\Batuhan\\AppData\\Roaming\\.minecraft"),
            "1.21.1".to_string(),
            crate::AuthMethod::Offline {
                username: "Miate".to_string(),
                uuid: None,
            },
        )
        .loader(loader::forge::Forge("43.2.0".to_string()).into())
        .build();

        let emitter = Emitter::default();
        let emitter2 = Emitter::default();

        emitter
            .on(
                crate::minecraft::emitter::Event::SingleDownloadProgress,
                |(path, current, total): (String, u64, u64)| {
                    println!("Downloading {} - {}/{}", path, current, total);
                },
            )
            .await;

        emitter
            .on(
                crate::minecraft::emitter::Event::MultipleDownloadProgress,
                |(current, total): (u64, u64)| {
                    println!("Downloading {}/{}", current, total);
                },
            )
            .await;

        emitter
            .on(crate::minecraft::emitter::Event::Console, |line: String| {
                println!("Line: {}", line);
            })
            .await;

        emitter
            .on(crate::minecraft::emitter::Event::AlreadyRunning, |_: ()| {
                println!("Already running");
            })
            .await;

        emitter
            .on(crate::minecraft::emitter::Event::Exit, |_: ()| {
                println!("Exit");
            })
            .await;

        emitter2
            .on(
                crate::minecraft::emitter::Event::SingleDownloadProgress,
                |(path, current, total): (String, u64, u64)| {
                    println!("Downloading {} - {}/{}", path, current, total);
                },
            )
            .await;

        emitter2
            .on(
                crate::minecraft::emitter::Event::MultipleDownloadProgress,
                |(current, total): (u64, u64)| {
                    println!("Downloading {}/{}", current, total);
                },
            )
            .await;

        emitter2
            .on(crate::minecraft::emitter::Event::Console, |line: String| {
                println!("Line: {}", line);
            })
            .await;

        emitter2
            .on(crate::minecraft::emitter::Event::AlreadyRunning, |_: ()| {
                println!("Already running");
            })
            .await;

        emitter2
            .on(crate::minecraft::emitter::Event::Exit, |_: ()| {
                println!("Exit");
            })
            .await;

        let mut manager = manager::Manager::default();

        let first_instance_id = manager.create_instance(first_config, Some(&emitter));
        let second_instance_id = manager.create_instance(second_config, Some(&emitter));

        manager.start_instance(&first_instance_id).await.unwrap();
        manager.start_instance(&second_instance_id).await.unwrap();
        println!("finish");

        tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;

        manager.stop_instance(&first_instance_id).await.unwrap();
    }
}
