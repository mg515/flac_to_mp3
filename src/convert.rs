use crate::album::Album;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug)]
pub enum Task {
    Convert {
        input: PathBuf,
        output: PathBuf,
        quality: u8,
    },
    Copy {
        input: PathBuf,
        output: PathBuf,
    },
}

impl Task {
    pub fn execute(&self) -> Result<()> {
        match self {
            Task::Convert {
                input,
                output,
                quality,
            } => convert_flac_to_mp3(input, output, *quality),
            Task::Copy { input, output } => {
                fs::copy(input, output)
                    .with_context(|| format!("Failed to copy file {:?} to {:?}", input, output))?;
                Ok(())
            }
        }
    }
}

pub fn collect_album_tasks(
    album: &Album,
    input_root: &Path,
    output_root: &Path,
    quality: u8,
) -> Result<Vec<Task>> {
    let mut tasks = Vec::new();

    // Determine the relative path of the album from the input root
    let relative_path = album
        .path
        .strip_prefix(input_root)
        .context("Failed to strip prefix from album path")?;

    // Create the corresponding output directory
    let output_dir = output_root.join(relative_path);
    fs::create_dir_all(&output_dir)
        .with_context(|| format!("Failed to create output directory: {:?}", output_dir))?;

    for file_path in &album.files {
        let file_name = file_path.file_name().context("Failed to get file name")?;

        let extension = file_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase());

        match extension.as_deref() {
            Some("flac") => {
                let output_filename = Path::new(file_name).with_extension("mp3");
                let output_path = output_dir.join(output_filename);
                tasks.push(Task::Convert {
                    input: file_path.clone(),
                    output: output_path,
                    quality,
                });
            }
            _ => {
                // For mp3 or other files, just copy
                let output_path = output_dir.join(file_name);
                tasks.push(Task::Copy {
                    input: file_path.clone(),
                    output: output_path,
                });
            }
        }
    }

    Ok(tasks)
}

fn convert_flac_to_mp3(input: &Path, output: &Path, quality: u8) -> Result<()> {
    // ffmpeg -i input.flac -codec:a libmp3lame -q:a <quality> -map_metadata 0 output.mp3
    let status = Command::new("ffmpeg")
        .arg("-i")
        .arg(input)
        .arg("-codec:a")
        .arg("libmp3lame")
        .arg("-q:a")
        .arg(quality.to_string())
        .arg("-map_metadata")
        .arg("0")
        .arg("-y") // Overwrite output file without asking
        .arg(output) // Output file
        .arg("-hide_banner")
        .arg("-loglevel")
        .arg("error")
        .status()
        .context("Failed to execute ffmpeg command")?;

    if !status.success() {
        anyhow::bail!("ffmpeg failed to convert {:?} to {:?}", input, output);
    }

    Ok(())
}
