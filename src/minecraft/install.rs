use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{
    env::consts::{ARCH, OS},
    fs,
    path::{Path, PathBuf, MAIN_SEPARATOR_STR},
};
use tokio::{fs::create_dir_all, process::Command};

use crate::{
    error::Error,
    http::{
        downloader::{download, download_multiple},
        fetch::fetch,
    },
    json::{
        java::{JavaFileManifest, JavaManifest},
        version::{
            asset_index::AssetIndex,
            manifest::VersionManifest,
            meta::vanilla::{self, JavaVersion, VersionMeta},
        },
    },
    minecraft::{
        CLASSPATH_SEPARATOR, JAVA_MANIFEST_ENDPOINT, RESOURCES_ENDPOINT, VERSION_MANIFEST_ENDPOINT,
    },
    util::{
        extract::{extract_file, read_file_from_jar},
        hash::calculate_sha1,
        json::{read_json, write_json},
    },
};

use super::{
    config::Config,
    emitter::Emitter,
    loader::Loader,
    parse::{parse_lib_path, ParseRule},
};

#[derive(Clone)]
enum FileType {
    Asset { is_virtual: bool, is_map: bool },
    Library,
    Java,
}

#[derive(Clone)]
struct DownloadFile {
    file_name: String,
    sha1: String,
    url: String,
    path: PathBuf,
    r#type: FileType,
}

pub async fn install<T: Loader>(
    config: &Config<T>,
    emitter: Option<&Emitter>,
) -> crate::Result<()> {
    let manifest: VersionManifest =
        fetch(VERSION_MANIFEST_ENDPOINT, config.client.as_ref()).await?;
    let version_json_path = config.get_version_json_path();
    let mut meta: VersionMeta = if !version_json_path.exists() {
        let mut meta =
            fetch_version_meta(&manifest, &config.version, config.client.as_ref()).await?;
        if let Some(loader) = &config.loader {
            meta = loader.merge(&config.into_vanilla(), meta, emitter).await?;
        }
        write_json(version_json_path, &meta).await?;
        meta
    } else {
        read_json(version_json_path).await?
    };

    let asset_index_path = config
        .get_indexes_path()
        .join(format!("{}.json", &meta.asset_index.id));
    let asset_index: AssetIndex = if !asset_index_path.exists() {
        let asset_index = fetch(&meta.asset_index.url, config.client.as_ref()).await?;
        write_json(asset_index_path, &asset_index).await?;
        asset_index
    } else {
        read_json(asset_index_path).await?
    };

    let version_jar_path = config.get_version_jar_path();
    if !version_jar_path.exists()
        || !calculate_sha1(&version_jar_path)?.eq(&meta.downloads.client.sha1)
    {
        download(
            &meta.downloads.client.url,
            version_jar_path,
            emitter,
            config.client.as_ref(),
        )
        .await?;
    }

    let natives_path = config.get_natives_path().join(&config.version);
    if !natives_path.is_dir() {
        create_dir_all(&natives_path).await?;
    }

    let check_natives = fs::read_dir(&natives_path)?.count() == 0;
    let mut to_be_extracted = Vec::with_capacity(10);

    let default_java_version = JavaVersion::default();
    let java_version = meta.java_version.as_ref().unwrap_or(&default_java_version);
    let runtime_path = config.get_runtime_path().join(&java_version.component);

    let java_manifest: JavaManifest = fetch(JAVA_MANIFEST_ENDPOINT, config.client.as_ref()).await?;
    let java_url = get_java_url(&java_manifest, java_version)?;
    let java_files: JavaFileManifest = fetch(java_url, config.client.as_ref()).await?;

    let file_map = build_file_map(
        &asset_index,
        &meta,
        &java_files,
        &runtime_path,
        config,
        check_natives,
        &mut to_be_extracted,
    )?;

    download_necessary(
        file_map,
        &config.game_dir,
        asset_index.map_to_resources.unwrap_or_default()
            || asset_index.r#virtual.unwrap_or_default(),
        emitter,
        config.client.as_ref(),
    )
    .await?;

    if !to_be_extracted.is_empty() {
        create_dir_all(&natives_path).await?;
        for extract in to_be_extracted {
            let path = PathBuf::from(extract.path.unwrap());
            download(&extract.url, &path, emitter, config.client.as_ref()).await?;
            extract_file(&path, &natives_path)?;
        }
    }

    execute_processors_if_exists(&mut meta, config).await?;

    Ok(())
}

async fn fetch_version_meta(
    manifest: &VersionManifest,
    version: &str,
    client: Option<&reqwest::Client>,
) -> crate::Result<VersionMeta> {
    let version_url = manifest
        .versions
        .iter()
        .find(|v| v.id == version)
        .ok_or_else(|| Error::UnknownVersion("Vanilla".to_string()))?
        .url
        .clone();
    fetch(&version_url, client).await
}

