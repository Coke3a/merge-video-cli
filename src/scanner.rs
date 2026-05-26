use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};

const VIDEO_EXTENSIONS: &[&str] = &["flv", "ts"];

pub fn scan_video_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let entries = std::fs::read_dir(dir)
        .with_context(|| format!("Failed to read directory: {}", dir.display()))?;

    let mut files: Vec<PathBuf> = Vec::new();
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
            let abs = path.canonicalize()
                .with_context(|| format!("Failed to resolve path: {}", path.display()))?;
            files.push(abs);
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

    #[test]
    fn returns_sorted_ts_files() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("003.ts"), b"fake").unwrap();
        fs::write(tmp.path().join("001.ts"), b"fake").unwrap();
        fs::write(tmp.path().join("002.ts"), b"fake").unwrap();

        let files = scan_video_files(tmp.path()).unwrap();
        let names: Vec<&str> = files.iter().map(|p| p.file_name().unwrap().to_str().unwrap()).collect();
        assert_eq!(names, vec!["001.ts", "002.ts", "003.ts"]);
    }

    #[test]
    fn returns_sorted_flv_files() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("b.flv"), b"fake").unwrap();
        fs::write(tmp.path().join("a.flv"), b"fake").unwrap();

        let files = scan_video_files(tmp.path()).unwrap();
        let names: Vec<&str> = files.iter().map(|p| p.file_name().unwrap().to_str().unwrap()).collect();
        assert_eq!(names, vec!["a.flv", "b.flv"]);
    }

    #[test]
    fn filters_out_non_video_files() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("video.ts"), b"fake").unwrap();
        fs::write(tmp.path().join("readme.txt"), b"text").unwrap();
        fs::write(tmp.path().join("image.png"), b"img").unwrap();

        let files = scan_video_files(tmp.path()).unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].file_name().unwrap().to_str().unwrap() == "video.ts");
    }

    #[test]
    fn case_insensitive_extension() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("a.TS"), b"fake").unwrap();
        fs::write(tmp.path().join("b.FLV"), b"fake").unwrap();

        let files = scan_video_files(tmp.path()).unwrap();
        assert_eq!(files.len(), 2);
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
        fs::write(tmp.path().join("a.ts"), b"fake").unwrap();

        let files = scan_video_files(tmp.path()).unwrap();
        assert!(files[0].is_absolute());
    }

    #[test]
    fn ignores_subdirectories() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("a.ts"), b"fake").unwrap();
        fs::create_dir(tmp.path().join("subdir.ts")).unwrap();

        let files = scan_video_files(tmp.path()).unwrap();
        assert_eq!(files.len(), 1);
    }
}
