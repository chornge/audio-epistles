use anyhow::{anyhow, Result};
use dotenv::dotenv;
use regex::Regex;
use reqwest::Client;
use serde::Deserialize;
use std::{env, fs};

#[derive(Deserialize)]
struct PlaylistItemResponse {
    items: Vec<PlaylistItem>,
}

#[derive(Deserialize)]
struct PlaylistItem {
    id: String,
    snippet: Snippet,
}

#[derive(Deserialize)]
struct Snippet {
    published_at: String,
}

#[derive(Deserialize)]
struct NoEmbedResponse {
    title: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = initialize_log_directories();
    let _ = dotenv().map_err(|_| anyhow!("Failed to load .env file"));
    let api_key = env::var("API_KEY").map_err(|_| anyhow!("API_KEY not set"))?;

    let live_services_playlist_id = "PLqOU6DjSKs7wkpl8NK-dplD2o31m1lXFT"; // DCH Live Services
    let last_checked_video_id = read_last_checked_video_id();

    let client = Client::new();
    let url = format!("https://yewtu.be/playlist?list={}", live_services_playlist_id);

    let response: PlaylistItemResponse = client.get(&url).send().await?.json().await?;

    let mut new_video_ids = Vec::new();

    for item in &response.items {
        if item.id == last_checked_video_id {
            break; // Stop processing after reaching the last checked video
        }
        new_video_ids.push(item.id.clone());
    }

    for video_id in new_video_ids.iter().rev() {
        let noembed_url = format!("https://www.youtube.com/watch?v={}", video_id);
        let noembed_response: NoEmbedResponse = client
            .get("https://noemed.com/embed")
            .query(&[("format", "json"), ("url", &noembed_url)])
            .send()
            .await?
            .json()
            .await?;

        let processed_title = process_title(&noembed_response.title);
        println!("{}", processed_title);
    }

    if let Some(most_recent) = response.items.first() {
        save_last_checked_video_id(&most_recent.id);
    }

    Ok(())
}

fn read_last_checked_video_id() -> String {
    fs::read_to_string("last_checked_video_id.txt").unwrap_or_default()
}

fn save_last_checked_video_id(video_id: &str) {
    fs::write("last_checked_video_id.txt", video_id).unwrap();
}

fn process_title(title: &str) -> String {
    // let regex = Regex::new(r"^(.*?)(?=\s+\|)?").unwrap();
    let regex = Regex::new(r"^(.*?)(?=\s+\|)?:?").unwrap();
    if let Some(captures) = regex.captures(title) {
        if let Some(matched) = captures.get(1) {
            return matched.as_str().to_string();
        }
    }

    title.to_string() // Return original title if no match
}

fn initialize_log_directories() -> std::io::Result<()> {
    let months = [
        "1-jan", "2-feb", "3-mar", "4-apr", "5-may", "6-jun", "7-jul", "8-aug", "9-sep", "10-oct",
        "11-nov", "12-dec",
    ];

    fs::create_dir_all("logs")?;

    for &month in months.iter() {
        let month_dir = format!("logs/{}", month);
        fs::create_dir_all(&month_dir)?;
    }

    Ok(())
}
