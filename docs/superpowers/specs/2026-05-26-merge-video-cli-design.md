# merge-video CLI — Design Spec

## Summary

A Rust CLI tool that merges multiple video files from an input directory into a single output file using ffmpeg's concat demuxer. Supports `.flv` and `.ts` formats. Tries stream copy first for speed; falls back to re-encode if copy fails.

## User Flow

1. User places video files (all `.flv` or all `.ts`) into `./input/`
2. User runs `merge-video` (or `cargo run`)
3. App scans input directory, sorts files alphabetically by filename
4. App displays numbered list and asks for confirmation
5. On confirmation, app merges files into `./output/merged_YYYYMMDD_HHMMSS.{ext}`
6. App reports output file path and size

## Constraints

- Input directory contains only one format per run (all `.flv` or all `.ts`, never mixed)
- Output format matches input format (`.ts` in → `.ts` out, `.flv` in → `.flv` out)
- Requires ffmpeg installed on the system
- Synchronous CLI — no async runtime needed
- Output filename is auto-generated with timestamp

## Module Structure

```
merge_video/
├── Cargo.toml
├── input/              ← user puts video files here
├── output/             ← merged result goes here
└── src/
    ├── main.rs         ← CLI entry: parse args, orchestrate flow
    ├── scanner.rs      ← scan directory, filter extensions, sort by name
    ├── ffmpeg.rs       ← build ffmpeg args, run process, capture stderr, error types
    └── merge.rs        ← write concat list → run ffmpeg (copy then encode fallback)
```

## Module Details

### `scanner.rs`

**Public API:**

```rust
pub fn scan_video_files(dir: &Path) -> Result<Vec<PathBuf>>
```

**Behavior:**

- Reads directory entries (non-recursive, single level)
- Filters files whose extension is `.flv` or `.ts` (case-insensitive)
- Sorts alphabetically by filename (`OsStr` ordering)
- Returns error if:
  - Directory does not exist
  - No video files found after filtering

**Returns:** sorted `Vec<PathBuf>` of absolute paths to video files.

### `ffmpeg.rs`

**Structured error type** (inspired by reference project `recording_engine_webhook.rs`):

```rust
pub struct FfmpegError {
    pub stage: &'static str,   // "spawn", "exit", "timeout"
    pub status: Option<ExitStatus>,
    pub stderr: String,
}
```

**Public API:**

```rust
pub fn check_ffmpeg_available() -> Result<()>
// Runs `ffmpeg -version`, returns error if not found.

pub fn run_ffmpeg(args: Vec<OsString>) -> Result<(), FfmpegError>
// Spawns ffmpeg with given args, captures stderr (last 8KB),
// returns structured error on failure.

pub fn build_concat_copy_args(concat_list: &Path, output: &Path) -> Vec<OsString>
// Builds: ffmpeg -hide_banner -nostdin -y -f concat -safe 0 -i <list> -c copy <output>

pub fn build_concat_encode_args(concat_list: &Path, output: &Path, ext: &str) -> Vec<OsString>
// Builds re-encode args. For .ts output: -c:v libx264 -preset veryfast -crf 23 -c:a aac
// For .flv output: same encoding but -f flv container.
```

**Implementation notes:**

- Uses `std::process::Command` (not async — simple CLI tool)
- Captures stderr via `Stdio::piped()`, keeps only last 8KB tail
- No timeout needed for a user-interactive CLI (user can Ctrl+C)

### `merge.rs`

**Public API:**

```rust
pub fn merge_videos(files: &[PathBuf], output_path: &Path) -> Result<()>
```

**Behavior:**

1. Writes ffmpeg concat demuxer list to a temp file (paths are single-quoted and inner single quotes are escaped as `'\''`):
   ```
   file '/absolute/path/to/001.ts'
   file '/absolute/path/to/002.ts'
   ```
2. Attempts `run_ffmpeg(build_concat_copy_args(...))` (stream copy — fast)
3. If step 2 fails:
   - Prints warning: "Stream copy failed, falling back to re-encode..."
   - Removes partial output file if it exists
   - Attempts `run_ffmpeg(build_concat_encode_args(...))`
4. Cleans up temp concat list file (via `tempfile` crate auto-cleanup)
5. Returns error if both attempts fail

### `main.rs`

**CLI arguments** (via `clap` derive):

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--input` | `-i` | `./input` | Input directory path |
| `--output` | `-o` | `./output` | Output directory path |
| `--yes` | `-y` | `false` | Skip confirmation prompt |

**Flow:**

1. Parse CLI args
2. Call `check_ffmpeg_available()` — exit with clear message if missing
3. Call `scan_video_files(input_dir)`
4. Determine output extension from first file's extension
5. Print numbered file list:
   ```
   Found 5 .ts files:
     1. segment_001.ts
     2. segment_002.ts
     3. segment_003.ts
     4. segment_004.ts
     5. segment_005.ts

   Output: ./output/merged_20260526_143000.ts
   Proceed? [y/N]
   ```
6. If `--yes` or user types "y", proceed
7. Create output directory if it doesn't exist
8. Generate output path: `{output_dir}/merged_{YYYYMMDD}_{HHMMSS}.{ext}`
9. Call `merge_videos(files, output_path)`
10. Print result: `Merged 5 files → ./output/merged_20260526_143000.ts (1.2 GB)`

## Dependencies

| Crate | Purpose |
|-------|---------|
| `clap` (derive) | CLI argument parsing |
| `anyhow` | Error propagation |
| `chrono` | Timestamp for output filename |
| `tempfile` | Temp concat list file with auto-cleanup |

## Error Messages

User-facing error messages should be clear and actionable:

- `"Error: ffmpeg not found. Please install ffmpeg (https://ffmpeg.org/download.html)"`
- `"Error: Input directory './input' does not exist"`
- `"Error: No .flv or .ts files found in './input'"`
- `"Error: Merge failed — stream copy failed and re-encode also failed. See stderr above."`

## Testing

- **Unit: `scanner.rs`** — create temp dir with dummy files, verify scan returns correct sorted list, verify empty dir returns error, verify non-video files are filtered out
- **Unit: `ffmpeg.rs`** — test arg builder functions produce correct `Vec<OsString>` (pure functions, no I/O)
- **Integration** — full merge flow with small test video files (optional, requires ffmpeg + test fixtures)
