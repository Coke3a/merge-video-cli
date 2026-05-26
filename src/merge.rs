use anyhow::{Context, Result};
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::{NamedTempFile, TempDir};

use crate::ffmpeg;

fn write_concat_list(files: &[PathBuf]) -> Result<NamedTempFile> {
    let mut tmp = NamedTempFile::new().context("Failed to create temp concat list file")?;
    for file in files {
        let path_str = file.to_string_lossy();
        let escaped = path_str.replace('\'', "'\\''");
        writeln!(tmp, "file '{}'", escaped)
            .context("Failed to write to concat list")?;
    }
    tmp.flush().context("Failed to flush concat list")?;
    Ok(tmp)
}

fn merge_via_ts_intermediate(files: &[PathBuf], output_path: &Path, output_format: &str) -> Result<()> {
    let tmp_dir = TempDir::new().context("Failed to create temp directory for TS remux")?;

    eprintln!("Remuxing {} files to TS intermediate format...", files.len());
    let mut ts_files: Vec<PathBuf> = Vec::new();
    for (i, file) in files.iter().enumerate() {
        let ts_name = format!("{:06}.ts", i);
        let ts_path = tmp_dir.path().join(&ts_name);
        let args = ffmpeg::build_remux_to_ts_args(file, &ts_path);
        ffmpeg::run_ffmpeg(args).map_err(|err| {
            anyhow::anyhow!(
                "Failed to remux '{}' to TS: {}",
                file.file_name().unwrap_or_default().to_string_lossy(),
                err
            )
        })?;
        ts_files.push(ts_path);
        if (i + 1) % 25 == 0 || i + 1 == files.len() {
            eprintln!("  [{}/{}] remuxed", i + 1, files.len());
        }
    }

    eprintln!("Concatenating TS files...");
    let concat_list = write_concat_list(&ts_files)?;
    let args = ffmpeg::build_ts_concat_to_output_args(concat_list.path(), output_path, output_format);
    ffmpeg::run_ffmpeg(args).map_err(|err| {
        anyhow::anyhow!("Failed to concatenate TS files: {}", err)
    })?;

    Ok(())
}

pub fn merge_videos(files: &[PathBuf], output_path: &Path) -> Result<()> {
    let ext = output_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("ts")
        .to_ascii_lowercase();

    // FLV concat demuxer with -c copy has timestamp issues;
    // use TS as intermediate format instead
    if ext == "flv" {
        return merge_via_ts_intermediate(files, output_path, "flv");
    }

    let concat_list = write_concat_list(files)?;

    let copy_args = ffmpeg::build_concat_copy_args(concat_list.path(), output_path);
    match ffmpeg::run_ffmpeg(copy_args) {
        Ok(()) => return Ok(()),
        Err(err) => {
            eprintln!(
                "Stream copy failed ({}), falling back to re-encode...",
                err
            );
            if output_path.exists() {
                let _ = std::fs::remove_file(output_path);
            }
        }
    }

    let encode_args = ffmpeg::build_concat_encode_args(concat_list.path(), output_path, &ext);
    ffmpeg::run_ffmpeg(encode_args).map_err(|err| {
        anyhow::anyhow!(
            "Merge failed — stream copy failed and re-encode also failed: {}",
            err
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn write_concat_list_produces_correct_format() {
        let files = vec![
            PathBuf::from("/input/001.ts"),
            PathBuf::from("/input/002.ts"),
            PathBuf::from("/input/003.ts"),
        ];
        let tmp = write_concat_list(&files).unwrap();

        let mut content = String::new();
        std::fs::File::open(tmp.path()).unwrap().read_to_string(&mut content).unwrap();

        assert_eq!(
            content,
            "file '/input/001.ts'\nfile '/input/002.ts'\nfile '/input/003.ts'\n"
        );
    }

    #[test]
    fn write_concat_list_escapes_single_quotes() {
        let files = vec![PathBuf::from("/input/it's a file.ts")];
        let tmp = write_concat_list(&files).unwrap();

        let mut content = String::new();
        std::fs::File::open(tmp.path()).unwrap().read_to_string(&mut content).unwrap();

        assert_eq!(content, "file '/input/it'\\''s a file.ts'\n");
    }
}
