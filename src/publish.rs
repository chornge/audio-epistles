use anyhow::Result;
use serde_json::json;
use std::fs;
use std::process::{Command, Stdio};

pub async fn process_video(new_video: String) -> Result<()> {
    publish(&new_video).await?;
    Ok(())
}

#[allow(unused_variables)]
pub async fn publish(video_json: &str) -> Result<()> {
    // Read the existing JSON file
    let file_path = "schroedinger-hat/episode.json";
    let mut video_data = if let Ok(contents) = fs::read_to_string(file_path) {
        serde_json::from_str(&contents).unwrap_or_else(|_| json!({ "id": "" }))
    } else {
        json!({ "id": "" }) // Default if file doesn't exist
    };

    // Update the JSON with the new video ID
    video_data["id"] = json!(video_json);

    // Write the updated JSON to schroedinger-hat/episode.json
    fs::write(file_path, video_data.to_string())?;
    println!("Update schroedinger-hat/episode.json with: {}", video_json);

    // Publish to Spotify
    println!("Processing {}...", video_json);
    let output = Command::new("npm")
        .arg("start")
        .current_dir("schroedinger-hat")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if output.status.success() {
        println!("Publish to Spotify succeeded!");
    } else {
        // Capture and display stderr
        let error_message = String::from_utf8_lossy(&output.stderr);
        eprintln!("Publish to Spotify failed! Error: {}", error_message);
    }

    Ok(())
}
