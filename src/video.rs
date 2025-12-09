//! YouTube video fetching module.
//!
//! This module provides functionality to fetch the latest video ID from a
//! YouTube playlist by scraping the playlist page HTML.

use anyhow::{Context, Result};
use dotenvy::dotenv;
use regex::Regex;
use reqwest::get;
use std::env;
use tracing::info;

/// Fetches the latest video ID from a YouTube playlist.
///
/// This function scrapes the YouTube playlist page HTML and extracts the most
/// recent video ID using regex pattern matching. The playlist ID is read from
/// the `SERMON_PLAYLIST_ID` environment variable. The function returns the last
/// video ID found in the playlist response, which corresponds to the most recently
/// added video.
///
/// # Returns
///
/// Returns the video ID as a `String` on success.
///
/// # Errors
///
/// Returns an error if:
/// - The `SERMON_PLAYLIST_ID` environment variable is not set
/// - The HTTP request to YouTube fails
/// - The response body cannot be parsed
/// - No video ID is found in the playlist HTML
///
/// # Example
///
/// ```no_run
/// # tokio_test::block_on(async {
/// // Ensure SERMON_PLAYLIST_ID is set in .env file
/// let video_id = audio_epistles::video::fetch_video().await.unwrap();
/// println!("Latest video: {}", video_id);
/// # })
/// ```
pub async fn fetch_video() -> Result<String> {
    dotenv().ok();
    let playlist_id = env::var("SERMON_PLAYLIST_ID")
        .context("SERMON_PLAYLIST_ID environment variable not set")?;
    let playlist_url = format!("https://www.youtube.com/playlist?list={playlist_id}");

    let body = get(&playlist_url).await?.text().await?;
    // Regex pattern is a compile-time constant and will never fail to compile
    let re = Regex::new(r#""videoId":"([^"]+)""#).unwrap();
    let video_id = re.captures_iter(&body).filter_map(|cap| cap.get(1)).last();

    match video_id {
        Some(id) => {
            let id_str = id.as_str().to_string();
            info!(video_id = %id_str, "Latest video ID found");
            Ok(id_str)
        }
        None => Err(anyhow::anyhow!("No video ID found in playlist")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_id_regex_pattern() {
        let re = Regex::new(r#""videoId":"([^"]+)""#).unwrap();

        // Test with sample YouTube API response
        let sample_response = r#"{"videoId":"dQw4w9WgXcQ","title":"Test Video"}"#;
        let captures: Vec<_> = re
            .captures_iter(sample_response)
            .filter_map(|cap| cap.get(1))
            .map(|m| m.as_str())
            .collect();

        assert_eq!(captures.len(), 1);
        assert_eq!(captures[0], "dQw4w9WgXcQ");
    }

    #[test]
    fn test_video_id_regex_multiple_videos() {
        let re = Regex::new(r#""videoId":"([^"]+)""#).unwrap();

        // Test with multiple video IDs (simulating playlist response)
        let sample_response = r#"
            {"videoId":"video1_abc123","title":"Video 1"}
            {"videoId":"video2_def456","title":"Video 2"}
            {"videoId":"video3_ghi789","title":"Video 3"}
        "#;

        let captures: Vec<_> = re
            .captures_iter(sample_response)
            .filter_map(|cap| cap.get(1))
            .map(|m| m.as_str())
            .collect();

        assert_eq!(captures.len(), 3);
        assert_eq!(captures[0], "video1_abc123");
        assert_eq!(captures[1], "video2_def456");
        assert_eq!(captures[2], "video3_ghi789");
    }

    #[test]
    fn test_video_id_regex_last_video() {
        let re = Regex::new(r#""videoId":"([^"]+)""#).unwrap();

        // Test that we can get the last video ID (as fetch_video does)
        let sample_response = r#"
            {"videoId":"old_video1","title":"Old Video 1"}
            {"videoId":"old_video2","title":"Old Video 2"}
            {"videoId":"latest_video","title":"Latest Video"}
        "#;

        let last_video = re
            .captures_iter(sample_response)
            .filter_map(|cap| cap.get(1))
            .last();

        assert!(last_video.is_some());
        assert_eq!(last_video.unwrap().as_str(), "latest_video");
    }

    #[test]
    fn test_video_id_regex_no_match() {
        let re = Regex::new(r#""videoId":"([^"]+)""#).unwrap();

        // Test with response that doesn't contain videoId
        let sample_response = r#"{"playlistId":"PLtest123","title":"Playlist"}"#;

        let captures: Vec<_> = re
            .captures_iter(sample_response)
            .filter_map(|cap| cap.get(1))
            .collect();

        assert_eq!(captures.len(), 0);
    }

    #[test]
    fn test_video_id_regex_various_formats() {
        let re = Regex::new(r#""videoId":"([^"]+)""#).unwrap();

        // Test various YouTube video ID formats
        let test_cases = vec![
            (r#""videoId":"dQw4w9WgXcQ""#, "dQw4w9WgXcQ"), // 11 chars (standard)
            (r#""videoId":"abc-_123456""#, "abc-_123456"), // with dash and underscore
            (r#""videoId":"X_yZ_12-34A""#, "X_yZ_12-34A"), // mixed case and symbols
        ];

        for (input, expected) in test_cases {
            let capture = re
                .captures(input)
                .and_then(|cap| cap.get(1))
                .map(|m| m.as_str());

            assert_eq!(capture, Some(expected));
        }
    }

    #[test]
    fn test_video_id_regex_nested_json() {
        let re = Regex::new(r#""videoId":"([^"]+)""#).unwrap();

        // Test with more realistic nested JSON structure (YouTube uses "key":"value" format without spaces)
        let sample_response = r#"
        {
            "contents": {
                "twoColumnBrowseResultsRenderer": {
                    "tabs": [{
                        "tabRenderer": {
                            "content": {
                                "sectionListRenderer": {
                                    "contents": [{
                                        "itemSectionRenderer": {
                                            "contents": [{
                                                "playlistVideoRenderer": {
                                                    "videoId":"realVideoId123"
                                                }
                                            }]
                                        }
                                    }]
                                }
                            }
                        }
                    }]
                }
            }
        }
        "#;

        let captures: Vec<_> = re
            .captures_iter(sample_response)
            .filter_map(|cap| cap.get(1))
            .map(|m| m.as_str())
            .collect();

        assert_eq!(captures.len(), 1);
        assert_eq!(captures[0], "realVideoId123");
    }
}
