pub mod auth;
pub mod error;
pub mod http;
pub mod json;
pub mod minecraft;
pub mod util;

use crate::error::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use std::env::current_dir;

    use crate::{
        auth::AuthMethod,
        minecraft::{
            config::ConfigBuilder,
            emitter::{Emitter, Event},
            install::install,
            launch::launch,
            loader::neoforge::NeoForge,
        },
    };

    #[tokio::test]
    async fn launch_game() {
        let current_dir = current_dir().unwrap();
        let config = ConfigBuilder::new(
            current_dir.join("target").join("game"),
            "1.21.4".into(),
            AuthMethod::Offline {
                username: "Miate".into(),
                uuid: None,
            },
        )
        .loader(NeoForge("21.4.111-beta".to_string()).into())
        .build();

        let emitter = Emitter::default();

        // Single download progress event send when
        // a file is being downloaded.
        emitter
            .on(
                Event::SingleDownloadProgress,
                |(path, current, total): (String, u64, u64)| {
                    println!("Downloading {} - {}/{}", path, current, total);
                },
            )
            .await;
    
        // Multiple download progress event send when
        // multiple files are being downloaded.
        // Java, libraries and assets are downloaded in parallel and
        // this event is triggered for each file.
        emitter
            .on(
                Event::MultipleDownloadProgress,
                |(current, total): (u64, u64)| {
                    println!("Downloading {}/{}", current, total);
                },
            )
            .await;

            emitter
            .on(
                Event::Console,
                |line: String| {
                    println!("Downloading {}", line);
                },
            )
            .await;

        install(&config, Some(&emitter)).await.unwrap();

        let mut child = launch(&config, Some(&emitter)).await.unwrap();

        child.wait().await.unwrap();
    }
}
