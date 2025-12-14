use std::env::current_dir;

use lyceris::{
    http::fetch::fetch,
    install, launch,
    minecraft::{
        config::ConfigBuilder,
        emitter::{Emitter, Event},
    },
    AuthMethod,
};
use serde_json::Value;

#[tokio::test]
async fn run_latest_vanilla_version() {
    const ORIGINAL_VERSION_MANIFEST_URL: &str =
        "https://piston-meta.mojang.com/mc/game/version_manifest.json";

    let version_manifest_json: Value = fetch(ORIGINAL_VERSION_MANIFEST_URL, None).await.unwrap();
    let latest_version = version_manifest_json["latest"]["release"].clone();

    let installation_path = current_dir().unwrap().join("target").join("game");
    let emitter = Emitter::default();
    let lyceris_config = ConfigBuilder::new(
        installation_path,
        latest_version.as_str().unwrap().to_string(),
        AuthMethod::Offline {
            username: "lyceris".to_string(),
            uuid: None,
        },
    )
    .build();

    println!("Using version {}", &latest_version.as_str().unwrap());

    emitter
        .on(
            Event::MultipleDownloadProgress,
            |(_, current, total, _): (String, u64, u64, String)| {
                println!("Downloading {}/{}", current, total);
            },
        )
        .await;

    println!("Installing game");

    install(&lyceris_config, Some(&emitter)).await.unwrap();
    launch(&lyceris_config, Some(&emitter)).await.unwrap();
}
