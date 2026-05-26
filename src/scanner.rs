use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};

const VIDEO_EXTENSIONS: &[&str] = &["flv", "ts"];
const MIN_FILE_SIZE: u64 = 1024;

pub fn scan_video_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let entries = std::fs::read_dir(dir)
        .with_context(|| format!("Failed to read directory: {}", dir.display()))?;

    let mut files: Vec<PathBuf> = Vec::new();
    let mut skipped: Vec<String> = Vec::new();
    for entry in entries {
        let entry = entry.with_context(|| format!("Failed to read entry in {}", dir.display()))?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = match path.extension().and_then(|e| e.to_str()) {
            Some(e) => e.to_ascii_lowercase(),
            None => continue,
        };
        if VIDEO_EXTENSIONS.contains(&ext.as_str()) {
            let metadata = path.metadata()
                .with_context(|| format!("Failed to read metadata: {}", path.display()))?;
            if metadata.len() < MIN_FILE_SIZE {
                skipped.push(
                    path.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into_owned(),
                );
                continue;
            }
            let abs = path.canonicalize()
                .with_context(|| format!("Failed to resolve path: {}", path.display()))?;
            files.push(abs);
        }
    }

    if !skipped.is_empty() {
        eprintln!(
            "Warning: skipped {} file(s) smaller than {} bytes (likely empty/corrupt):",
            skipped.len(),
            MIN_FILE_SIZE
        );
        for name in &skipped {
            eprintln!("  - {}", name);
        }
    }

    if files.is_empty() {
        bail!("No .flv or .ts files found in '{}'", dir.display());
    }

    files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn dummy_video_data() -> Vec<u8> {
        vec![0u8; 2048]
    }

    #[test]
    fn returns_sorted_ts_files() {
        let tmp = TempDir::new().unwrap();
        let data = dummy_video_data();
        fs::write(tmp.path().join("003.ts"), &data).unwrap();
        fs::write(tmp.path().join("001.ts"), &data).unwrap();
        fs::write(tmp.path().join("002.ts"), &data).unwrap();

        let files = scan_video_files(tmp.path()).unwrap();
        let names: Vec<&str> = files.iter().map(|p| p.file_name().unwrap().to_str().unwrap()).collect();
        assert_eq!(names, vec!["001.ts", "002.ts", "003.ts"]);
    }

    #[test]
    fn returns_sorted_flv_files() {
        let tmp = TempDir::new().unwrap();
        let data = dummy_video_data();
        fs::write(tmp.path().join("b.flv"), &data).unwrap();
        fs::write(tmp.path().join("a.flv"), &data).unwrap();

        let files = scan_video_files(tmp.path()).unwrap();
        let names: Vec<&str> = files.iter().map(|p| p.file_name().unwrap().to_str().unwrap()).collect();
        assert_eq!(names, vec!["a.flv", "b.flv"]);
    }

    #[test]
    fn filters_out_non_video_files() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("video.ts"), &dummy_video_data()).unwrap();
        fs::write(tmp.path().join("readme.txt"), b"text").unwrap();
        fs::write(tmp.path().join("image.png"), b"img").unwrap();

        let files = scan_video_files(tmp.path()).unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].file_name().unwrap().to_str().unwrap() == "video.ts");
    }

    #[test]
    fn case_insensitive_extension() {
        let tmp = TempDir::new().unwrap();
        let data = dummy_video_data();
        fs::write(tmp.path().join("a.TS"), &data).unwrap();
        fs::write(tmp.path().join("b.FLV"), &data).unwrap();

        let files = scan_video_files(tmp.path()).unwrap();
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn skips_files_smaller_than_min_size() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("good.flv"), &dummy_video_data()).unwrap();
        fs::write(tmp.path().join("empty.flv"), b"FLV header only").unwrap();

        let files = scan_video_files(tmp.path()).unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].file_name().unwrap().to_str().unwrap() == "good.flv");
    }

    #[test]
    fn error_on_empty_directory() {
        let tmp = TempDir::new().unwrap();
        let result = scan_video_files(tmp.path());
        assert!(result.is_err());
    }

    #[test]
    fn error_on_nonexistent_directory() {
        let result = scan_video_files(Path::new("/nonexistent/dir/xyz"));
        assert!(result.is_err());
    }

    #[test]
    fn returns_absolute_paths() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("a.ts"), &dummy_video_data()).unwrap();

        let files = scan_video_files(tmp.path()).unwrap();
        assert!(files[0].is_absolute());
    }

    #[test]
    fn ignores_subdirectories() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("a.ts"), &dummy_video_data()).unwrap();
        fs::create_dir(tmp.path().join("subdir.ts")).unwrap();

        let files = scan_video_files(tmp.path()).unwrap();
        assert_eq!(files.len(), 1);
    }
}
