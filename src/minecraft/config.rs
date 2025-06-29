use std::path::{Path, PathBuf};

#[cfg(not(target_os = "windows"))]
use std::os::unix::fs::PermissionsExt;

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{auth::AuthMethod, json::version::meta::vanilla::JavaVersion};

use super::loader::Loader;

#[derive(Serialize, Deserialize, Clone)]
pub enum Memory {
    Megabyte(u64),
    Gigabyte(u16),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub root: PathBuf,
}

impl Profile {
    pub fn new(name: String, root: PathBuf) -> Self {
        Self { name, root }
    }

    pub fn change_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn change_root(&mut self, root: PathBuf) {
        self.root = root;
    }
}

/// Configuration structure for managing Minecraft installation settings.
#[derive(Serialize, Deserialize, Clone)]
pub struct Config<T: Loader> {
    pub game_dir: PathBuf,
    pub version: String,
    pub authentication: AuthMethod,
    pub memory: Option<Memory>,
    pub version_name: Option<String>,
    pub profile: Option<Profile>,
    pub loader: Option<T>,
    pub java_version: Option<String>,
    pub runtime_dir: Option<PathBuf>,
    pub custom_java_args: Vec<String>,
    pub custom_args: Vec<String>,
    #[serde(skip)]
    pub client: Option<Client>
}

impl<T: Loader> Config<T> {
    pub fn into_vanilla(&self) -> Config<()> {
        Config {
            game_dir: self.game_dir.clone(),
            version: self.version.clone(),
            authentication: self.authentication.clone(),
            memory: self.memory.clone(),
            version_name: self.version_name.clone(),
            loader: None,
            profile: self.profile.clone(),
            java_version: self.java_version.clone(),
            runtime_dir: self.runtime_dir.clone(),
            custom_java_args: self.custom_java_args.clone(),
            custom_args: self.custom_args.clone(),
            client: self.client.clone()
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ConfigBuilder<T: Loader = ()> {
    game_dir: PathBuf,
    version: String,
    authentication: AuthMethod,
    memory: Option<Memory>,
    version_name: Option<String>,
    pub profile: Option<Profile>,
    loader: Option<T>,
    java_version: Option<String>,
    runtime_dir: Option<PathBuf>,
    custom_java_args: Vec<String>,
    custom_args: Vec<String>,
    #[serde(skip)]
    client: Option<Client>  
}

impl ConfigBuilder<()> {
    pub fn new<T: AsRef<Path>>(
        game_dir: T,
        version: String,
        authentication: AuthMethod,
    ) -> ConfigBuilder<()> {
        ConfigBuilder {
            game_dir: game_dir.as_ref().to_path_buf(),
            version,
            authentication,
            memory: None,
            version_name: None,
            loader: None,
            java_version: None,
            profile: None,
            runtime_dir: None,
            custom_java_args: Vec::new(),
            custom_args: Vec::new(),
            client: None
        }
    }
}

impl<T: Loader> ConfigBuilder<T> {
    pub fn memory(mut self, memory: Memory) -> Self {
        self.memory = Some(memory);
        self
    }

    pub fn version_name(mut self, version_name: String) -> Self {
        self.version_name = Some(version_name);
        self
    }

    pub fn loader(self, loader: Box<dyn Loader>) -> ConfigBuilder<Box<dyn Loader>> {
        ConfigBuilder {
            game_dir: self.game_dir,
            version: self.version,
            authentication: self.authentication,
            memory: self.memory,
            version_name: self.version_name,
            profile: self.profile,
            loader: Some(loader),
            java_version: self.java_version,
            runtime_dir: self.runtime_dir,
            custom_java_args: self.custom_java_args,
            custom_args: self.custom_args,
            client: self.client
        }
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

    pub fn client(mut self, client: Client) -> Self {
        self.client = Some(client);
        self
    }

    pub fn profile(mut self, profile: Profile) -> Self {
        self.profile = Some(profile);
        self
    }

    pub fn build(self) -> Config<T> {
        Config {
            game_dir: self.game_dir,
            version: self.version,
            authentication: self.authentication,
            memory: self.memory,
            version_name: self.version_name,
            loader: self.loader,
            java_version: self.java_version,
            runtime_dir: self.runtime_dir,
            profile: self.profile,
            custom_java_args: self.custom_java_args,
            custom_args: self.custom_args,
            client: self.client
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
            profile: None,
            loader: None,
            java_version: None,
            runtime_dir: None,
            custom_java_args: Vec::new(),
            custom_args: Vec::new(),
            client: None
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

    /// Gets the path to the Java executable for the specified version.
    ///
    /// # Parameters
    /// - `version`: The Java version for which to retrieve the path.
    ///
    /// # Returns
    /// A result containing the path to the Java executable.
    pub async fn get_java_path(&self, version: &JavaVersion) -> crate::Result<PathBuf> {
        #[cfg(target_os = "windows")]
        let java_path = self
            .get_runtime_path()
            .join(version.component.clone())
            .join("bin")
            .join("javaw");

        #[cfg(target_os = "linux")]
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

    /// Gets the path to the versions directory.
    ///
    /// # Returns
    /// The path to the versions directory.
    pub fn get_versions_path(&self) -> PathBuf {
        self.game_dir.join("versions")
    }

    /// Gets the path to the assets directory.
    ///
    /// # Returns
    /// The path to the assets directory.
    pub fn get_assets_path(&self) -> PathBuf {
        self.game_dir.join("assets")
    }

    /// Gets the path to the natives directory.
    ///
    /// # Returns
    /// The path to the natives directory.
    pub fn get_natives_path(&self) -> PathBuf {
        self.game_dir.join("natives")
    }

    /// Gets the path to the runtime directory.
    ///
    /// # Returns
    /// The path to the runtime directory.
    pub fn get_runtime_path(&self) -> PathBuf {
        self.runtime_dir
            .clone()
            .unwrap_or_else(|| self.game_dir.join("runtimes"))
    }

    /// Gets the path to the indexes directory.
    ///
    /// # Returns
    /// The path to the indexes directory.
    pub fn get_indexes_path(&self) -> PathBuf {
        self.get_assets_path().join("indexes")
    }

    /// Gets the path to the version directory.
    ///
    /// # Returns
    /// The path to the version directory.
    pub fn get_version_path(&self) -> PathBuf {
        self.get_versions_path().join(self.get_version_name())
    }

    /// Gets the path to the version JSON file.
    ///
    /// # Returns
    /// The path to the version JSON file.
    pub fn get_version_json_path(&self) -> PathBuf {
        self.get_version_path()
            .join(format!("{}.json", self.get_version_name()))
    }

    pub fn get_version_jar_path(&self) -> PathBuf {
        self.get_version_path()
            .join(format!("{}.jar", self.get_version_name()))
    }
}
