use anyhow::Result;
use dotenvy::dotenv;
use regex::Regex;
use reqwest::get;
use serde_json::Value;
use std::{env, fs};

pub async fn fetch_new_video() -> Result<String> {
    dotenv().ok();

    let playlist_id = env::var("SERMON_PLAYLIST_ID").expect("SERMON_PLAYLIST_ID is not set");

    let playlist_url = format!("https://www.youtube.com/playlist?list={}", playlist_id);

    let response = get(playlist_url)
        .await
        .expect("Failed to send request for playlist");

    let body = response
        .text()
        .await
        .expect("Failed to read response body for playlist");

    let re = Regex::new(r#""videoId":"([^"]+)""#).unwrap();
    let video_id = re
        .captures_iter(&body)
        .filter_map(|cap| cap.get(1).map(|id| id.as_str().to_string()))
        .last();

    println!("Latest video ID: {:?}", video_id);
    Ok(video_id.expect("No video ID found in playlist response"))
}

pub fn last_seen_upload() -> String {
    let episode_json = fs::read_to_string("schroedinger-hat/episode.son")
        .expect("Failed to read schroedinger-hat/episode.json");

    let episode_data: Value =
        serde_json::from_str(&episode_json).expect("schroedinger-hat/episode.json is blank");

    episode_data["id"]
        .as_str()
        .expect("Missing or invalid 'id' field in schroedinger-hat/episode.json")
        .to_string()
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_last_seen_upload_valid_json() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("episode.json");
        let json_content = r#"{"id":"video_id"}"#;
        let mut file = File::create(&file_path).unwrap();
        file.write_all(json_content.as_bytes()).unwrap();

        // Temporarily rename file to match expected path
        let orig_path = "schroedinger-hat/episode.son";
        fs::create_dir_all("schroedinger-hat").unwrap();
        fs::copy(&file_path, orig_path).unwrap();

        let id = last_seen_upload();
        assert_eq!(id, "video_id");

        // Clean up
        fs::remove_file(orig_path).unwrap();
    }

    #[test]
    #[should_panic(expected = "schroedinger-hat/episode.json is blank")]
    fn test_last_seen_upload_invalid_json() {
        fs::create_dir_all("schroedinger-hat").unwrap();
        let orig_path = "schroedinger-hat/episode.son";
        let mut file = File::create(orig_path).unwrap();
        file.write_all(b"not json").unwrap();

        last_seen_upload();

        // Clean up
        let _ = fs::remove_file(orig_path);
    }
}
