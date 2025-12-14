use std::env::current_dir;

use lyceris::{
    http::fetch::fetch,
    install, launch,
    minecraft::{
        config::ConfigBuilder,
        emitter::{Emitter, Event},
        loader::fabric::Fabric
    },
    AuthMethod,
};
use serde_json::Value;

#[tokio::test]
async fn run_latest_fabric_version() {
    const ORIGINAL_VANILLA_VERSION_MANIFEST_URL: &str =
        "https://piston-meta.mojang.com/mc/game/version_manifest.json";

    const ORIGINAL_FABRIC_VERSION_MANIFEST_URL: &str = "
        https://meta.fabricmc.net/v2/versions";

    let version_manifest_json: Value = fetch(ORIGINAL_VANILLA_VERSION_MANIFEST_URL, None).await.unwrap();
    let fabric_version_manifest_json: Value = fetch(ORIGINAL_FABRIC_VERSION_MANIFEST_URL, None).await.unwrap();
    let latest_fabric_version = fabric_version_manifest_json["loader"][0]["version"].clone();
    let latest_version = version_manifest_json["latest"]["release"].clone();

    let installation_path = current_dir().unwrap().join("target").join("game");
    let emitter = Emitter::default();

    println!("Using version {}", &latest_fabric_version.to_string());


    let lyceris_config = ConfigBuilder::new(
        installation_path,
        latest_version.as_str().unwrap().to_string(),
        AuthMethod::Offline {
            username: "lyceris".to_string(),
            uuid: None,
        },
    ).loader(
        Box::new(Fabric(latest_fabric_version.as_str().unwrap().to_string()))
    )
    .build();


    emitter
        .on(
            Event::MultipleDownloadProgress,
            |(_, current, total, _): (String, u64, u64, String)| {
                println!("Downloading {}/{}", current, total);
            },
        )
        .await;

    install(&lyceris_config, Some(&emitter)).await.unwrap();
    launch(&lyceris_config, Some(&emitter)).await.unwrap();
}
