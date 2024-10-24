use anyhow::{anyhow, Result};
use dotenv::dotenv;
use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::Command;
use std::{env, fs};
use tokio::time::{sleep, Duration};

#[derive(Serialize, Deserialize, Debug)]
struct PlaylistItemResponse {
    data: Vec<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct VideoEntry {
    title: String,
    video_id: String,
}

impl VideoEntry {
    fn from_raw_data(data: &[String]) -> Self {
        let url = &data[0];
        let video_id_regex = Regex::new(r"watch\?v=([^&]+)").unwrap();

        let video_id = video_id_regex
            .captures(url)
            .and_then(|caps| caps.get(1))
            .map_or(String::new(), |m| m.as_str().to_string());

        let title = data[1].clone();

        VideoEntry { title, video_id }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = initialize_log_directories();
    let _ = dotenv().map_err(|_| anyhow!("Failed to load .env file"));

    let _ = env::var("SPOTIFY_EMAIL").map_err(|_| anyhow!("SPOTIFY_EMAIL not set"))?;
    let _ = env::var("SPOTIFY_PASSWORD").map_err(|_| anyhow!("SPOTIFY_PASSWORD not set"))?;

    let live_services_playlist_id = "PLqOU6DjSKs7wkpl8NK-dplD2o31m1lXFT";

    loop {
        // Every 2 hours
        let last_checked_video_id = read_last_checked_video_id();
        let url = format!(
            "https://yewtu.be/playlist?list={}",
            live_services_playlist_id
        );

        let response = reqwest::get(&url).await?;
        let body = response.text().await?;
        let document = Html::parse_document(&body);
        let selector = Selector::parse(".video-card-row:not(.flexible)").unwrap();

        let mut extracted_data = Vec::new();

        // Extract data using selector
        for element in document.select(&selector) {
            if let Some(a_element) = element.select(&Selector::parse("a").unwrap()).next() {
                let link = a_element.value().attr("href").unwrap_or("").to_string();

                if let Some(p_element) = a_element.select(&Selector::parse("p").unwrap()).next() {
                    let title = p_element.text().collect::<Vec<_>>().join(", ");
                    extracted_data.push(vec![link, title]);
                }
            }
        }

        // Fetch 2 most recent uploads
        let structured_response: Vec<VideoEntry> = extracted_data
            .iter()
            .map(|entry| VideoEntry::from_raw_data(entry))
            .take(2)
            .collect();

        let mut new_video_ids = Vec::new();

        let episode_json = fs::read_to_string("episode.json")?;
        let mut episode_data: Value = serde_json::from_str(&episode_json)?;

        for video in &structured_response {
            if video.video_id == last_checked_video_id {
                break; // Stop processing after reaching the last checked video
            }
            new_video_ids.push(video.video_id.clone());
        }

        // Switch to main branch (before publishing)
        let _ = Command::new("git")
            .args(["checkout", "main"])
            .spawn()?
            .wait()?;

        for new_video_id in new_video_ids.iter().rev() {
            episode_data["id"] = Value::String(new_video_id.clone());

            let updated_json = serde_json::to_string(&episode_data)?;
            fs::write("episode.json", &updated_json)?;

            let _ = Command::new("git")
                .args(["add", "episode.json"])
                .spawn()?
                .wait()?;
            let _ = Command::new("git")
                .args(["commit", "-m", "Upload new episode"])
                .spawn()?
                .wait()?;
            let _ = Command::new("git").args(["push"]).spawn()?.wait()?;
            save_last_checked_video_id(new_video_id);
        }

        // Sleep for 2 hours (7200 seconds) - if not running crontab
        sleep(Duration::from_secs(2 * 3600)).await;
    }
}

fn read_last_checked_video_id() -> String {
    fs::read_to_string("last_checked_video_id.txt").unwrap_or_default()
}

fn save_last_checked_video_id(video_id: &str) {
    fs::write("last_checked_video_id.txt", video_id).unwrap();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_entry_from_raw_data() {
        let raw_data = vec![
            String::from("/watch?v=mVda8IUcKEQ&list=PLID"),
            String::from("Sample Video Title"),
        ];

        let video_entry = VideoEntry::from_raw_data(&raw_data);

        assert_eq!(video_entry.video_id, "mVda8IUcKEQ");
        assert_eq!(video_entry.title, "Sample Video Title");
    }
}
