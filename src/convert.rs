use std::path::Path;
use std::process::Command;
use std::fs;
use anyhow::{Result, Context};
use crate::album::Album;

pub fn process_album(album: &Album, input_root: &Path, output_root: &Path, quality: u8) -> Result<()> {
    // Determine the relative path of the album from the input root
    let relative_path = album.path.strip_prefix(input_root)
        .context("Failed to strip prefix from album path")?;
    
    // Create the corresponding output directory
    let output_dir = output_root.join(relative_path);
    fs::create_dir_all(&output_dir)
        .with_context(|| format!("Failed to create output directory: {:?}", output_dir))?;

    for file_path in &album.files {
        let file_name = file_path.file_name()
            .context("Failed to get file name")?;
        
        let extension = file_path.extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase());

        match extension.as_deref() {
            Some("flac") => {
                let output_filename = Path::new(file_name).with_extension("mp3");
                let output_path = output_dir.join(output_filename);
                convert_flac_to_mp3(file_path, &output_path, quality)?;
            }
            _ => {
                // For mp3 or other files, just copy
                let output_path = output_dir.join(file_name);
                fs::copy(file_path, &output_path)
                    .with_context(|| format!("Failed to copy file {:?} to {:?}", file_path, output_path))?;
            }
        }
    }

    Ok(())
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