fn get_java_url(java_manifest: &JavaManifest, java_version: &JavaVersion) -> crate::Result<String> {
    let os = if OS == "macos" { "mac-os" } else { OS };
    let arch = match ARCH {
        "x86" => {
            if os == "linux" {
                "i386"
            } else {
                "x86"
            }
        }
        "x86_64" => "x64",
        "aarch64" => "arm64",
        _ => return Err(Error::UnsupportedArchitecture),
    };
    let os_arch = if (os == "linux" && arch != "i386")
        || (os == "mac-os" && (arch != "arm64" || java_version.major_version == 8))
    {
        os.to_string()
    } else {
        format!("{}-{}", os, arch)
    };
    java_manifest
        .get(&os_arch)
        .ok_or_else(|| Error::NotFound("Java map by operating system".to_string()))?
        .get(&java_version.component)
        .ok_or_else(|| Error::UnknownVersion("Java version".to_string()))?
        .first()
        .ok_or_else(|| Error::NotFound("Java gamecore".to_string()))
        .map(|entry| &entry.manifest.url)
        .cloned()
}

fn build_file_map(
    asset_index: &AssetIndex,
    meta: &VersionMeta,
    java_files: &JavaFileManifest,
    runtime_path: &Path,
    config: &Config<impl Loader>,
    check_natives: bool,
    to_be_extracted: &mut Vec<vanilla::File>,
) -> crate::Result<Vec<DownloadFile>> {
    let asset_files = asset_index
        .objects
        .iter()
        .map(|(key, meta)| {
            let assets_path = config.get_assets_path();
            let hash = &meta.hash;
            DownloadFile {
                file_name: key.clone(),
                sha1: hash.clone(),
                url: format!("{}/{}/{}", RESOURCES_ENDPOINT, &hash[0..2], hash),
                path: assets_path.join("objects").join(&hash[0..2]).join(hash),
                r#type: FileType::Asset {
                    is_map: asset_index.map_to_resources.unwrap_or_default(),
                    is_virtual: asset_index.r#virtual.unwrap_or_default(),
                },
            }
        })
        .collect::<Vec<_>>();

    let library_files = meta
        .libraries
        .iter()
        .filter_map(|lib| {
            if !lib.rules.parse_rule() {
                return None;
            }
            let downloads = lib.downloads.as_ref()?;
            if check_natives {
                if let Some(classifiers) = &downloads.classifiers {
                    let classifier = match OS {
                        "windows" => &classifiers.natives_windows,
                        "linux" => &classifiers.natives_linux,
                        "macos" => &classifiers.natives_macos,
                        _ => return None,
                    };
                    if let Some(classifier) = classifier {
                        if let Some(classifier_path) = &classifier.path {
                            let path = config
                                .game_dir
                                .join("libraries")
                                .join(classifier_path.replace("/", MAIN_SEPARATOR_STR));
                            let url = classifier.url.clone();
                            let sha1 = classifier.sha1.clone();
                            to_be_extracted.push(vanilla::File {
                                path: Some(path.to_string_lossy().into_owned()),
                                sha1: sha1.clone(),
                                size: classifier.size,
                                url: url.clone(),
                            });
                            return Some(DownloadFile {
                                file_name: PathBuf::from(url.clone())
                                    .file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .to_string(),
                                sha1,
                                url,
                                path,
                                r#type: FileType::Library,
                            });
                        }
                    }
                }
            }
            let artifact = downloads.artifact.as_ref()?;
            Some(DownloadFile {
                file_name: PathBuf::from(artifact.url.clone())
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                sha1: artifact.sha1.clone(),
                url: artifact.url.clone(),
                path: config
                    .game_dir
                    .join("libraries")
                    .join(artifact.path.as_ref()?.replace("/", MAIN_SEPARATOR_STR)),
                r#type: FileType::Library,
            })
        })
        .collect::<Vec<_>>();

    let java_files = java_files
        .files
        .iter()
        .filter_map(|(name, file)| {
            let path = runtime_path.join(name.replace("/", MAIN_SEPARATOR_STR));
            file.downloads.as_ref().map(|downloads| DownloadFile {
                file_name: name
                    .split(MAIN_SEPARATOR_STR)
                    .last()
                    .unwrap_or(name)
                    .to_string(),
                path,
                sha1: downloads.raw.sha1.clone(),
                url: downloads.raw.url.clone(),
                r#type: FileType::Java,
            })
        })
        .collect::<Vec<_>>();

    Ok([asset_files, library_files, java_files].concat())
}

