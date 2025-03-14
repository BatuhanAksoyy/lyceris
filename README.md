
<div align="center">

<h3 align="center">Lyceris</h3>
<p align="center">
An open source Minecraft launcher library written in Rust.
<br/>

[![Crates.io](https://img.shields.io/crates/v/lyceris?color=fc8d62)](https://crates.io/crates/lyceris)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/BatuhanAksoyy/lyceris#license)
[![Crates.io](https://img.shields.io/crates/d/lyceris.svg)](https://crates.io/crates/lyceris)

</p>
</div>

## About The Project

Lyceris is written with functional programming paradigm to achieve simplicity. It supports Microsoft authentication, loaders like Fabric, Quilt (more will be implemented soon), multi-threaded control system and download parallelism. It also automatically downloads necessary Java version. Library name comes from a character from Sword Art Online anime.

## Supported Mod Loaders
- [X] Forge (Above version 1.12.2)
- [X] NeoForge
- [X] Fabric
- [X] Quilt

Versions below 1.12.2 Forge is not supported and won't be supported in the future.

## Getting Started

```sh
cargo add lyceris
```

## Usage

Don't forget to change the game directory path!
```rust
use std::env;

use lyceris::minecraft::{
    config::ConfigBuilder,
    emitter::{Emitter, Event},
    install::install,
    launch::launch,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Emitter uses `EventEmitter` inside of it
    // and it uses tokio::Mutex for locking.
    // That causes emitter methods to be async.
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

    // Console event send when a line is printed to the console.
    // It uses a seperated tokio thread to handle this operation.
    emitter
        .on(Event::Console, |line: String| {
            println!("Line: {}", line);
        })
        .await;

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
    .build();

    // Install method also checks for broken files
    // and downloads them again if they are broken.
    install(&config, Some(&emitter)).await?;

    // This method never downloads any file and just runs the game.
    launch(&config, Some(&emitter)).await?.wait().await?;

    Ok(())
}


```
## Roadmap
- [ ] Download resumption

See the [open issues](https://github.com/cubidron/lyceris/issues) for a full list of proposed features (and known issues).
## License

Distributed under the MIT License. See [MIT License](https://opensource.org/licenses/MIT) for more information.
