use anyhow::Result;
use dotenvy::dotenv;
use regex::Regex;
use reqwest::get;
use serde_json::Value;
use std::{env, fs};

pub async fn fetch_new_video() -> Result<String> {
    dotenv().ok();

    let playlist_id = env::var("SERMON_PLAYLIST_ID").expect("SERMON_PLAYLIST_ID is not set");

    let playlist_url = format!("https://www.youtube.com/playlist?list={playlist_id}");

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

    println!("Latest video ID: {video_id:?}");
    Ok(video_id.expect("No video ID found in playlist response"))
}

pub fn last_seen_upload() -> String {
    let episode_json = fs::read_to_string("schroedinger-hat/episode.json")
        .expect("Failed to read schroedinger-hat/episode.json");

    let episode_data: Value =
        serde_json::from_str(&episode_json).expect("schroedinger-hat/episode.json is blank");

    episode_data["id"]
        .as_str()
        .expect("Missing or invalid 'id' field in schroedinger-hat/episode.json")
        .to_string()
}
