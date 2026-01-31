use clap::Parser;
use rayon::prelude::*;
use std::path::PathBuf;
use std::time::Instant;
use std::sync::atomic::{AtomicUsize, Ordering};
use env_logger::Builder;
use indicatif::{ProgressBar, ProgressStyle};

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

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize logger
    let mut builder = Builder::from_default_env();
    if args.debug {
        builder.filter_level(log::LevelFilter::Debug);
    } else if std::env::var("RUST_LOG").is_err() {
        builder.filter_level(log::LevelFilter::Info);
    }
    builder.init();
    
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
    log::info!("Found {} potential album folders.", albums.len());

    let mut all_tasks = Vec::new();
    let mut mixed_count = 0;

    for album in &albums {
        log::debug!("Inspecting album: {:?}", album.path);
        if !album.validate() {
            log::warn!("Album failed validation (mixed FLAC/MP3): {:?}", album.path);
            mixed_count += 1;
            continue;
        }

        match convert::collect_album_tasks(album, &args.input, &args.output, args.quality) {
            Ok(album_tasks) => {
                log::debug!("Found {} tasks for album: {:?}", album_tasks.len(), album.path);
                all_tasks.extend(album_tasks);
            }
            Err(e) => {
                log::error!("Failed to prepare tasks for album {:?}: {:#}", album.path, e);
            }
        }
    }

    let task_count = all_tasks.len();
    log::info!("Prepared {} individual file tasks.", task_count);
    if mixed_count > 0 {
        log::info!("Skipped {} mixed-content albums.", mixed_count);
    }

    if task_count == 0 {
        log::info!("No tasks to process.");
        return Ok(());
    }

    // Set up progress bar
    let pb = ProgressBar::new(task_count as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")?
        .progress_chars("#>-"));

    let success_count = AtomicUsize::new(0);
    let failure_count = AtomicUsize::new(0);

    all_tasks.par_iter().for_each(|task| {
        match task.execute() {
            Ok(_) => {
                success_count.fetch_add(1, Ordering::Relaxed);
            }
            Err(e) => {
                log::error!("Task failed: {:#}", e);
                failure_count.fetch_add(1, Ordering::Relaxed);
            }
        }
        pb.inc(1);
    });

    pb.finish_with_message("Done!");

    let duration = start_time.elapsed();
    log::info!("Conversion finished in {:.2?}", duration);
    log::info!("Summary: {} successful, {} failed, {} skipped (mixed albums).", 
               success_count.load(Ordering::Relaxed), 
               failure_count.load(Ordering::Relaxed), 
               mixed_count);

    Ok(())
}
