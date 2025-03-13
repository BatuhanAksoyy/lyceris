use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::vanilla::{Element, LibraryDownloads};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomMeta {
    pub id: String,
    pub inherits_from: String,
    pub release_time: String,
    pub time: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    pub main_class: String,
    pub arguments: Arguments,
    pub libraries: Vec<Library>,
}

#[derive(Serialize, Deserialize)]
pub struct Arguments {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game: Option<Vec<Element>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jvm: Option<Vec<Element>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Library {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub md5: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha1: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha512: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub downloads: Option<LibraryDownloads>,
}

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