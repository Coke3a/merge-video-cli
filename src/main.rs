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

    for entry in std::fs::read_dir(&cli.output)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some(&ext) {
            std::fs::remove_file(&path)
                .with_context(|| format!("Failed to remove old output: {}", path.display()))?;
            eprintln!("Removed old output: {}", path.file_name().unwrap_or_default().to_string_lossy());
        }
    }

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
