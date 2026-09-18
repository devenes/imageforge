use crate::errors::AppError;
use crate::models::{CompressionSettings, ImageFormat, OutputMode};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Resolves the intended destination path, taking into account output directory,
/// filename suffixes, conversion target format, and collision avoidance.
pub fn resolve_output_path(
    input_path: &Path,
    target_format: ImageFormat,
    settings: &CompressionSettings,
) -> Result<PathBuf, AppError> {
    let parent_dir = if let Some(ref custom_dir) = settings.custom_output_dir {
        PathBuf::from(custom_dir)
    } else {
        input_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
    };

    if !parent_dir.exists() {
        std::fs::create_dir_all(&parent_dir).map_err(|e| AppError::IoError(e.to_string()))?;
    }

    let file_stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("image");

    let ext = target_format.extension();

    let base_name = match settings.output_mode {
        OutputMode::SameFormat => {
            let suffix = settings.filename_suffix.as_deref().unwrap_or("-compressed");
            format!("{file_stem}{suffix}.{ext}")
        }
        OutputMode::ConvertToJpg => {
            // For conversion: default to photo.jpg, but if converting photo.jpg to jpg with same format, add suffix
            if input_path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase())
                == Some("jpg".to_string())
                || input_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.to_lowercase())
                    == Some("jpeg".to_string())
            {
                let suffix = settings.filename_suffix.as_deref().unwrap_or("-compressed");
                format!("{file_stem}{suffix}.{ext}")
            } else {
                format!("{file_stem}.{ext}")
            }
        }
    };

    let target_path = parent_dir.join(&base_name);

    // Collision handling: never overwrite an existing file!
    if !target_path.exists() {
        return Ok(target_path);
    }

    let stem_no_ext = target_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(file_stem);

    let mut counter = 1;
    loop {
        let candidate = parent_dir.join(format!("{stem_no_ext}-{counter}.{ext}"));
        if !candidate.exists() {
            return Ok(candidate);
        }
        counter += 1;
    }
}

/// Atomically writes data to the target path using a temporary file in the target directory
/// followed by an atomic rename.
pub fn atomic_write_file(target_path: &Path, data: &[u8]) -> Result<(), AppError> {
    let parent = target_path
        .parent()
        .ok_or_else(|| AppError::IoError("Target path has no parent directory".into()))?;

    // Create a uniquely named temporary file in the same directory as the target
    // so that std::fs::rename is guaranteed to be an atomic filesystem rename on the same mount.
    let temp_name = format!(
        ".tmp_{}_{}",
        std::process::id(),
        uuid::Uuid::new_v4().simple()
    );
    let temp_path = parent.join(temp_name);

    {
        let mut file = File::create(&temp_path).map_err(|e| AppError::IoError(e.to_string()))?;
        file.write_all(data)
            .map_err(|e| AppError::IoError(e.to_string()))?;
        file.flush().map_err(|e| AppError::IoError(e.to_string()))?;
    }

    // Atomic rename
    if let Err(e) = std::fs::rename(&temp_path, target_path) {
        let _ = std::fs::remove_file(&temp_path);
        return Err(AppError::IoError(format!("Atomic rename failed: {e}")));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_collision_resolution() {
        let dir = tempdir().unwrap();
        let file1 = dir.path().join("photo.png");
        std::fs::write(&file1, b"original").unwrap();

        let settings = CompressionSettings {
            preset: Default::default(),
            output_mode: OutputMode::SameFormat,
            preserve_metadata: true,
            preserve_color_profile: true,
            custom_output_dir: Some(dir.path().to_string_lossy().to_string()),
            filename_suffix: Some("-compressed".to_string()),
        };

        // First resolve: photo-compressed.png
        let out1 = resolve_output_path(&file1, ImageFormat::Png, &settings).unwrap();
        assert_eq!(out1.file_name().unwrap(), "photo-compressed.png");
        std::fs::write(&out1, b"compressed1").unwrap();

        // Second resolve: should avoid collision -> photo-compressed-1.png
        let out2 = resolve_output_path(&file1, ImageFormat::Png, &settings).unwrap();
        assert_eq!(out2.file_name().unwrap(), "photo-compressed-1.png");
    }

    #[test]
    fn test_atomic_write() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("output.jpg");
        atomic_write_file(&target, b"test payload").unwrap();
        assert_eq!(std::fs::read(&target).unwrap(), b"test payload");
    }
}
