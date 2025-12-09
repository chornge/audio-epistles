//! Video processing orchestration module.
//!
//! This module coordinates the complete workflow of processing a YouTube video:
//! downloading, extracting audio, and uploading to Spotify for Podcasters.

use anyhow::Result;
use tracing::info;

use crate::{
    episode::{extract_sermon_chapter, fetch_metadata, trim_audio},
    webdriver::upload,
};

/// Processes a YouTube video by downloading, extracting audio, and uploading.
///
/// This is the main entry point for processing a new video. It delegates to
/// the `schedule` function to perform the actual work.
///
/// # Arguments
///
/// * `video_id` - The YouTube video ID to process
///
/// # Returns
///
/// Returns `Ok(())` on success.
///
/// # Errors
///
/// Returns an error if any step in the processing pipeline fails:
/// - Video download failure
/// - Audio extraction failure
/// - Upload to Spotify failure
///
/// # Example
///
/// ```no_run
/// # tokio_test::block_on(async {
/// audio_epistles::processor::process("dQw4w9WgXcQ").await.unwrap();
/// # })
/// ```
pub async fn process(video_id: &str) -> Result<()> {
    schedule(video_id).await?;
    Ok(())
}

/// Downloads a video, extracts the sermon audio segment, and uploads to Spotify.
///
/// This function performs the complete processing workflow:
/// 1. Downloads the video and fetches metadata using yt-dlp
/// 2. Parses the video description to find sermon chapter timestamps
/// 3. Extracts the sermon audio segment using ffmpeg
/// 4. Uploads the audio file to Spotify for Podcasters as a draft episode
///
/// The extracted audio is saved to `assets/audio.mp3`. If no sermon chapter
/// is found in the description, the entire video audio is extracted.
///
/// # Arguments
///
/// * `video_id` - The YouTube video ID to process
///
/// # Returns
///
/// Returns `Ok(())` on success.
///
/// # Errors
///
/// Returns an error if:
/// - Video download or metadata fetching fails
/// - No sermon chapter is found and no fallback is available
/// - Audio extraction with ffmpeg fails
/// - Upload to Spotify fails
///
/// # Example
///
/// ```no_run
/// # tokio_test::block_on(async {
/// audio_epistles::processor::schedule("dQw4w9WgXcQ").await.unwrap();
/// # })
/// ```
pub async fn schedule(video_id: &str) -> Result<()> {
    let (title, desc, video_path, duration) = fetch_metadata(video_id).await?;
    info!(title = %title, path = %video_path, duration_secs = duration, "Video metadata retrieved");

    let output_audio = "assets/audio.mp3";

    if let Some((start, end)) = extract_sermon_chapter(&desc, duration) {
        let duration = end - start;
        trim_audio(&video_path, output_audio, start, duration)?;
        info!(output_path = %output_audio, duration_secs = duration, "Audio saved successfully");
    }

    upload(&title).await?;

    Ok(())
}
