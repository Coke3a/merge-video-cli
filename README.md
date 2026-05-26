# merge-video

A simple CLI tool to merge multiple video files into one using ffmpeg's concat demuxer.

Supports `.ts` and `.flv` files. Tries lossless stream copy first for speed; falls back to re-encode if that fails.

## Prerequisites

- [Rust](https://rustup.rs/) (1.70+)
- [ffmpeg](https://ffmpeg.org/download.html) installed and available in `PATH`

## Installation

```bash
git clone https://github.com/your-username/merge-video.git
cd merge-video
cargo build --release
```

The binary will be at `target/release/merge_video`.

## Usage

### 1. Place video files in the input directory

```bash
cp /path/to/segment_001.ts /path/to/segment_002.ts input/
```

All files must be the same format (all `.ts` or all `.flv`).

### 2. Run

```bash
cargo run
```

The tool will:
1. Scan the input directory and sort files alphabetically by filename
2. Display a numbered list for you to review the order
3. Ask for confirmation
4. Merge into a single timestamped output file

```
Found 3 .ts files:
  1. segment_001.ts
  2. segment_002.ts
  3. segment_003.ts

Output: ./output/merged_20260526_143000.ts
Proceed? [y/N] y
Merged 3 files → ./output/merged_20260526_143000.ts (1.2 GB)
```

### Skip confirmation

```bash
cargo run -- --yes
```

### Custom directories

```bash
cargo run -- --input /path/to/videos --output /path/to/result
```

### All options

```
Usage: merge_video [OPTIONS]

Options:
  -i, --input <INPUT>    Input directory [default: ./input]
  -o, --output <OUTPUT>  Output directory [default: ./output]
  -y, --yes              Skip confirmation prompt
  -h, --help             Print help
```

## How it works

1. Scans the input directory for `.ts` or `.flv` files (non-recursive)
2. Sorts files alphabetically by filename — name your files with numeric prefixes (e.g., `001_`, `002_`) to control order
3. Writes an ffmpeg concat demuxer list to a temp file
4. Runs `ffmpeg -f concat -c copy` (stream copy, no quality loss, very fast)
5. If stream copy fails (e.g., codec mismatch between files), falls back to re-encode with `libx264` + `aac`
6. Output format matches the input (`.ts` in = `.ts` out)

## Running tests

```bash
cargo test
```

## License

MIT
