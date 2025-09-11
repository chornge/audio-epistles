use anyhow::Result;
use std::fs;

// use crate::scheduler;

/// Process new video by its ID
pub async fn process_video(video_id: &str) -> Result<()> {
    publish(video_id).await?;
    Ok(())
}

pub async fn publish(video_id: &str) -> Result<()> {
    let file_path = "episode.json";
    let mut video_data = if let Ok(contents) = fs::read_to_string(file_path) {
        serde_json::from_str(&contents).unwrap_or_else(|_| serde_json::json!({ "id": "" }))
    } else {
        serde_json::json!({ "id": "" })
    };
    video_data["id"] = serde_json::json!(video_id);
    fs::write(file_path, video_data.to_string())?;
    println!("📄 Updated episode.json with: {video_id}");

    let (title, desc, video_path, duration) = crate::episode::fetch_metadata(video_id).await?;
    println!("🎬 Title: {title}");
    println!("🎬 Path: {video_path}");
    println!("🎬 Duration: {duration}");

    let output_audio = "assets/audio.mp3";

    if let Some((start, end)) = crate::episode::extract_sermon_chapter(&desc, duration) {
        let duration = end - start;
        println!("✂️ Trimming sermon chapter from {}s to {}s...", start, end);
        crate::episode::trim_audio(&video_path, output_audio, start, duration)?;
        println!("✅ Trimmed sermon audio saved to {}", output_audio);
    } else {
        println!("🎧 No sermon chapter detected, converting full audio...");
        crate::episode::convert_full_audio(&video_path, output_audio)?;
        println!("✅ Full audio saved to {}", output_audio);
    }

    // scheduler::schedule()?;
    println!("✅ Publish completed via scheduler.");

    Ok(())
}
