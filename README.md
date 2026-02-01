# FLAC to MP3 Converter

A high-performance, parallelized command-line tool written in Rust to convert music libraries from FLAC to MP3.

## Features

- **Parallel Processing**: Uses `rayon` to process multiple albums simultaneously, maximizing CPU usage.
- **Smart Mirroring**: 
  - Converts `.flac` files to `.mp3` using `ffmpeg`.
  - Copies `.mp3` and other non-audio files (artwork, logs, cues) directly to the output.
  - Preserves directory structure.
  - **Incremental Update**: Skips files that already exist in the output directory (unless `--force` is used).
- **Safety**: Skips albums that contain mixed FLAC and MP3 content to avoid duplication or overwrite issues (logs a warning).
- **Quality Control**: Configurable VBR quality settings.

## Library Structure Assumptions

The tool assumes a nested directory structure, typically organized as `Artist/Album/Files`. 

1. **Album Definition**: An "album" is defined as any directory that contains files. The tool discovers these by walking the input directory.
2. **Atomic Processing**: Each directory (album) is processed as a unit. 
3. **Safety Rule**: If a directory contains both `.flac` and `.mp3` files, it is considered "mixed" and will be skipped to prevent inconsistent output. A warning will be logged.
4. **Non-Audio Files**: Any files that are not `.flac` or `.mp3` (e.g., `folder.jpg`, `album.log`, `.cue` files) are copied directly to the output directory to preserve metadata and artwork.
5. **Path Preservation**: The relative path from the input root to the album directory is preserved in the output root.

## Prerequisites

- **Rust**: [Install Rust](https://www.rust-lang.org/tools/install)
- **FFmpeg**: Must be installed and available in your system's PATH.
  - Ubuntu/Debian: `sudo apt install ffmpeg`
  - macOS: `brew install ffmpeg`

## Usage

Run the tool using `cargo`:

```bash
cargo run --release -- --input <INPUT_DIR> --output <OUTPUT_DIR> [OPTIONS]
```

### Arguments

- `-i, --input <PATH>`: Input directory containing the source music library.
- `-o, --output <PATH>`: Output directory for the converted library.
- `-t, --target <PATH>`: Specific album or subdirectory to convert (relative to input root).
- `-q, --quality <0-9>`: MP3 VBR quality setting (Default: 0).
  - `0`: Highest quality (largest file size).
  - `9`: Lowest quality (smallest file size).
- `-f, --force`: Force overwrite of existing files.
- `-d, --debug`: Enable debug logging (shows album inspection details).

## Progress Bar

The tool displays a progress bar (similar to `tqdm` in Python) showing:
- Elapsed time
- Progress percentage and bar
- Completed/Total files
- Estimated time of arrival (ETA)
- Status message

### Examples

Convert a library with default quality (V0):

```bash
cargo run --release -- --input /data/music_flac --output /data/music_mp3
```

Convert with lower quality (V5) to save space:

```bash
cargo run --release -- --input /data/music_flac --output /data/music_mp3 --quality 5
```

Convert a single album:

```bash
cargo run --release -- --input /data/music_flac --output /data/music_mp3 --target "Artist/Album"
```


## Project Structure

- `src/main.rs`: Entry point, CLI argument parsing, and parallel orchestration.
- `src/album.rs`: Logic for discovering and grouping files into albums.
- `src/convert.rs`: Wrapper around `ffmpeg` for conversion and file copying.
