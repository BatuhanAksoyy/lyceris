use std::{collections::HashMap, env::temp_dir, fs::remove_file, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    download, download_multiple,
    http::fetch::fetch,
    minecraft::{emitter::Emitter, install::FileType},
    util::{
        extract::{extract_specific_directory, read_file_from_jar},
        hash::calculate_sha1,
    },
};

const MODRINTH_API: &str = "https://api.modrinth.com/v2";

#[derive(Serialize, Deserialize, Debug)]
struct ModrinthVersion {
    files: Vec<ModrinthAPIFile>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ModrinthAPIFile {
    hashes: HashMap<String, String>,
    url: String,
    filename: String,
    primary: bool,
    size: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct ModrinthIndex {
    files: Vec<ModrinthIndexFile>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ModrinthIndexFile {
    path: String,
    hashes: HashMap<String, String>,
    downloads: Vec<String>,
    #[serde(rename = "fileSize")]
    file_size: u64,
}

// Installs modrinth pack by downloading .mrpack file.
// This method might not be the best way to install modrinth packs,
// but it works for now. It downloads the .mrpack file, extracts it, and downloads the files inside it.
pub async fn install_modrinth_pack(
    root_path: PathBuf,
    version_id: String,
    emitter: Option<&Emitter>,
) -> crate::Result<()> {
    let temp_dir = temp_dir();
    let modrinth_version_url = format!("{}/version/{}", MODRINTH_API, version_id);
    let modrinth_version = fetch::<ModrinthVersion>(modrinth_version_url, None).await?;

    let primary_file = modrinth_version
        .files
        .iter()
        .find(|file| file.primary)
        .ok_or_else(|| {
            crate::Error::Integration(format!(
                "No modrinth primary file found for version {}",
                version_id
            ))
        })?;

    let mr_file_path = temp_dir.join(&primary_file.filename);
    download(&primary_file.url, &mr_file_path, emitter, None).await?;

    extract_specific_directory(&mr_file_path, "overrides", &root_path)?;

    let index_content = read_file_from_jar(&mr_file_path, "modrinth.index.json")?;
    let index = serde_json::from_str::<ModrinthIndex>(&index_content)?;

    let files_to_download = index
        .files
        .iter()
        .filter_map(|file| {
            let file_path = root_path.join(&file.path);
            let sha1 = file
                .hashes
                .get("sha1")
                .unwrap_or(&String::default())
                .clone();

            if file_path.exists()
                && !sha1.is_empty()
                && calculate_sha1(file_path.clone()).ok()? == *sha1
            {
                None
            } else {
                file.downloads
                    .first()
                    .map(|url| (url.clone(), file_path, FileType::Custom))
            }
        })
        .collect::<Vec<_>>();

    download_multiple(files_to_download, emitter, None).await?;
    remove_file(mr_file_path)?;

    Ok(())
}
