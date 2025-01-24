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
            config::ConfigBuilder, emitter::Emitter, install::install, launch::launch,
        },
    };

    #[tokio::test]
    async fn launch_game() {
        let current_dir = current_dir().unwrap();
        let config = ConfigBuilder::new(
            current_dir.join("target").join("game"),
            "1.16.5",
            AuthMethod::Offline {
                username: "Miate",
                uuid: None,
            },
        )
        .build(None::<()>);

        let emitter = Emitter::default();

        install(&config, Some(&emitter)).await.unwrap();

        let mut child = launch(&config, Some(&emitter)).await.unwrap();

        child.wait().await.unwrap();
    }
}
