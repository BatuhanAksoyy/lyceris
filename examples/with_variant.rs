use std::env;

use lyceris::minecraft::{
    config::ConfigBuilder,
    install::install,
    launch::launch,
    loader::{fabric::Fabric, forge::Forge, quilt::Quilt, Loader},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = env::current_dir()?;
    let config = ConfigBuilder::new(
        current_dir.join("game"),
        "1.21.4".into(),
        lyceris::auth::AuthMethod::Offline {
            username: "Lyceris".into(),
            // If none given, it will be generated.
            uuid: None,
        },
    )
    // You can use Fabric, Quilt or Forge here.
    .loader(get_loader_by_name("fabric", "0.16.0"))
    .build();

    // Install method also checks for broken files
    // and downloads them again if they are broken.
    install(&config, None).await?;

    // This method never downloads any file and just runs the game.
    launch(&config, None).await?.wait().await?;

    Ok(())
}

// Example implementation to decide
// which loader to use by name and version.
fn get_loader_by_name(name: &str, loader_version: &str) -> Box<dyn Loader> {
    match name {
        "fabric" => Fabric(loader_version.to_string()).into(),
        "forge" => Forge(loader_version.to_string()).into(),
        "quilt" => Quilt(loader_version.to_string()).into(),
        _ => panic!("Loader not found"),
    }
}
