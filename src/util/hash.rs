use sha1::{Digest, Sha1};
use std::{fs::File, io::Read, path::Path};

/// Calculates the SHA-1 hash of a file at the specified path.
///
/// # Parameters
/// - `path`: The path to the file for which to calculate the SHA-1 hash.
///
/// # Returns
/// A result containing the SHA-1 hash as a hexadecimal string or an error if the file could not be read.
pub fn calculate_sha1<P: AsRef<Path>>(path: P) -> crate::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha1::new();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    hasher.update(&buffer);
    Ok(format!("{:x}", hasher.finalize()))
}
