use anyhow::{anyhow, Result};
use dotenv::dotenv;
use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::Command;
use std::{env, fs};
use tokio::time::{sleep, Duration};

mod log;
use crate::log::log_event;

#[derive(Debug, Deserialize, Serialize)]
struct PlaylistItemResponse {
    data: Vec<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct VideoEntry {
    id: String,
    title: String,
}

#[allow(clippy::ptr_arg)]
impl VideoEntry {
    fn from_raw_data(data: &Vec<String>) -> Self {
        let url = &data[0];
        let video_id_regex = Regex::new(r"watch\?v=([^&]+)").unwrap();

        let id = video_id_regex
            .captures(url)
            .and_then(|caps| caps.get(1))
            .map_or(String::new(), |m| m.as_str().to_string());

        let title = parse_title(&data[1]);

        VideoEntry { id, title }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = initialize_log_directories();
    let _ = dotenv().map_err(|_| anyhow!("Failed to load .env file"));

    let _ = env::var("ANCHOR_EMAIL").map_err(|_| anyhow!("ANCHOR_EMAIL not set"))?;
    let _ = env::var("ANCHOR_PASSWORD").map_err(|_| anyhow!("ANCHOR_PASSWORD not set"))?;
    
    let _ = env::var("SPOTIFY_EMAIL").map_err(|_| anyhow!("SPOTIFY_EMAIL not set"))?;
    let _ = env::var("SPOTIFY_PASSWORD").map_err(|_| anyhow!("SPOTIFY_PASSWORD not set"))?;

    let live_services_playlist_id = "PLqOU6DjSKs7wkpl8NK-dplD2o31m1lXFT";

    loop {
        let url = format!(
            "https://yewtu.be/playlist?list={}",
            live_services_playlist_id
        );

        let response = reqwest::get(&url).await?;
        let body = response.text().await?;
        let document = Html::parse_document(&body);

        // Extract data using css selector
        let mut extracted_data = Vec::new();
        let selector = Selector::parse(".video-card-row:not(.flexible)").unwrap();
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
        let most_recent_uploads: Vec<VideoEntry> = extracted_data
            .iter()
            .map(VideoEntry::from_raw_data)
            .take(2)
            .collect();

        let mut new_videos = Vec::new();
        for video in &most_recent_uploads {
            if video.id == last_checked_video() {
                break; // Stop processing after reaching the last checked video
            }
            new_videos.push(video);
        }

        let episode_json = fs::read_to_string("episode.json")?;
        let mut episode_data: Value = serde_json::from_str(&episode_json)?;

        // Publish in reverse order (newer uploads get published last)
        for new_video in new_videos.iter().rev() {
            // Publish from main branch
            let _ = Command::new("git")
                .args(["checkout", "main"])
                .spawn()?
                .wait()?;

            episode_data["id"] = Value::String(new_video.id.clone());
            let updated_json = serde_json::to_string(&episode_data)?;
            fs::write("episode.json", &updated_json)?;

            let _ = Command::new("git")
                .args(["add", "episode.json"])
                .spawn()?
                .wait()?;
            let _ = Command::new("git")
                .args(["commit", "-m", "Update episode ID for publishing"])
                .spawn()?
                .wait()?;
            let _ = Command::new("git").args(["push"]).spawn()?.wait()?;

            log_event(&format!(
                "Draft episode: {} - {} for approval",
                new_video.id, new_video.title
            ))?;
        }

        // Loop every hour
        sleep(Duration::from_secs(60 * 60)).await;
    }
}

fn last_checked_video() -> String {
    let episode_json = fs::read_to_string("episode.json");
    let episode_data: Value =
        serde_json::from_str(&episode_json.unwrap()).expect("episode.json is blank");
    episode_data["id"].to_string()
}

fn parse_title(title: &str) -> String {
    let regex = Regex::new(r"^(.*?)(?: \| [^|]* \d{1,2}, \d{4})?$").unwrap();

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

        assert_eq!(video_entry.id, "mVda8IUcKEQ");
        assert_eq!(video_entry.title, "Sample Video Title");
    }

    #[test]
    fn test_parse_title() {
        let title_with_date = "The Spirit of Excellence | Pastor Bayo Fadugba | Celebration Service September 8, 2024";
        let title_without_date = "Unplugged Service | Celebration Service";

        assert_eq!(
            parse_title(title_with_date),
            "The Spirit of Excellence | Pastor Bayo Fadugba"
        );
        assert_eq!(
            parse_title(title_without_date),
            "Unplugged Service | Celebration Service"
        );

        let title_no_match = "This title has no date";
        assert_eq!(parse_title(title_no_match), "This title has no date");
    }
}
