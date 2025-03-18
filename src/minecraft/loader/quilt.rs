use std::{future::Future, pin::Pin};

use super::Loader;
use crate::{
    error::Error,
    http::fetch::fetch,
    json::version::meta::{
        custom::CustomMeta,
        vanilla::{self, VersionMeta},
    },
    minecraft::{config::Config, emitter::Emitter, parse::parse_lib_path},
};
use serde::{Deserialize, Serialize};

const VERSION_META_ENDPOINT: &str = "https://meta.quiltmc.org/v3/";

/// Represents the Quilt loader metadata.
#[derive(Serialize, Deserialize)]
struct QuiltLoader {
    separator: Separator,
    build: i64,
    maven: String,
    version: String,
}

/// Represents the separator used in Quilt versioning.
#[derive(Serialize, Deserialize)]
enum Separator {
    #[serde(rename = "+build.")]
    Build,
    #[serde(rename = ".")]
    Empty,
}

/// Represents a version of Quilt.
#[derive(Serialize, Deserialize)]
struct Version {
    version: String,
    stable: bool,
}

/// Represents the Quilt loader.
pub struct Quilt(pub String);

impl From<Quilt> for Box<dyn Loader> {
    fn from(value: Quilt) -> Self {
        Box::new(value)
    }
}

impl Loader for Quilt {
    /// Merges the Quilt loader with the provided configuration and version metadata.
    ///
    /// This function fetches the available Quilt loaders and versions, then updates
    /// the provided version metadata with the relevant libraries and arguments for
    /// the specified Quilt version.
    ///
    /// # Parameters
    /// - `config`: The configuration for the Minecraft installation.
    /// - `meta`: The version metadata to be merged.
    /// - `_emitter`: An optional emitter for tracking events.
    ///
    /// # Returns
    /// A future that resolves to the updated `VersionMeta`.
    fn merge<'a>(
        &'a self,
        config: &'a Config<()>,
        mut meta: VersionMeta,
        _emitter: Option<&'a Emitter>,
    ) -> Pin<Box<dyn Future<Output = crate::Result<VersionMeta>> + Send + 'a>> {
        Box::pin(async move {
            // Fetch the available Quilt loaders
            let loaders: Vec<QuiltLoader> = fetch(
                format!("{}versions/loader", VERSION_META_ENDPOINT),
                config.client.as_ref(),
            )
            .await?;
            // Fetch the available Quilt versions
            let versions: Vec<Version> = fetch(
                format!("{}versions/game", VERSION_META_ENDPOINT),
                config.client.as_ref(),
            )
            .await?;

            // Find the loader that matches the current Quilt version
            let loader = loaders
                .into_iter()
                .find(|v| v.version == self.0)
                .ok_or_else(|| Error::UnknownVersion("Quilt Loader".into()))?;
            // Find the Quilt version that matches the metadata
            let fabric = versions
                .into_iter()
                .find(|v| v.version == meta.id)
                .ok_or_else(|| Error::UnknownVersion("Quilt".into()))?;

            // Fetch the custom metadata for the loader
            let version: CustomMeta = fetch(
                format!(
                    "{}versions/loader/{}/{}/profile/json",
                    VERSION_META_ENDPOINT, fabric.version, loader.version
                ),
                config.client.as_ref(),
            )
            .await?;

            // Retain libraries that are not in the fetched version
            meta.libraries.retain(|lib| {
                version
                    .libraries
                    .iter()
                    .all(|v_lib| v_lib.name.split(':').nth(1) != lib.name.split(':').nth(1))
            });

            // Extend the libraries with the new ones from the fetched version
            meta.libraries.extend(
                version
                    .libraries
                    .into_iter()
                    .filter_map(|lib| {
                        let path = parse_lib_path(&lib.name).ok()?;
                        lib.url.map(|url| vanilla::Library {
                            downloads: Some(vanilla::LibraryDownloads {
                                artifact: Some(vanilla::File {
                                    path: Some(path.clone()),
                                    sha1: lib.sha1.unwrap_or_default(),
                                    size: lib.size.unwrap_or_default(),
                                    url: format!("{}/{}", url, path),
                                }),
                                classifiers: None,
                            }),
                            extract: None,
                            name: lib.name.clone(),
                            rules: None,
                            natives: None,
                            skip_args: false,
                        })
                    })
                    .collect::<Vec<_>>(),
            );

            // Update the arguments for the Minecraft launch
            if let Some(ref mut arguments) = meta.arguments {
                if let Some(jvm) = version.arguments.jvm {
                    arguments.jvm.extend(jvm);
                }
                if let Some(game) = version.arguments.game {
                    arguments.game.extend(game);
                }
            }

            // Set the main class for the Quilt version
            meta.main_class = version.main_class;

            Ok(meta)
        })
    }

    /// Returns the version of the Quilt loader.
    ///
    /// # Returns
    /// The version as a string.
    fn get_version(&self) -> String {
        self.0.to_string()
    }
}
