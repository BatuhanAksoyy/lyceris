use std::{collections::HashMap, env::current_dir};

use lyceris::{
    http::fetch::fetch,
    install, launch,
    minecraft::{
        config::ConfigBuilder,
        emitter::{Emitter, Event},
        loader::forge::Forge,
    },
    AuthMethod,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[tokio::test]
async fn run_latest_forge_version() {
    const ORIGINAL_VANILLA_VERSION_MANIFEST_URL: &str =
        "https://piston-meta.mojang.com/mc/game/version_manifest.json";

    let version_manifest_json: Value = fetch(ORIGINAL_VANILLA_VERSION_MANIFEST_URL, None)
        .await
        .unwrap();

    let latest_version = version_manifest_json["latest"]["release"].clone();
    let latest_forge_version = get_latest_forge_version().await.unwrap();
    let installation_path = current_dir().unwrap().join("target").join("game");
    let emitter = Emitter::default();

    println!("{}", &latest_forge_version);

    let lyceris_config = ConfigBuilder::new(
        installation_path,
        latest_version.as_str().unwrap().to_string(),
        AuthMethod::Offline {
            username: "lyceris".to_string(),
            uuid: None,
        },
    )
    .loader(Box::new(Forge(latest_forge_version)))
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

fn parse_mc_version(version: &str) -> Vec<u32> {
    version
        .split('.')
        .filter_map(|v| v.parse::<u32>().ok())
        .collect()
}

async fn get_latest_forge_version() -> Result<String, Box<dyn std::error::Error>> {
    const ORIGINAL_FORGE_VERSION_MANIFEST_URL: &str =
        "https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json";

    #[derive(Serialize, Deserialize)]
    struct Promotions {
        promos: HashMap<String, String>,
    }

    let promotions: Promotions = fetch(ORIGINAL_FORGE_VERSION_MANIFEST_URL, None).await?;

    let mut latest = promotions
        .promos
        .into_iter()
        .filter(|(k, _)| k.ends_with("-latest"))
        .map(|(k, v)| {
            let mc_version = k.trim_end_matches("-latest").to_string();
            let parsed = parse_mc_version(&mc_version);
            (mc_version, parsed, v)
        })
        .collect::<Vec<_>>();

    latest.sort_by(|a, b| a.1.cmp(&b.1));

    let (_, _, forge_build) = latest.last().expect("No Forge versions found");

    Ok(forge_build.clone())
}
