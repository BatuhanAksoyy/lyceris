use serde::{de::DeserializeOwned, Serialize};
use std::path::Path;
use tokio::{
    fs::{create_dir_all, File},
    io::{AsyncReadExt, AsyncWriteExt},
};

/// Reads a JSON file from the specified path and deserializes it into the specified type.
///
/// # Parameters
/// - `path`: The path to the JSON file to read.
///
/// # Returns
/// A result containing the deserialized data on success, or an error if the file could not be read or parsed.
pub async fn read_json<T: DeserializeOwned>(path: &Path) -> crate::Result<T> {
    let mut file = File::open(path).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    Ok(serde_json::from_str(&contents)?)
}

/// Writes the specified data to a JSON file at the given path.
///
/// # Parameters
/// - `path`: The path where the JSON file should be written.
/// - `data`: The data to serialize and write to the file.
///
/// # Returns
/// A result indicating success or failure of the write operation.
pub async fn write_json<T: Serialize>(path: &Path, data: &T) -> crate::Result<()> {
    let json_string = serde_json::to_string(data)?;
    if let Some(parent) = path.parent() {
        if !parent.is_dir() {
            create_dir_all(parent).await?;
        }
    }
    let mut file = File::create(path).await?;
    file.write_all(json_string.as_bytes()).await?;
    Ok(())
}
