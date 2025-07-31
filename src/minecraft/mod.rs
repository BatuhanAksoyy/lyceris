pub mod install;
pub mod launch;
pub mod loader;
pub mod parse;
pub mod emitter;
pub mod config;

#[cfg(target_os = "windows")]
pub const TARGET_OS: &str = "windows";
#[cfg(target_os = "macos")]
pub const TARGET_OS: &str = "osx";

#[cfg(target_os = "linux")]
pub const TARGET_OS: &str = "linux";

#[cfg(target_arch = "x86")]
pub const TARGET_ARCH: &str = "x86";

#[cfg(target_arch = "x86_64")]
pub const TARGET_ARCH: &str = "x86_64";

#[cfg(target_arch = "aarch64")]
pub const TARGET_ARCH: &str = "aarch64";

#[cfg(target_os = "windows")]
pub const CLASSPATH_SEPARATOR: &str = ";";

#[cfg(not(target_os = "windows"))]
pub const CLASSPATH_SEPARATOR: &str = ":";

pub const JAVA_MANIFEST_ENDPOINT: &str = 
    "https://launcher.baso.network/download/all.json";

pub const VERSION_MANIFEST_ENDPOINT: &str =
    "https://launcher.baso.network/download/version_manifest_v2.json";
    
pub const RESOURCES_ENDPOINT: &str = 
    "https://resources.download.minecraft.net";
