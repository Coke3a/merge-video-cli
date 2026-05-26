use std::ffi::OsString;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};

const STDERR_TAIL_BYTES: usize = 8 * 1024;

#[derive(Debug)]
pub struct FfmpegError {
    pub stage: &'static str,
    pub status: Option<ExitStatus>,
    pub stderr: String,
}

impl std::fmt::Display for FfmpegError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.status {
            Some(status) => write!(f, "ffmpeg {} failed ({}): {}", self.stage, status, self.stderr),
            None => write!(f, "ffmpeg {} failed: {}", self.stage, self.stderr),
        }
    }
}

impl std::error::Error for FfmpegError {}

pub fn check_ffmpeg_available() -> anyhow::Result<()> {
    let output = std::process::Command::new("ffmpeg")
        .arg("-version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    match output {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(anyhow::anyhow!(
            "ffmpeg exited with status {}. Please install ffmpeg (https://ffmpeg.org/download.html)",
            status
        )),
        Err(_) => Err(anyhow::anyhow!(
            "ffmpeg not found. Please install ffmpeg (https://ffmpeg.org/download.html)"
        )),
    }
}

pub fn run_ffmpeg(args: Vec<OsString>) -> Result<(), FfmpegError> {
    let mut child = Command::new("ffmpeg")
        .args(&args)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| FfmpegError {
            stage: "spawn",
            status: None,
            stderr: err.to_string(),
        })?;

    let stderr_bytes = child
        .stderr
        .take()
        .map(|mut pipe| {
            let mut buf = Vec::new();
            std::io::Read::read_to_end(&mut pipe, &mut buf).ok();
            buf
        })
        .unwrap_or_default();

    let status = child.wait().map_err(|err| FfmpegError {
        stage: "wait",
        status: None,
        stderr: err.to_string(),
    })?;

    if !status.success() {
        let start = stderr_bytes.len().saturating_sub(STDERR_TAIL_BYTES);
        let tail = String::from_utf8_lossy(&stderr_bytes[start..]).trim().to_string();
        return Err(FfmpegError {
            stage: "exit",
            status: Some(status),
            stderr: tail,
        });
    }

    Ok(())
}

pub fn build_concat_copy_args(concat_list: &Path, output: &Path) -> Vec<OsString> {
    vec![
        "-hide_banner".into(),
        "-nostdin".into(),
        "-y".into(),
        "-f".into(),
        "concat".into(),
        "-safe".into(),
        "0".into(),
        "-i".into(),
        concat_list.as_os_str().to_os_string(),
        "-c".into(),
        "copy".into(),
        output.as_os_str().to_os_string(),
    ]
}

pub fn build_concat_encode_args(concat_list: &Path, output: &Path, ext: &str) -> Vec<OsString> {
    let format = match ext {
        "flv" => "flv",
        _ => "mpegts",
    };

    vec![
        "-hide_banner".into(),
        "-nostdin".into(),
        "-y".into(),
        "-f".into(),
        format.into(),
        "-f".into(),
        "concat".into(),
        "-safe".into(),
        "0".into(),
        "-i".into(),
        concat_list.as_os_str().to_os_string(),
        "-c:v".into(),
        "libx264".into(),
        "-preset".into(),
        "veryfast".into(),
        "-crf".into(),
        "23".into(),
        "-pix_fmt".into(),
        "yuv420p".into(),
        "-c:a".into(),
        "aac".into(),
        "-b:a".into(),
        "160k".into(),
        output.as_os_str().to_os_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn args_to_strings(args: &[OsString]) -> Vec<String> {
        args.iter().map(|a| a.to_string_lossy().into_owned()).collect()
    }

    #[test]
    fn concat_copy_args_contains_required_flags() {
        let list = PathBuf::from("/tmp/list.txt");
        let output = PathBuf::from("/out/merged.ts");
        let args = build_concat_copy_args(&list, &output);
        let strs = args_to_strings(&args);

        assert!(strs.contains(&"-hide_banner".to_string()));
        assert!(strs.contains(&"-nostdin".to_string()));
        assert!(strs.contains(&"-y".to_string()));
        assert!(strs.contains(&"concat".to_string()));
        assert!(strs.contains(&"-safe".to_string()));
        assert!(strs.contains(&"0".to_string()));
        assert!(strs.contains(&"copy".to_string()));
        assert_eq!(strs.last().unwrap(), "/out/merged.ts");
    }

    #[test]
    fn concat_copy_args_input_file_position() {
        let list = PathBuf::from("/tmp/list.txt");
        let output = PathBuf::from("/out/merged.ts");
        let args = build_concat_copy_args(&list, &output);
        let strs = args_to_strings(&args);

        let i_pos = strs.iter().position(|s| s == "-i").unwrap();
        assert_eq!(strs[i_pos + 1], "/tmp/list.txt");
    }

    #[test]
    fn concat_encode_args_ts_uses_libx264_aac() {
        let list = PathBuf::from("/tmp/list.txt");
        let output = PathBuf::from("/out/merged.ts");
        let args = build_concat_encode_args(&list, &output, "ts");
        let strs = args_to_strings(&args);

        assert!(strs.contains(&"libx264".to_string()));
        assert!(strs.contains(&"aac".to_string()));
        assert!(strs.contains(&"veryfast".to_string()));
        assert!(strs.contains(&"23".to_string()));
    }

    #[test]
    fn concat_encode_args_flv_forces_flv_format() {
        let list = PathBuf::from("/tmp/list.txt");
        let output = PathBuf::from("/out/merged.flv");
        let args = build_concat_encode_args(&list, &output, "flv");
        let strs = args_to_strings(&args);

        let f_pos = strs.iter().position(|s| s == "-f").unwrap();
        assert_eq!(strs[f_pos + 1], "flv");
    }

    #[test]
    fn concat_encode_args_ts_forces_mpegts_format() {
        let list = PathBuf::from("/tmp/list.txt");
        let output = PathBuf::from("/out/merged.ts");
        let args = build_concat_encode_args(&list, &output, "ts");
        let strs = args_to_strings(&args);

        let f_pos = strs.iter().position(|s| s == "-f").unwrap();
        assert_eq!(strs[f_pos + 1], "mpegts");
    }

    #[test]
    fn ffmpeg_error_display_with_status() {
        let err = FfmpegError {
            stage: "exit",
            status: None,
            stderr: "some error".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("exit"));
        assert!(msg.contains("some error"));
    }
}
