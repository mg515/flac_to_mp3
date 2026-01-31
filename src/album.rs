use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use std::collections::HashMap;
use anyhow::Result;

#[derive(Debug)]
pub struct Album {
    pub path: PathBuf,
    pub files: Vec<PathBuf>,
}

impl Album {
    pub fn new(path: PathBuf, files: Vec<PathBuf>) -> Self {
        Self { path, files }
    }

    /// Discover albums in the input directory.
    /// An album is defined as a directory containing relevant files.
    pub fn discover(input_dir: &Path) -> Result<Vec<Album>> {
        let mut albums_map: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();

        for entry in WalkDir::new(input_dir).follow_links(true).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let path = entry.path().to_path_buf();
                if let Some(parent) = path.parent() {
                    albums_map.entry(parent.to_path_buf())
                        .or_default()
                        .push(path);
                }
            }
        }

        let albums: Vec<Album> = albums_map.into_iter()
            .map(|(path, files)| Album::new(path, files))
            .collect();

        Ok(albums)
    }

    /// Check if the album is valid for conversion.
    /// Returns true if it contains only FLACs/MP3s/other allowed files.
    /// Returns false if it contains mixed FLAC and MP3 files (which should be skipped).
    pub fn validate(&self) -> bool {
        let has_flac = self.files.iter().any(|f| f.extension().and_then(|e| e.to_str()) == Some("flac"));
        let has_mp3 = self.files.iter().any(|f| f.extension().and_then(|e| e.to_str()) == Some("mp3"));

        if has_flac && has_mp3 {
            log::warn!("Skipping album with mixed FLAC and MP3 files: {:?}", self.path);
            return false;
        }

        true
    }
}
