use std::path::Path;

use zip::ZipArchive;

use crate::error::{Error, ErrorContext};

pub fn copy_recursive(src: &Path, dst: &Path) -> Result<(), ErrorContext> {
    if src.is_dir() {
        let entries = ignore::WalkBuilder::new(src).ignore(false).build().flatten();

        for entry in entries {
            if entry.path().is_dir() {
                continue;
            }

            let relative_path = entry.path().strip_prefix(src).unwrap();
            let dst_path = dst.join(relative_path);

            let parent_path = dst_path.parent().ok_or_else(|| {
                ErrorContext::builder()
                    .error(Error::file_system(
                        "Failed to get parent directory".to_string(),
                        "Create Directory",
                    ))
                    .suggested_action(
                        "Check if the parent directory is readable or permissions are set correctly.",
                    )
                    .build()
            })?;

            std::fs::create_dir_all(parent_path).map_err(|e| {
                ErrorContext::builder()
                    .error(Error::file_system(e.to_string(), "Create Directory"))
                    .suggested_action(
                        "Check if the mod storage directory is writable or permissions are set correctly.",
                    )
                    .build()
            })?;

            std::fs::copy(entry.path(), &dst_path).map_err(|e| {
                ErrorContext::builder()
                    .error(Error::file_system(e.to_string(), "Copy File"))
                    .suggested_action(
                        "Check if the mod storage directory is writable or permissions are set correctly.",
                    )
                    .build()
            })?;
        }
    } else {
        std::fs::copy(src, dst).map_err(|e| {
            ErrorContext::builder()
                .error(Error::file_system(e.to_string(), "Copy File"))
                .suggested_action(
                    "Check if the mod storage directory is writable or permissions are set correctly.",
                )
                .build()
        })?;
    }
    Ok(())
}

pub fn extract_zip(zip_path: &Path, dst_path: &Path) -> Result<(), ErrorContext> {
    let handle = std::fs::File::open(zip_path).map_err(|e| {
        ErrorContext::builder()
            .error(Error::file_system(e.to_string(), "Extract Zip"))
            .suggested_action("Check if the file is readable or permissions are set correctly.")
            .build()
    })?;

    let mut archive = ZipArchive::new(handle).map_err(|e| {
        ErrorContext::builder()
            .error(Error::file_system(e.to_string(), "Extract Zip"))
            .suggested_action("Check if the zip file is valid and not corrupted.")
            .build()
    })?;

    tracing::trace!("Extracting {} files", archive.len());
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| {
            ErrorContext::builder()
                .error(Error::file_system(e.to_string(), "Extract Zip"))
                .suggested_action("Check if the zip file is valid and not corrupted.")
                .build()
        })?;

        let output_path = match file.enclosed_name() {
            Some(path) => dst_path.join(path),
            None => continue,
        };

        if file.is_dir() {
            std::fs::create_dir_all(&output_path).map_err(|e| {
                ErrorContext::builder()
                    .error(Error::file_system(e.to_string(), "Extract Zip"))
                    .suggested_action(
                        "Check if the mod storage directory is writable or permissions are set correctly.",
                    )
                    .build()
            })?;
        } else {
            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    ErrorContext::builder()
                    .error(Error::file_system(e.to_string(), "Extract Zip"))
                    .suggested_action(
                        "Check if the mod storage directory is writable or permissions are set correctly.",
                    )
                    .build()
                })?;
            }

            let mut output_file = std::fs::File::create(&output_path).map_err(|e| {
                ErrorContext::builder()
                    .error(Error::file_system(e.to_string(), "Extract Zip"))
                    .suggested_action(
                        "Check if the mod storage directory is writable or permissions are set correctly.",
                    )
                    .build()
            })?;

            std::io::copy(&mut file, &mut output_file).map_err(|e| {
                ErrorContext::builder().error(Error::file_system(e.to_string(), "Extract Zip")).build()
            })?;
        }
    }
    Ok(())
}