async fn execute_processors_if_exists(
    meta: &mut VersionMeta,
    config: &Config<impl Loader>,
) -> crate::Result<()> {
    if let Some(ref mut processors) = meta.processors {
        let data = meta
            .data
            .as_ref()
            .ok_or_else(|| Error::NotFound("Forge Installer Data".to_string()))?;

        let libraries_path = config.get_libraries_path();

        for processor in processors {
            if let Some(sides) = &processor.sides {
                if !sides.contains(&"client".to_string()) {
                    continue;
                }
            }

            if processor.success {
                continue;
            }

            let classpath = processor
                .classpath
                .iter()
                .filter_map(|arg| {
                    Some(
                        libraries_path
                            .join(parse_lib_path(arg).ok()?)
                            .to_string_lossy()
                            .into_owned(),
                    )
                })
                .collect::<Vec<String>>()
                .join(CLASSPATH_SEPARATOR);

            let main_class = read_file_from_jar(
                &libraries_path
                    .join(parse_lib_path(&processor.jar)?)
                    .to_string_lossy()
                    .into_owned(),
                "META-INF/MANIFEST.MF",
            )?
            .lines()
            .find(|line| line.starts_with("Main-Class:"))
            .ok_or_else(|| Error::NotFound("Main-Class of processor".to_string()))?
            .split(":")
            .last()
            .ok_or_else(|| Error::NotFound("Main-Class of processor".to_string()))?
            .trim()
            .to_string();

            let args = processor
                .args
                .iter()
                .map(|arg| {
                    let trimmed_arg = &arg[1..arg.len() - 1];
                    if arg.starts_with('{') {
                        if let Some(entry) = data.get(trimmed_arg) {
                            if entry.client.starts_with('[') {
                                if let Ok(parsed_path) =
                                    parse_lib_path(&entry.client[1..entry.client.len() - 1])
                                {
                                    return libraries_path
                                        .join(parsed_path)
                                        .to_string_lossy()
                                        .into_owned();
                                }
                            }
                            return entry.client.clone();
                        }
                    } else if arg.starts_with('[') {
                        if let Ok(parsed_path) = parse_lib_path(trimmed_arg) {
                            return libraries_path
                                .join(parsed_path)
                                .to_string_lossy()
                                .into_owned();
                        }
                    }

                    arg.clone()
                })
                .collect::<Vec<_>>();

            let child = Command::new(
                config
                    .get_java_path(
                        meta.java_version
                            .as_ref()
                            .unwrap_or(&JavaVersion::default()),
                    )
                    .await?,
            )
            .arg("-cp")
            .arg(format!(
                "{}{}{}",
                classpath,
                CLASSPATH_SEPARATOR,
                libraries_path
                    .join(parse_lib_path(&processor.jar)?)
                    .to_string_lossy()
                    .into_owned()
            ))
            .arg(main_class)
            .args(args)
            .output()
            .await?;

            if child.status.success() {
                processor.success = true;
            } else {
                return Err(Error::Fail(format!(
                    "Processor failed: {}",
                    String::from_utf8_lossy(&child.stderr)
                )));
            }
        }
    }

    write_json(config.get_version_json_path(), &meta).await?;

    Ok(())
}

async fn download_necessary(
    files: Vec<DownloadFile>,
    game_dir: &Path,
    legacy: bool,
    emitter: Option<&Emitter>,
    client: Option<&reqwest::Client>,
) -> crate::Result<()> {
    let broken_ones: Vec<(String, PathBuf)> = files
        .par_iter()
        .filter_map(|file| {
            if file.url.is_empty() {
                return None;
            }
            if !file.path.exists()
                || (!file.sha1.is_empty() && calculate_sha1(&file.path).ok()? != file.sha1)
            {
                return Some((file.url.clone(), file.path.clone()));
            }
            None
        })
        .collect();

    download_multiple(broken_ones, emitter, client).await?;

    if legacy {
        files.par_iter().try_for_each(|file| {
            if let FileType::Asset { is_virtual, is_map } = file.r#type {
                let target_path = if is_virtual {
                    game_dir
                        .join("assets")
                        .join("virtual")
                        .join("legacy")
                        .join(&file.file_name)
                } else if is_map {
                    game_dir.join("resources").join(&file.file_name)
                } else {
                    return None::<()>;
                };

                if let Some(parent) = target_path.parent() {
                    if !parent.is_dir() {
                        fs::create_dir_all(parent).ok();
                    }
                }

                if !target_path.exists() || calculate_sha1(&target_path).ok()? != file.sha1 {
                    fs::copy(&file.path, &target_path).ok();
                }

                return None;
            }

            None
        });
    }

    Ok(())
}
