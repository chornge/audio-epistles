use anyhow::Result;

use crate::{
    episode::{extract_sermon_chapter, fetch_metadata, trim_audio},
    webdriver::upload,
};

pub async fn process(video_id: &str) -> Result<()> {
    schedule(video_id).await?;
    Ok(())
}

pub async fn schedule(video_id: &str) -> Result<()> {
    let (title, desc, video_path, duration) = fetch_metadata(video_id).await?;
    println!("🎬 Title: {title}");
    println!("🎬 Path: {video_path}");
    println!("🎬 Duration: {duration}");

    let output_audio = "assets/audio.mp3";

    if let Some((start, end)) = extract_sermon_chapter(&desc, duration) {
        let duration = end - start;
        trim_audio(&video_path, output_audio, start, duration)?;
        println!("✅ Audio saved to {output_audio}");
    }

    upload(&title).await?;

    Ok(())
}
