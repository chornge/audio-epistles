use anyhow::Result;
use dotenvy::dotenv;
use regex::Regex;
use reqwest::get;
use serde_json::Value;
use std::{env, fs};

pub async fn fetch_new_video() -> Result<String> {
    dotenv().ok();
    let playlist_id = env::var("SERMON_PLAYLIST_ID").expect("SERMON_PLAYLIST_ID not set");
    let playlist_url = format!("https://www.youtube.com/playlist?list={playlist_id}");

    let body = get(&playlist_url).await?.text().await?;
    let re = Regex::new(r#""videoId":"([^"]+)""#).unwrap();
    let video_id = re.captures_iter(&body).filter_map(|cap| cap.get(1)).last();

    match video_id {
        Some(id) => {
            println!("📺 Latest video ID: {:?}", id);
            Ok(id.as_str().to_string())
        }
        None => Err(anyhow::anyhow!("No video ID found in playlist")),
    }
}

pub fn last_seen_upload() -> String {
    let episode_json = fs::read_to_string("episode.json").expect("Failed to read episode.json");
    let episode_data: Value = serde_json::from_str(&episode_json).expect("Invalid episode.json");
    episode_data["id"].as_str().unwrap_or_default().to_string()
}
