# merge-video CLI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Rust CLI tool that scans an input directory for `.flv`/`.ts` video files, shows a sorted list for user confirmation, then merges them into a single output file via ffmpeg's concat demuxer (try stream copy first, fallback to re-encode).

**Architecture:** Four-module sync Rust binary — `main.rs` (CLI + orchestration), `scanner.rs` (directory scan + filter + sort), `ffmpeg.rs` (subprocess runner + arg builders + error type), `merge.rs` (concat list writer + copy/encode orchestration). No async runtime; uses `std::process::Command`.

**Tech Stack:** Rust, clap (derive), anyhow, chrono, tempfile, ffmpeg (external)

---

## File Map

| File | Responsibility |
|------|---------------|
| `Cargo.toml` | Project metadata + dependencies |
| `src/main.rs` | CLI args (clap), user confirmation prompt, orchestration flow |
| `src/scanner.rs` | Scan input dir, filter `.flv`/`.ts`, sort by filename, return `Vec<PathBuf>` |
| `src/ffmpeg.rs` | `FfmpegError` type, `check_ffmpeg_available()`, `run_ffmpeg()`, arg builders |
| `src/merge.rs` | Write concat list to temp file, try copy then encode fallback |

---

### Task 1: Project scaffold + dependencies

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs` (minimal placeholder)

- [ ] **Step 1: Initialize Cargo project**

```bash
cd /Users/coke/Projects/merge_video
cargo init --name merge_video
```

- [ ] **Step 2: Add dependencies to `Cargo.toml`**

Replace the `[dependencies]` section in `Cargo.toml` with:

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
anyhow = "1"
chrono = "0.4"
tempfile = "3"
```

- [ ] **Step 3: Verify it compiles**

```bash
cd /Users/coke/Projects/merge_video && cargo build
```

Expected: compiles successfully.

- [ ] **Step 4: Create module files**

Create empty module files so later tasks can work independently:

`src/scanner.rs`:
```rust
use anyhow::Result;
use std::path::{Path, PathBuf};

pub fn scan_video_files(_dir: &Path) -> Result<Vec<PathBuf>> {
    todo!()
}
```

`src/ffmpeg.rs`:
```rust
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
```

`src/merge.rs`:
```rust
use anyhow::Result;
use std::path::{Path, PathBuf};

pub fn merge_videos(_files: &[PathBuf], _output_path: &Path) -> Result<()> {
    todo!()
}
```

Update `src/main.rs` to declare modules:

```rust
mod ffmpeg;
mod merge;
mod scanner;

fn main() {
    println!("merge-video placeholder");
}
```

- [ ] **Step 5: Verify it compiles**

```bash
cd /Users/coke/Projects/merge_video && cargo check
```

Expected: compiles (with dead_code warnings, which is fine).

- [ ] **Step 6: Commit**

```bash
cd /Users/coke/Projects/merge_video
git add Cargo.toml Cargo.lock src/
git commit -m "feat: scaffold project with module stubs and dependencies"
```

---

### Task 2: `scanner.rs` — test + implement scan_video_files

**Files:**
- Modify: `src/scanner.rs`

- [ ] **Step 1: Write failing tests**

Replace `src/scanner.rs` with the full test module and the public function signature:

```rust
use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};

const VIDEO_EXTENSIONS: &[&str] = &["flv", "ts"];

pub fn scan_video_files(dir: &Path) -> Result<Vec<PathBuf>> {
    todo!()
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
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cd /Users/coke/Projects/merge_video && cargo test --lib scanner
```

Expected: all tests fail with "not yet implemented".

- [ ] **Step 3: Implement `scan_video_files`**

Replace the `todo!()` body of `scan_video_files` in `src/scanner.rs`:

```rust
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
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cd /Users/coke/Projects/merge_video && cargo test --lib scanner
```

Expected: all 8 tests pass.

- [ ] **Step 5: Commit**

```bash
cd /Users/coke/Projects/merge_video
git add src/scanner.rs
git commit -m "feat: implement scanner with tests — scan, filter, sort video files"
```

---

### Task 3: `ffmpeg.rs` — test + implement arg builders

**Files:**
- Modify: `src/ffmpeg.rs`

- [ ] **Step 1: Write failing tests for arg builders**

Add these tests at the bottom of `src/ffmpeg.rs`:

```rust
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
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cd /Users/coke/Projects/merge_video && cargo test --lib ffmpeg
```

Expected: all tests fail with "not yet implemented".

- [ ] **Step 3: Implement arg builders**

Replace the `todo!()` bodies of `build_concat_copy_args` and `build_concat_encode_args` in `src/ffmpeg.rs`:

