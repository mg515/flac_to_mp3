use clap::Parser;
use rayon::prelude::*;
use std::path::PathBuf;
use std::time::Instant;
use std::sync::atomic::{AtomicUsize, Ordering};
use env_logger::Builder;

mod album;
mod convert;

use album::Album;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input directory containing the music library
    #[arg(short, long)]
    input: PathBuf,

    /// Output directory for the converted library
    #[arg(short, long)]
    output: PathBuf,

    /// MP3 VBR quality (0-9, where 0 is highest quality/largest size)
    #[arg(short, long, default_value_t = 0, value_parser = clap::value_parser!(u8).range(0..=9))]
    quality: u8,

    /// Specific album or subdirectory to convert (relative to input root)
    #[arg(short, long)]
    target: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    // Initialize logger
    let mut builder = Builder::from_default_env();
    if std::env::var("RUST_LOG").is_err() {
        builder.filter_level(log::LevelFilter::Info);
    }
    builder.init();
    
    let args = Args::parse();
    let start_time = Instant::now();

    let scan_path = match &args.target {
        Some(target) => args.input.join(target),
        None => args.input.clone(),
    };

    if !scan_path.exists() {
        anyhow::bail!("Target path does not exist: {:?}", scan_path);
    }

    log::info!("Scanning for albums in {:?}", scan_path);
    let albums = Album::discover(&scan_path)?;
    log::info!("Found {} potential albums folders.", albums.len());

    let success_count = AtomicUsize::new(0);
    let failure_count = AtomicUsize::new(0);
    let mixed_count = AtomicUsize::new(0);

    albums.par_iter().for_each(|album| {
        if !album.validate() {
            mixed_count.fetch_add(1, Ordering::Relaxed);
            return;
        }

        match convert::process_album(album, &args.input, &args.output, args.quality) {
            Ok(_) => {
                success_count.fetch_add(1, Ordering::Relaxed);
            }
            Err(e) => {
                log::error!("Failed to process album {:?}: {:#}", album.path, e);
                failure_count.fetch_add(1, Ordering::Relaxed);
            }
        }
    });

    let duration = start_time.elapsed();
    log::info!("Conversion finished in {:.2?}", duration);
    log::info!("Summary: {} successful, {} failed, {} skipped (mixed content).", 
               success_count.load(Ordering::Relaxed), 
               failure_count.load(Ordering::Relaxed), 
               mixed_count.load(Ordering::Relaxed));

    Ok(())
}
