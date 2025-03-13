use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::custom::{Data, Processor};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Arguments>,
    pub asset_index: AssetIndex,
    pub assets: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compliance_level: Option<i64>,
    pub downloads: Downloads,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub java_version: Option<JavaVersion>,
    pub libraries: Vec<Library>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging: Option<Logging>,
    pub main_class: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum_launcher_version: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_arguments: Option<String>,
    pub release_time: String,
    pub time: String,
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processors: Option<Vec<Processor>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<HashMap<String, Data>>
}

#[derive(Serialize, Deserialize)]
pub struct Arguments {
    pub game: Vec<Element>,
    pub jvm: Vec<Element>,
}

#[derive(Serialize, Deserialize)]
pub struct GameClass {
    pub rules: Vec<Rule>,
    pub value: Value,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Features {
    pub is_demo_user: Option<bool>,
    pub has_custom_resolution: Option<bool>,
    pub has_quick_plays_support: Option<bool>,
    pub is_quick_play_singleplayer: Option<bool>,
    pub is_quick_play_multiplayer: Option<bool>,
    pub is_quick_play_realms: Option<bool>,
}

#[derive(Serialize, Deserialize)]
pub struct Class {
    pub rules: Vec<Rule>,
    pub value: Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Rule {
    pub action: Action,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os: Option<Os>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub features: Option<Features>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Extract {
    #[serde(rename = "exclude")]
    pub exclude: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Os {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<Name>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arch: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetIndex {
    pub id: String,
    pub sha1: String,
    pub size: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_size: Option<i64>,
    pub url: String,
}

#[derive(Serialize, Deserialize)]
pub struct Downloads {
    pub client: File,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_mappings: Option<File>,
    pub server: File,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_mappings: Option<File>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct File {
    pub sha1: String,
    pub size: i64,
    pub url: String,
    pub path: Option<String>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct JavaVersion {
    #[serde(default = "default_java_version")]
    pub component: String,
    pub major_version: i64,
}

fn default_java_version() -> String {
    "jre-legacy".to_string()
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Library {
    pub downloads: Option<LibraryDownloads>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<Rule>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extract: Option<Extract>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub natives: Option<Natives>,
    #[serde(default)]
    pub skip_args: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Natives {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linux: Option<String>,
    #[serde(rename = "linux-arm64")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linux_arm64: Option<String>,
    #[serde(rename = "linux-arm32")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linux_arm32: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub osx: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub windows: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LibraryDownloads {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact: Option<File>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classifiers: Option<Classifiers>,
}

#[derive(Serialize, Deserialize)]
pub struct Logging {
    pub client: LoggingClient,
}

#[derive(Serialize, Deserialize)]
pub struct LoggingClient {
    pub argument: String,
    pub file: AssetIndex,
    pub r#type: String,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Element {
    Class(Class),

    String(String),
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    Single(String),

    Multiple(Vec<String>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Classifiers {
    #[serde(rename = "natives-linux")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub natives_linux: Option<File>,

    #[serde(rename = "natives-osx")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub natives_osx: Option<File>,

    #[serde(rename = "natives-macos")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub natives_macos: Option<File>,

    #[serde(rename = "natives-windows")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub natives_windows: Option<File>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Action {
    #[serde(rename = "allow")]
    Allow,
    #[serde(rename = "disallow")]
    Disallow,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum Name {
    #[serde(rename = "linux")]
    Linux,

    #[serde(rename = "osx")]
    Osx,

    #[serde(rename = "osx-arm64")]
    OsxArm64,

    #[serde(rename = "windows")]
    Windows,

    #[serde(rename = "linux-arm64")]
    LinuxArm64,

    #[serde(rename = "linux-arm32")]
    LinuxArm32,
}