```rust
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
        "-f".into(),
        format.into(),
        output.as_os_str().to_os_string(),
    ]
}
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cd /Users/coke/Projects/merge_video && cargo test --lib ffmpeg
```

Expected: all 6 tests pass.

- [ ] **Step 5: Commit**

```bash
cd /Users/coke/Projects/merge_video
git add src/ffmpeg.rs
git commit -m "feat: implement ffmpeg arg builders with tests"
```

---

### Task 4: `ffmpeg.rs` — implement `check_ffmpeg_available` and `run_ffmpeg`

**Files:**
- Modify: `src/ffmpeg.rs`

- [ ] **Step 1: Implement `check_ffmpeg_available`**

Replace the `todo!()` body in `src/ffmpeg.rs`:

```rust
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
```

- [ ] **Step 2: Implement `run_ffmpeg`**

Replace the `todo!()` body in `src/ffmpeg.rs`. Add `use std::process::{Command, Stdio};` to the top imports:

```rust
use std::ffi::OsString;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};

const STDERR_TAIL_BYTES: usize = 8 * 1024;

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
```

- [ ] **Step 3: Verify compilation**

```bash
cd /Users/coke/Projects/merge_video && cargo check
```

Expected: compiles. (No unit tests for these — they require ffmpeg and are covered by integration.)

- [ ] **Step 4: Run all existing tests still pass**

```bash
cd /Users/coke/Projects/merge_video && cargo test
```

Expected: all tests from Tasks 2 and 3 still pass.

- [ ] **Step 5: Commit**

```bash
cd /Users/coke/Projects/merge_video
git add src/ffmpeg.rs
git commit -m "feat: implement check_ffmpeg_available and run_ffmpeg subprocess runner"
```

---

### Task 5: `merge.rs` — implement merge_videos

**Files:**
- Modify: `src/merge.rs`

- [ ] **Step 1: Write test for concat list generation**

Replace `src/merge.rs` with the full implementation and a unit test for the concat list writer:

```rust
use anyhow::{bail, Context, Result};
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

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

pub fn merge_videos(files: &[PathBuf], output_path: &Path) -> Result<()> {
    todo!()
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
```

- [ ] **Step 2: Run tests to verify the concat list tests pass**

```bash
cd /Users/coke/Projects/merge_video && cargo test --lib merge
```

Expected: 2 tests pass, no compilation error (the `todo!()` in `merge_videos` is not called).

- [ ] **Step 3: Implement `merge_videos`**

Replace the `todo!()` body of `merge_videos`:

```rust
pub fn merge_videos(files: &[PathBuf], output_path: &Path) -> Result<()> {
    let concat_list = write_concat_list(files)?;

    let ext = output_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("ts");

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

    let encode_args = ffmpeg::build_concat_encode_args(concat_list.path(), output_path, ext);
    ffmpeg::run_ffmpeg(encode_args).map_err(|err| {
        anyhow::anyhow!(
            "Merge failed — stream copy failed and re-encode also failed: {}",
            err
        )
    })
}
```

- [ ] **Step 4: Verify compilation**

```bash
cd /Users/coke/Projects/merge_video && cargo check
```

Expected: compiles.

- [ ] **Step 5: Run all tests**

```bash
cd /Users/coke/Projects/merge_video && cargo test
```

Expected: all tests pass.

- [ ] **Step 6: Commit**

```bash
cd /Users/coke/Projects/merge_video
git add src/merge.rs
git commit -m "feat: implement merge_videos with concat list writer and copy/encode fallback"
```

---

### Task 6: `main.rs` — CLI entry point with clap, confirmation, and output

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Implement full `main.rs`**

Replace `src/main.rs`:

