use anyhow::Result;
use regex::Regex;
use reqwest::get;
use serde_json::Value;
use std::{env, fs};

pub async fn fetch_new_video() -> Result<String> {
    // Fetch playlist ID from environment variable
    let playlist_id = env::var("SERMON_PLAYLIST_ID").expect("SERMON_PLAYLIST_ID is not set");

    // Dynamically construct playlist URL
    let playlist_url = format!("https://www.youtube.com/playlist?list={}", playlist_id);
    let response = get(playlist_url)
        .await
        .expect("Failed to send request for playlist");
    let body = response
        .text()
        .await
        .expect("Failed to read response body for playlist");

    // Extract the most recent video's ID
    let re = Regex::new(r#""videoId":"([^"]+)""#).unwrap();
    let video_id = re
        .captures_iter(&body)
        .filter_map(|cap| cap.get(1).map(|id| id.as_str().to_string()))
        .last();

    println!("Most recent video ID: {:?}", video_id);
    Ok(video_id.expect("No video ID found in playlist response"))
}

pub fn last_seen_upload() -> String {
    let episode_json = fs::read_to_string("video_id.json").expect("Failed to read video_id.json");
    let episode_data: Value = serde_json::from_str(&episode_json).expect("video_id.json is blank");

    episode_data["id"]
        .as_str()
        .expect("Missing or invalid 'id' field in video_id.json")
        .to_string()
}
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires actual playlist ID
    async fn test_fetch_new_video() {
        let playlist_id =
            std::env::var("SERMON_PLAYLIST_ID").unwrap_or_else(|_| "playlist_id".to_string());
        std::env::set_var("SERMON_PLAYLIST_ID", playlist_id);

        let new_video_id = fetch_new_video().await;
        assert!(new_video_id.is_ok(), "Failed to fetch new video ID");
    }

    #[test]
    fn test_last_seen_upload() {
        let last_id = last_seen_upload();
        assert!(
            !last_id.is_empty(),
            "Last seen upload ID should not be empty"
        );
    }
}
