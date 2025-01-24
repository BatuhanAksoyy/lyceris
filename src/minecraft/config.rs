use std::path::{Path, PathBuf};

#[cfg(not(target_os = "windows"))]
use std::os::unix::fs::PermissionsExt;

use serde::{Deserialize, Serialize};

use crate::{auth::AuthMethod, json::version::meta::vanilla::JavaVersion};

use super::loader::Loader;

#[derive(Serialize, Deserialize)]
pub enum Memory {
    Megabyte(u64),
    Gigabyte(u16),
}

#[derive(Serialize, Deserialize)]
pub struct Config<T: Loader> {
    pub game_dir: PathBuf,
    pub version: String,
    pub authentication: AuthMethod,
    pub memory: Option<Memory>,
    pub version_name: Option<String>,
    pub loader: Option<T>,
    pub java_version: Option<String>,
    pub runtime_dir: Option<PathBuf>,
    pub custom_java_args: Vec<String>,
    pub custom_args: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ConfigBuilder {
    game_dir: PathBuf,
    version: String,
    authentication: AuthMethod,
    memory: Option<Memory>,
    version_name: Option<String>,
    java_version: Option<String>,
    loader: Option<String>,
    runtime_dir: Option<PathBuf>,
    custom_java_args: Vec<String>,
    custom_args: Vec<String>,
}

impl ConfigBuilder {
    pub fn new<T: AsRef<Path>>(
        game_dir: T,
        version: String,
        authentication: AuthMethod,
    ) -> ConfigBuilder {
        ConfigBuilder {
            game_dir: game_dir.as_ref().to_path_buf(),
            version,
            authentication,
            memory: None,
            version_name: None,
            java_version: None,
            loader: None,
            runtime_dir: None,
            custom_java_args: Vec::new(),
            custom_args: Vec::new(),
        }
    }

    pub fn memory(mut self, memory: Memory) -> Self {
        self.memory = Some(memory);
        self
    }

    pub fn version_name(mut self, version_name: String) -> Self {
        self.version_name = Some(version_name);
        self
    }
    pub fn java_version(mut self, java_version: String) -> Self {
        self.java_version = Some(java_version);
        self
    }

    pub fn runtime_dir(mut self, runtime_dir: PathBuf) -> Self {
        self.runtime_dir = Some(runtime_dir);
        self
    }

    pub fn custom_java_args(mut self, custom_java_args: Vec<String>) -> Self {
        self.custom_java_args = custom_java_args;
        self
    }

    pub fn custom_args(mut self, custom_args: Vec<String>) -> Self {
        self.custom_args = custom_args;
        self
    }

    pub fn build<C: Loader>(self, loader: Option<C>) -> Config<C> {
        Config {
            game_dir: self.game_dir,
            version: self.version.to_string(),
            authentication: self.authentication,
            memory: self.memory,
            version_name: self.version_name,
            loader,
            java_version: self.java_version,
            runtime_dir: self.runtime_dir,
            custom_java_args: self.custom_java_args,
            custom_args: self.custom_args,
        }
    }
}

impl<T: Loader> Config<T> {
    pub fn new(game_dir: PathBuf, version: String, authentication: AuthMethod) -> Self {
        Self {
            game_dir,
            version,
            authentication,
            memory: None,
            version_name: None,
            loader: None,
            java_version: None,
            runtime_dir: None,
            custom_java_args: Vec::new(),
            custom_args: Vec::new(),
        }
    }

    pub fn get_version_name(&self) -> String {
        self.version_name
            .as_ref()
            .map(|name| name.to_owned())
            .or_else(|| {
                self.loader
                    .as_ref()
                    .map(|loader| format!("{}-{}", self.version, loader.get_version()))
            })
            .unwrap_or_else(|| self.version.to_string())
    }

    pub fn get_libraries_path(&self) -> PathBuf {
        self.game_dir.join("libraries")
    }

    pub async fn get_java_path(&self, version: &JavaVersion) -> crate::Result<PathBuf> {
        #[cfg(not(target_os = "macos"))]
        let java_path = self
            .get_runtime_path()
            .join(version.component.clone())
            .join("bin")
            .join("java");

        #[cfg(target_os = "macos")]
        let java_path = self
            .get_runtime_path()
            .join(&version.component)
            .join("jre.bundle")
            .join("Contents")
            .join("Home")
            .join("bin")
            .join("java");

        #[cfg(not(target_os = "windows"))]
        {
            let mut perms = tokio::fs::metadata(&java_path).await?.permissions();
            perms.set_mode(0o755);
            tokio::fs::set_permissions(&java_path, perms).await?;
        }

        Ok(java_path)
    }

    pub fn get_versions_path(&self) -> PathBuf {
        self.game_dir.join("versions")
    }

    pub fn get_assets_path(&self) -> PathBuf {
        self.game_dir.join("assets")
    }

    pub fn get_natives_path(&self) -> PathBuf {
        self.game_dir.join("natives")
    }

    pub fn get_runtime_path(&self) -> PathBuf {
        self.runtime_dir
            .clone()
            .unwrap_or_else(|| self.game_dir.join("runtimes"))
    }

    pub fn get_indexes_path(&self) -> PathBuf {
        self.get_assets_path().join("indexes")
    }

    pub fn get_version_path(&self) -> PathBuf {
        self.get_versions_path().join(self.get_version_name())
    }

    pub fn get_version_json_path(&self) -> PathBuf {
        self.get_version_path()
            .join(format!("{}.json", self.get_version_name()))
    }

    pub fn get_version_jar_path(&self) -> PathBuf {
        self.get_version_path()
            .join(format!("{}.jar", self.get_version_name()))
    }
}