```rust
mod ffmpeg;
mod merge;
mod scanner;

use anyhow::{Context, Result};
use chrono::Local;
use clap::Parser;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "merge-video", about = "Merge video files from a directory into one")]
struct Cli {
    #[arg(short, long, default_value = "./input")]
    input: PathBuf,

    #[arg(short, long, default_value = "./output")]
    output: PathBuf,

    #[arg(short, long, help = "Skip confirmation prompt")]
    yes: bool,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {:#}", err);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    ffmpeg::check_ffmpeg_available()?;

    let files = scanner::scan_video_files(&cli.input)?;

    let ext = files[0]
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("ts")
        .to_ascii_lowercase();

    println!("Found {} .{} files:", files.len(), ext);
    for (i, file) in files.iter().enumerate() {
        let name = file.file_name().unwrap_or_default().to_string_lossy();
        println!("  {}. {}", i + 1, name);
    }

    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let output_filename = format!("merged_{}.{}", timestamp, ext);
    let output_path = cli.output.join(&output_filename);

    println!();
    println!("Output: {}", output_path.display());

    if !cli.yes {
        print!("Proceed? [y/N] ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let trimmed = input.trim().to_ascii_lowercase();
        if trimmed != "y" && trimmed != "yes" {
            println!("Aborted.");
            return Ok(());
        }
    }

    std::fs::create_dir_all(&cli.output)
        .with_context(|| format!("Failed to create output directory: {}", cli.output.display()))?;

    merge::merge_videos(&files, &output_path)?;

    let size = std::fs::metadata(&output_path)
        .map(|m| m.len())
        .unwrap_or(0);
    let size_display = format_size(size);

    println!(
        "Merged {} files → {} ({})",
        files.len(),
        output_path.display(),
        size_display
    );

    Ok(())
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
```

- [ ] **Step 2: Verify compilation**

```bash
cd /Users/coke/Projects/merge_video && cargo build
```

Expected: compiles and links.

- [ ] **Step 3: Run all tests**

```bash
cd /Users/coke/Projects/merge_video && cargo test
```

Expected: all tests pass.

- [ ] **Step 4: Smoke test — run with `--help`**

```bash
cd /Users/coke/Projects/merge_video && cargo run -- --help
```

Expected: prints usage with `--input`, `--output`, `--yes` flags.

- [ ] **Step 5: Smoke test — run with empty input (expect clear error)**

```bash
cd /Users/coke/Projects/merge_video && mkdir -p input && cargo run
```

Expected: prints `Error: No .flv or .ts files found in './input'` and exits with code 1.

- [ ] **Step 6: Commit**

```bash
cd /Users/coke/Projects/merge_video
git add src/main.rs
git commit -m "feat: implement CLI entry point with clap, confirmation prompt, and output"
```

---

### Task 7: Create `input/` and `output/` directories + `.gitkeep`

**Files:**
- Create: `input/.gitkeep`
- Create: `output/.gitkeep`
- Create: `.gitignore`

- [ ] **Step 1: Create directories and gitkeep files**

```bash
cd /Users/coke/Projects/merge_video
mkdir -p input output
touch input/.gitkeep output/.gitkeep
```

- [ ] **Step 2: Create `.gitignore`**

```gitignore
/target
input/*
!input/.gitkeep
output/*
!output/.gitkeep
```

- [ ] **Step 3: Commit**

```bash
cd /Users/coke/Projects/merge_video
git add .gitignore input/.gitkeep output/.gitkeep
git commit -m "chore: add input/output directories and gitignore"
```

---

### Task 8: End-to-end test with real ffmpeg

**Files:**
- None modified — manual verification

- [ ] **Step 1: Verify ffmpeg is available**

```bash
ffmpeg -version | head -1
```

Expected: prints ffmpeg version.

- [ ] **Step 2: Generate two small test .ts files**

```bash
cd /Users/coke/Projects/merge_video
ffmpeg -y -f lavfi -i testsrc=duration=2:size=320x240:rate=15 -f lavfi -i sine=frequency=440:duration=2 -c:v libx264 -preset ultrafast -c:a aac -shortest input/001_test.ts
ffmpeg -y -f lavfi -i testsrc=duration=2:size=320x240:rate=15 -f lavfi -i sine=frequency=880:duration=2 -c:v libx264 -preset ultrafast -c:a aac -shortest input/002_test.ts
```

Expected: two small `.ts` files in `input/`.

- [ ] **Step 3: Run merge-video with `--yes`**

```bash
cd /Users/coke/Projects/merge_video && cargo run -- --yes
```

Expected: prints file list, merges without prompting, outputs something like:
```
Found 2 .ts files:
  1. 001_test.ts
  2. 002_test.ts

Output: ./output/merged_20260526_XXXXXX.ts
Merged 2 files → ./output/merged_20260526_XXXXXX.ts (X.X KB)
```

- [ ] **Step 4: Verify output plays**

```bash
ffprobe /Users/coke/Projects/merge_video/output/merged_*.ts 2>&1 | grep Duration
```

Expected: shows duration ~4 seconds (2+2).

- [ ] **Step 5: Clean up test files**

```bash
rm -f /Users/coke/Projects/merge_video/input/001_test.ts /Users/coke/Projects/merge_video/input/002_test.ts /Users/coke/Projects/merge_video/output/merged_*.ts
```

- [ ] **Step 6: Final commit if any fixes were needed**

Only commit if earlier tasks needed fixes found during this end-to-end test.
