//! Audio Epistles automation tool.
//!
//! This application automatically monitors a YouTube playlist for new sermon videos,
//! downloads them, extracts the sermon audio segment, and uploads it to Spotify for Podcasters.

mod db;
mod episode;
mod processor;
mod types;
mod video;
mod webdriver;

use anyhow::Result;
use db::{get_last_id, save_id};
use dotenvy::dotenv;
use processor::process;
use sqlx::SqlitePool;
use std::env;
use std::time::Instant;
use tracing::{error, info};
use video::fetch_video;

/// Main entry point for the audio epistles automation tool.
///
/// This function orchestrates the entire workflow:
/// 1. Initializes the tracing subscriber for structured logging
/// 2. Connects to the SQLite database to track processed videos
/// 3. Fetches the latest video ID from the configured YouTube playlist
/// 4. Compares it with the last processed video
/// 5. If a new video is found, processes it (download, extract audio, upload to Spotify)
/// 6. Updates the database with the new video ID upon successful completion
///
/// The function tracks execution time and logs the total duration at the end.
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error if database connection or environment setup fails.
///
/// # Errors
///
/// Returns an error if:
/// - The `DB_URL` environment variable is not set
/// - Database connection or initialization fails
/// - Any critical operation in the pipeline fails
///
/// # Example
///
/// ```no_run
/// # tokio_test::block_on(async {
/// // Ensure .env file contains DB_URL and other required variables
/// let result = audio_epistles::main().await;
/// assert!(result.is_ok());
/// # })
/// ```
#[tokio::main]
async fn main() -> Result<()> {
    let timer = Instant::now();

    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    dotenv().ok();

    let db_url = env::var("DB_URL")?;
    let pool = SqlitePool::connect(&db_url).await?;

    db::init(&pool).await?;

    let last_seen_id = get_last_id(&pool).await?;

    match fetch_video().await {
        Ok(video_id) => {
            if video_id != last_seen_id {
                if let Err(e) = process(&video_id).await {
                    error!(error = %e, "Failed to process new video");
                } else {
                    let mut transaction = pool.begin().await?;
                    save_id(&mut transaction, &video_id).await?;
                    transaction.commit().await?;
                    info!(video_id = %video_id, "Updated DB with video ID");
                }
            } else {
                info!("No new video found since last publish");
            }
        }
        Err(e) => error!(error = %e, "Failed to fetch new video ID"),
    }

    let duration = timer.elapsed();
    let total_seconds = duration.as_secs();
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;

    let formatted = if minutes > 0 {
        format!("{minutes}min {seconds}sec")
    } else {
        format!("{seconds}sec")
    };

    info!(duration_secs = total_seconds, "Finished in {}", formatted);

    Ok(())
}
