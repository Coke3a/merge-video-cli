use std::ffi::OsString;
use std::path::Path;
use std::process::ExitStatus;

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
    todo!()
}

pub fn run_ffmpeg(_args: Vec<OsString>) -> Result<(), FfmpegError> {
    todo!()
}

pub fn build_concat_copy_args(_concat_list: &Path, _output: &Path) -> Vec<OsString> {
    todo!()
}

pub fn build_concat_encode_args(_concat_list: &Path, _output: &Path, _ext: &str) -> Vec<OsString> {
    todo!()
}
