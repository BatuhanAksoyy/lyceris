/// The main library module for the Minecraft management system.
///
/// This library provides functionalities for managing Minecraft installations,
/// including authentication, downloading necessary files, and launching the game.
///
/// # Modules
/// - `auth`: Handles authentication with Microsoft and Xbox Live.
/// - `error`: Defines error types used throughout the library.
/// - `http`: Provides HTTP utilities for making requests.
/// - `json`: Contains utilities for handling JSON data.
/// - `minecraft`: Manages Minecraft-specific functionalities, including installation and launching.
/// - `util`: Contains various utility functions and types.
///
/// # Examples
///
/// You can find examples of how to use this library in the `examples` directory.
/// For instance, the `with_emitter` example demonstrates how to track download progress
/// and launch Minecraft. Below is the code for the `with_emitter` example:
///
/// ```rust
/// use std::env;
///
/// use lyceris::minecraft::{
///     config::ConfigBuilder,
///     emitter::{Emitter, Event},
///     install::install,
///     launch::launch,
/// };
///
/// /// Example of using the Emitter to track download progress and launch Minecraft.
/// ///
/// /// This example demonstrates how to set up an Emitter to listen for download
/// /// progress events and launch the Minecraft game with a specified configuration.
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let emitter = Emitter::default();
///
///     emitter
///         .on(
///             Event::SingleDownloadProgress,
///             |(path, current, total): (String, u64, u64)| {
///                 println!("Downloading {} - {}/{}", path, current, total);
///             },
///         )
///         .await;
///
///     emitter
///         .on(
///             Event::MultipleDownloadProgress,
///             |(current, total): (u64, u64)| {
///                 println!("Downloading {}/{}", current, total);
///             },
///         )
///         .await;
///
///     emitter
///         .on(Event::Console, |line: String| {
///             println!("Line: {}", line);
///         })
///         .await;
///
///     let current_dir = env::current_dir()?; 
///     let config = ConfigBuilder::new(
///         current_dir.join("game"),
///         "1.21.4".into(),
///         lyceris::auth::AuthMethod::Offline {
///             username: "Lyceris".into(),
///             uuid: None,
///         },
///     )
///     .build();
///
///     install(&config, Some(&emitter)).await?;
///     launch(&config, Some(&emitter)).await?.wait().await?;
///
///     Ok(())
/// }
/// ```
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
