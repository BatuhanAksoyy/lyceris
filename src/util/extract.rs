use std::{fs::{create_dir_all, File}, io::Read, path::{Path, PathBuf}};
use zip::read::ZipArchive;

pub fn extract_file<P: AsRef<Path>>(zip_path: &P, output_dir: &P) -> crate::Result<()> {
    let file = File::open(zip_path)?;

    create_dir_all(output_dir)?;

    let mut archive = ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let file_path = file.mangled_name();

        if file.is_dir() {
            let directory_path = &output_dir.as_ref().join(file_path);
            std::fs::create_dir_all(directory_path)?;
        } else {
            let mut file_buffer = File::create(output_dir.as_ref().join(file_path))?;
            std::io::copy(&mut file, &mut file_buffer)?;
        }
    }

    Ok(())
}

pub fn extract_specific_file<P: AsRef<Path>>(
    zip_path: &P,
    file_name: &str,
    output_file: &P,
) -> crate::Result<()> {
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    if let Some(parent) = &output_file.as_ref().parent() {
        create_dir_all(parent)?;
    }

    let mut file_found = false;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        if file.name() == file_name {
            file_found = true;

            let mut file_buffer = File::create(output_file)?;
            std::io::copy(&mut file, &mut file_buffer)?;
            break;
        }
    }

    if !file_found {
        return Err(crate::Error::NotFound(format!(
            "File '{}' in the ZIP archive",
            file_name
        )));
    }

    Ok(())
}
pub fn extract_specific_directory<P: AsRef<Path>>(
    zip_path: &P,
    dir_name: &str,
    output_dir: &P,
) -> crate::Result<()> {
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    create_dir_all(output_dir)?;

    let normalized_dir = dir_name.trim_start_matches('/');

    let mut dir_found = false;
    for i in 0..archive.len() {
        let mut zip_file = archive.by_index(i)?;
        let normalized_name = zip_file.name().trim_start_matches('/');

        if normalized_name == normalized_dir || normalized_name.starts_with(&format!("{}/", normalized_dir)) {
            dir_found = true;

            let relative_path = if normalized_name == normalized_dir {
                PathBuf::new()
            } else {
                Path::new(normalized_name).strip_prefix(normalized_dir)?.to_path_buf()
            };

            let output_path = if relative_path.as_os_str().is_empty() {
                output_dir.as_ref().to_path_buf()
            } else {
                output_dir.as_ref().join(relative_path)
            };

            if zip_file.is_dir() {
                create_dir_all(&output_path)?;
            } else {
                if let Some(parent) = output_path.parent() {
                    create_dir_all(parent)?;
                }
                let mut outfile = File::create(&output_path)?;
                std::io::copy(&mut zip_file, &mut outfile)?;
            }
        }
    }

    if !dir_found {
        return Err(crate::Error::NotFound(format!(
            "Directory '{}' in the ZIP archive",
            dir_name
        )));
    }

    Ok(())
}

pub fn read_file_from_jar<P: AsRef<Path>>(
    zip_path: &P,
    file_name: &str,
) -> crate::Result<String> {
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        if file.name() == file_name {
            let mut buffer = String::new();
            file.read_to_string(&mut buffer)?;
            return Ok(buffer);
        }
    }

    Err(crate::Error::NotFound(format!(
        "File '{}' in the ZIP archive",
        file_name
    )))
}