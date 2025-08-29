use std::io::Write;
use std::path::Path;

use zip::ZipArchive;

pub fn copy_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if src.is_dir() {
        let entries = ignore::WalkBuilder::new(src).ignore(false).build().flatten();

        for entry in entries {
            if entry.path().is_dir() {
                continue;
            }

            let relative_path = entry.path().strip_prefix(src).unwrap();
            let dst_path = dst.join(relative_path);
            std::fs::create_dir_all(
                dst_path.parent().ok_or(std::io::Error::other("Failed to create directory"))?,
            )?;
            std::fs::File::create(&dst_path)?.write_all(&std::fs::read(entry.path())?)?;
        }
    } else {
        std::fs::copy(src, dst)?;
    }
    Ok(())
}

pub fn extract_zip(zip_path: &Path, dst_path: &Path) -> Result<(), crate::error::Error> {
    tracing::trace!("Extracting zip file {} to {}", zip_path.display(), dst_path.display());
    let handle = std::fs::File::open(zip_path)
        .map_err(|e| crate::error::Error::file_system(e.to_string(), "Open".to_string()))?;

    let mut archive = ZipArchive::new(handle)
        .map_err(|e| crate::error::Error::external(e.to_string(), "Extract Zip".to_string()))?;

    tracing::trace!("Processing {} files", archive.len());
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| crate::error::Error::external(e.to_string(), "Extract Zip".to_string()))?;
        let output_path = match file.enclosed_name() {
            Some(path) => dst_path.join(path),
            None => continue,
        };
        if file.is_dir() {
            std::fs::create_dir_all(&output_path)?;
        } else {
            let mut output_file = std::fs::File::create(&output_path)?;
            std::io::copy(&mut file, &mut output_file)?;
        }
    }
    Ok(())
}
