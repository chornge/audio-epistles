use anyhow::Result;
use serde_json::Value;
use std::fs;
use std::process::Command;

use crate::VideoEntry;

pub async fn process_new_videos(new_videos: Vec<VideoEntry>) -> Result<()> {
    let episode_json = fs::read_to_string("episode.json")?;
    let mut episode_data: Value = serde_json::from_str(&episode_json)?;

    // Publish in reverse order (newer uploads get published last)
    for new_video in new_videos.iter().rev() {
        publish_video(new_video, &mut episode_data).await?;
    }
    Ok(())
}

pub async fn publish_video(new_video: &VideoEntry, episode_data: &mut Value) -> Result<()> {
    // Checkout main branch
    let _ = Command::new("git")
        .args(["checkout", "main"])
        .spawn()?
        .wait()?;

    // Write new video ID to episode.json
    episode_data["id"] = Value::String(new_video.id.clone());
    let updated_json = serde_json::to_string(&episode_data)?;
    fs::write("episode.json", &updated_json)?;

    // Add changes
    let _ = Command::new("git")
        .args(["add", "episode.json"])
        .spawn()?
        .wait()?;

    // Commit changes
    let _ = Command::new("git")
        .args(["commit", "-m", "Update episode ID for publishing"])
        .spawn()?
        .wait()?;

    // Push changes
    let _ = Command::new("git").args(["push"]).spawn()?.wait()?;

    Ok(())
}
