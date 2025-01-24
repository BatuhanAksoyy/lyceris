use std::{
    collections::{HashMap, HashSet},
    env::temp_dir,
};

use serde::{Deserialize, Serialize};

use crate::{
    http::downloader::download,
    json::version::meta::{
        custom::{CustomMeta, Library},
        vanilla::{self, VersionMeta},
    },
    minecraft::{config::Config, emitter::Emitter, parse::parse_lib_path},
    util::{
        extract::{extract_specific_directory, extract_specific_file},
        json::read_json,
    },
};

use super::Loader;

const INSTALLER_JAR_ENDPOINT: &str = "https://maven.minecraftforge.net/net/minecraftforge/forge/{loader_version}/forge-{loader_version}-installer.jar";

#[derive(Serialize, Deserialize)]
struct Mirror {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<String>,
    homepage: String,
    url: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Installer {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<HashMap<String, Data>>,
    pub processors: Option<Vec<Processor>>,
    pub libraries: Vec<Library>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mirror_list: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Processor {
    pub classpath: Vec<String>,
    pub args: Vec<String>,
    pub sides: Option<Vec<String>>,
    pub outputs: Option<HashMap<String, String>>,
    pub jar: String,
    #[serde(default)]
    pub success: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Data {
    pub client: String,
    pub server: String,
}

pub struct Forge(pub &'static str);

impl Loader for Forge {
    async fn merge<T: Loader>(
        &self,
        config: &Config<T>,
        mut meta: VersionMeta,
        emitter: Option<&Emitter>,
    ) -> crate::Result<VersionMeta> {
        let version_name = config.get_version_name();
        let profiles_path = config
            .game_dir
            .join(".forge")
            .join("profiles")
            .join(&version_name);

        let installer_json_path = profiles_path.join(format!("installer-{}.json", &version_name));
        let version_json_path = profiles_path.join(format!("version-{}.json", &version_name));
        let installer_path = temp_dir().join(format!("forge-{}.jar", version_name));

        let mut installer: Installer = if installer_json_path.is_file() {
            read_json(&installer_json_path).await?
        } else {
            download_installer(&installer_path, &version_name, emitter).await?;
            extract_specific_file(
                &installer_path,
                "install_profile.json",
                &installer_json_path,
            )
            .await?;
            read_json(&installer_json_path).await?
        };

        let version: CustomMeta = if version_json_path.is_file() {
            read_json(&version_json_path).await?
        } else {
            download_installer(&installer_path, &version_name, emitter).await?;
            extract_specific_file(&installer_path, "version.json", &version_json_path).await?;
            read_json(&version_json_path).await?
        };

        process_data(config, &installer_path, &mut installer.data).await?;
        
        meta.data = Some(merge_data(
            config,
            &meta,
            installer.data.unwrap_or_default(),
        ));

        meta.processors = installer.processors;

        extract_specific_directory(
            &installer_path,
            "maven/",
            &config.game_dir.join("libraries"),
        )
        .await
        .ok();

        meta.libraries.retain(|lib| {
            version
                .libraries
                .iter()
                .all(|v_lib| v_lib.name.split(':').nth(1) != lib.name.split(':').nth(1))
        });

        let mut seen = HashSet::new();

        meta.libraries
            .extend(merge_libraries(config, version.libraries, &mut seen, false));
        meta.libraries.extend(merge_libraries(
            config,
            installer.libraries,
            &mut seen,
            true,
        ));

        if let Some(ref mut arguments) = meta.arguments {
            if let Some(jvm) = version.arguments.jvm {
                arguments.jvm.extend(jvm);
            }
            if let Some(game) = version.arguments.game {
                arguments.game.extend(game);
            }
        }

        meta.main_class = version.main_class;

        Ok(meta)
    }

    fn get_version(&self) -> String {
        self.0.to_string()
    }
}

async fn download_installer(
    installer_path: &std::path::Path,
    version_name: &str,
    emitter: Option<&Emitter>,
) -> crate::Result<()> {
    if !installer_path.is_file() {
        let installer_url = INSTALLER_JAR_ENDPOINT.replace("{loader_version}", version_name);
        download(installer_url, installer_path, emitter).await?;
    }
    Ok(())
}

fn merge_data(
    config: &Config<impl Loader>,
    meta: &VersionMeta,
    installer_data: HashMap<String, Data>,
) -> HashMap<String, Data> {
    [
        (
            "SIDE".to_string(),
            Data {
                client: "client".to_string(),
                server: "".to_string(),
            },
        ),
        (
            "MINECRAFT_VERSION".to_string(),
            Data {
                client: meta.id.clone(),
                server: "".to_string(),
            },
        ),
        (
            "ROOT".to_string(),
            Data {
                client: config.game_dir.to_string_lossy().into_owned(),
                server: "".to_string(),
            },
        ),
        (
            "LIBRARY_DIR".to_string(),
            Data {
                client: config
                    .game_dir
                    .join("libraries")
                    .to_string_lossy()
                    .into_owned(),
                server: "".to_string(),
            },
        ),
        (
            "MINECRAFT_JAR".to_string(),
            Data {
                client: config.get_version_jar_path().to_string_lossy().into_owned(),
                server: "".to_string(),
            },
        ),
    ]
    .into_iter()
    .chain(installer_data)
    .collect()
}

async fn process_data(
    config: &Config<impl Loader>,
    installer_path: &std::path::PathBuf,
    data: &mut Option<HashMap<String, Data>>,
) -> crate::Result<()> {
    if let Some(ref mut data) = data {
        for value in data.values_mut() {
            if value.client.starts_with('/') {
                let file_path = &value.client[1..];
                let file = file_path.split('/').last().ok_or(crate::Error::NotFound(
                    "File not found for the processor".to_string(),
                ))?;
                let file_name = file.split('.').next().ok_or(crate::Error::NotFound(
                    "File name not found for the processor".to_string(),
                ))?;
                let ext = file.split('.').last().ok_or(crate::Error::NotFound(
                    "File extension not found for the processor".to_string(),
                ))?;
                let path = format!(
                    "com.cubidron.lyceris:forge-installer-extracts:{}:{}@{}",
                    config.version, file_name, ext
                );

                extract_specific_file(
                    installer_path,
                    file_path,
                    &config
                        .game_dir
                        .join("libraries")
                        .join(parse_lib_path(&path)?),
                )
                .await?;

                value.client = format!("[{}]", path);
            }
        }
    }
    Ok(())
}

fn merge_libraries(
    config: &Config<impl Loader>,
    libraries: Vec<Library>,
    seen: &mut HashSet<String>,
    skip_args: bool,
) -> Vec<vanilla::Library> {
    libraries
        .into_iter()
        .filter_map(|lib| {
            if !seen.insert(lib.name.clone()) {
                return None;
            }

            if let Some(url) = lib.url {
                let path = parse_lib_path(&lib.name).ok()?;
                return Some(vanilla::Library {
                    downloads: Some(vanilla::LibraryDownloads {
                        artifact: Some(vanilla::File {
                            path: Some(
                                config
                                    .get_libraries_path()
                                    .join(&path)
                                    .to_string_lossy()
                                    .into_owned(),
                            ),
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
                    skip_args,
                });
            }

            if let Some(downloads) = lib.downloads {
                if let Some(artifact) = downloads.artifact {
                    if let Some(path) = artifact.path {
                        return Some(vanilla::Library {
                            downloads: Some(vanilla::LibraryDownloads {
                                artifact: Some(vanilla::File {
                                    path: Some(
                                        config
                                            .get_libraries_path()
                                            .join(path)
                                            .to_string_lossy()
                                            .into_owned(),
                                    ),
                                    sha1: lib.sha1.unwrap_or_default(),
                                    size: lib.size.unwrap_or_default(),
                                    url: artifact.url,
                                }),
                                classifiers: None,
                            }),
                            extract: None,
                            name: lib.name.clone(),
                            rules: None,
                            natives: None,
                            skip_args,
                        });
                    }
                }
            }

            None
        })
        .collect()
}
