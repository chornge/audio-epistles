//! Episode processing module.
//!
//! This module handles downloading YouTube videos, extracting metadata,
//! identifying sermon chapters, and trimming audio files using ffmpeg.

use anyhow::{anyhow, Result};
use regex::Regex;
use std::process::Command;
use std::{env, fs};
use tokio::task;
use tracing::info;
use youtube_dl::{YoutubeDl, YoutubeDlOutput};

/// Downloads a YouTube video and extracts its metadata.
///
/// This function uses yt-dlp to download a video in MP4 format and extract
/// metadata including title, description, and duration. The video is saved
/// to the `assets/` directory and renamed to `video.mp4`. The title is
/// sanitized to keep at most 2 segments if separated by `|`.
///
/// # Arguments
///
/// * `video_id` - The YouTube video ID to download
///
/// # Returns
///
/// Returns a tuple containing:
/// - `title` - The sanitized video title (String)
/// - `description` - The video description (String)
/// - `video_path` - The absolute path to the downloaded video (String)
/// - `duration` - The video duration in seconds (u32)
///
/// # Errors
///
/// Returns an error if:
/// - The current directory cannot be determined
/// - The assets directory cannot be created
/// - The yt-dlp download fails
/// - No MP4 file is found after download
/// - The downloaded file cannot be renamed
/// - The output is a playlist instead of a single video
///
/// # Example
///
/// ```no_run
/// # tokio_test::block_on(async {
/// let (title, desc, path, duration) =
///     audio_epistles::episode::fetch_metadata("dQw4w9WgXcQ").await.unwrap();
/// println!("Downloaded: {} ({}s)", title, duration);
/// # })
/// ```
pub async fn fetch_metadata(video_id: &str) -> Result<(String, String, String, u32)> {
    let video_url = format!("https://www.youtube.com/watch?v={video_id}");
    info!(video_url = %video_url, "Downloading video and metadata");

    let project_root = env::current_dir()?;
    let downloads_dir = project_root.join("assets");

    if !downloads_dir.exists() {
        fs::create_dir_all(&downloads_dir)?;
    }

    let video_url_clone = video_url.clone();
    let downloads_dir_clone = downloads_dir.clone();

    // Spawn blocking because YoutubeDl is synchronous
    let output: YoutubeDlOutput = task::spawn_blocking(move || {
        let mut ytdl = YoutubeDl::new(&video_url_clone);
        ytdl.youtube_dl_path("yt-dlp");
        ytdl.extra_arg("--format");
        ytdl.extra_arg("mp4");

        // Keep original filename (temporarily)
        ytdl.download_to(&downloads_dir_clone)?;
        ytdl.run() // actually download + fetch metadata
    })
    .await??;

    // Rename downloaded file to "video.mp4"
    let mut downloaded_file = None;
    for entry in fs::read_dir(&downloads_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map(|ext| ext == "mp4").unwrap_or(false) {
            downloaded_file = Some(path);
            break;
        }
    }

    let downloaded_file = downloaded_file
        .ok_or_else(|| anyhow!("Downloaded file not found in {}", downloads_dir.display()))?;

    let final_video_path = downloads_dir.join("video.mp4");
    fs::rename(&downloaded_file, &final_video_path)?;

    match output {
        YoutubeDlOutput::SingleVideo(video) => {
            let raw_title = video.title.unwrap_or_else(|| "Untitled".to_string());

            // Sanitize title (keep at most 2 segments if '|' is present)
            let parts: Vec<String> = raw_title.split('|').map(|s| s.trim().to_string()).collect();

            // Take at most 2 parts, join with a normalized " | "
            let title = if parts.len() >= 2 {
                parts[..2].join(" | ")
            } else {
                parts[0].clone()
            };

            let desc = video.description.unwrap_or_default();

            // Borrow value to avoid moving
            let duration = video
                .duration
                .as_ref()
                .and_then(|d| d.as_u64())
                .unwrap_or(0) as u32;

            let path_str = final_video_path.to_string_lossy().to_string();
            info!(path = %path_str, "Downloaded video successfully");

            Ok((title, desc, path_str, duration))
        }
        _ => Err(anyhow!("Expected single video, got playlist.")),
    }
}

/// Parses timestamp string ("12:34" or "1:02:03") into seconds.
///
/// Returns 0 for invalid timestamp parts as a safe fallback. Since timestamps
/// come from video descriptions which may be user-edited and contain typos,
/// this graceful degradation is intentional - a malformed timestamp should not
/// cause the entire video processing to fail.
///
/// The `.unwrap_or(0)` is a deliberate design choice for robustness.
pub(crate) fn parse_timestamp(ts: &str) -> u32 {
    let parts: Vec<u32> = ts
        .split(':')
        .map(|p| p.parse::<u32>().unwrap_or(0))
        .collect();
    match parts.len() {
        3 => parts[0] * 3600 + parts[1] * 60 + parts[2],
        2 => parts[0] * 60 + parts[1],
        1 => parts[0],
        _ => 0,
    }
}

/// Extracts the sermon chapter timestamps from a video description.
///
/// This function parses the video description for YouTube-style chapters
/// (timestamp followed by title on each line). It looks for a chapter
/// containing the word "sermon" and returns its start and end timestamps.
/// If no chapters are found, it returns the full video length (0 to video_end).
///
/// # Arguments
///
/// * `description` - The video description text containing chapter markers
/// * `video_end` - The total duration of the video in seconds
///
/// # Returns
///
/// Returns `Some((start, end))` with timestamps in seconds if a sermon chapter
/// is found or if no chapters exist (returns full video). Returns `None` if
/// chapters exist but no sermon chapter is found.
///
/// # Example
///
/// ```
/// let description = "0:00 Welcome\n5:30 Worship\n15:00 Sermon\n45:00 Closing";
/// let result = audio_epistles::episode::extract_sermon_chapter(description, 3000);
/// assert_eq!(result, Some((900, 2700))); // 15:00 to 45:00
/// ```
pub fn extract_sermon_chapter(description: &str, video_end: u32) -> Option<(u32, u32)> {
    // Match all chapter lines: timestamp + title
    let re = Regex::new(r"(?m)^(\d{1,2}:\d{2}(?::\d{2})?)\s+(.+)$").ok()?;

    let mut chapters: Vec<(u32, String)> = re
        .captures_iter(description)
        .map(|cap| (parse_timestamp(&cap[1]), cap[2].to_lowercase()))
        .collect();

    // If no chapters → return full length
    if chapters.is_empty() {
        return Some((0, video_end));
    }

    chapters.sort_by_key(|c| c.0);

    // Find the first video chapter called "sermon"
    if let Some((i, (start, _))) = chapters
        .iter()
        .enumerate()
        .find(|(_, c)| c.1.contains("sermon"))
    {
        let end = if i + 1 < chapters.len() {
            chapters[i + 1].0 // next chapter
        } else {
            video_end // sermon is last chapter
        };
        return Some((*start, end));
    }

    None
}

/// Trims audio from a video file using ffmpeg and saves it as MP3.
///
/// This function extracts an audio segment from a video file, converts it to
/// MP3 format using the libmp3lame codec, and saves it to the specified output path.
/// The `-vn` flag is used to strip video, keeping only audio.
///
/// # Arguments
///
/// * `input` - Path to the input video file
/// * `output` - Path where the output MP3 file should be saved
/// * `start` - Start time in seconds for the audio segment
/// * `duration` - Duration in seconds of the audio segment to extract
///
/// # Returns
///
/// Returns `Ok(())` on success.
///
/// # Errors
///
/// Returns an error if:
/// - The ffmpeg command fails to execute
/// - The ffmpeg process exits with a non-zero status code
///
/// # Example
///
/// ```no_run
/// // Extract 30 minutes of audio starting at 15:00
/// audio_epistles::episode::trim_audio(
///     "assets/video.mp4",
///     "assets/audio.mp3",
///     900,  // 15 minutes
///     1800  // 30 minutes
/// ).unwrap();
/// ```
pub fn trim_audio(input: &str, output: &str, start: u32, duration: u32) -> Result<()> {
    let status = Command::new("ffmpeg")
        .args([
            "-y",
            "-i",
            input,
            "-ss",
            &start.to_string(),
            "-t",
            &duration.to_string(),
            "-vn",
            "-acodec",
            "libmp3lame",
            output,
        ])
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(anyhow!("ffmpeg trimming failed: {status}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_timestamp_minutes_seconds() {
        assert_eq!(parse_timestamp("12:34"), 754); // 12*60 + 34 = 754
        assert_eq!(parse_timestamp("1:02"), 62); // 1*60 + 2 = 62
        assert_eq!(parse_timestamp("0:30"), 30); // 30 seconds
        assert_eq!(parse_timestamp("59:59"), 3599); // 59*60 + 59 = 3599
    }

    #[test]
    fn test_parse_timestamp_hours_minutes_seconds() {
        assert_eq!(parse_timestamp("1:02:03"), 3723); // 1*3600 + 2*60 + 3 = 3723
        assert_eq!(parse_timestamp("0:01:30"), 90); // 1*60 + 30 = 90
        assert_eq!(parse_timestamp("2:30:45"), 9045); // 2*3600 + 30*60 + 45 = 9045
    }

    #[test]
    fn test_parse_timestamp_seconds_only() {
        assert_eq!(parse_timestamp("45"), 45);
        assert_eq!(parse_timestamp("0"), 0);
        assert_eq!(parse_timestamp("120"), 120);
    }

    #[test]
    fn test_parse_timestamp_invalid_inputs() {
        assert_eq!(parse_timestamp(""), 0); // Empty string
        assert_eq!(parse_timestamp("abc"), 0); // Non-numeric
        assert_eq!(parse_timestamp("12:abc"), 720); // 12*60 + 0 = 720
        assert_eq!(parse_timestamp("::"), 0); // Only colons
        assert_eq!(parse_timestamp("1:2:3:4"), 0); // Too many parts
    }

    #[test]
    fn test_extract_sermon_chapter_with_sermon() {
        let description = r#"
0:00 Introduction
5:30 Worship
15:45 Sermon
45:20 Closing Prayer
"#;
        let result = extract_sermon_chapter(description, 3000);
        assert_eq!(result, Some((945, 2720))); // Sermon starts at 15:45 (945s), ends at 45:20 (2720s)
    }

    #[test]
    fn test_extract_sermon_chapter_sermon_is_last() {
        let description = r#"
0:00 Introduction
5:30 Worship
15:45 Sermon
"#;
        let result = extract_sermon_chapter(description, 3000);
        assert_eq!(result, Some((945, 3000))); // Sermon starts at 15:45 (945s), ends at video_end (3000s)
    }

    #[test]
    fn test_extract_sermon_chapter_no_chapters() {
        let description = "This is a simple video description without any chapters.";
        let result = extract_sermon_chapter(description, 3000);
        assert_eq!(result, Some((0, 3000))); // Should return full video length
    }

    #[test]
    fn test_extract_sermon_chapter_no_sermon_chapter() {
        let description = r#"
0:00 Introduction
5:30 Worship
15:45 Announcements
45:20 Closing Prayer
"#;
        let result = extract_sermon_chapter(description, 3000);
        assert_eq!(result, None); // No sermon chapter found
    }

    #[test]
    fn test_extract_sermon_chapter_empty_description() {
        let description = "";
        let result = extract_sermon_chapter(description, 3000);
        assert_eq!(result, Some((0, 3000))); // Should return full video length
    }

    #[test]
    fn test_extract_sermon_chapter_case_insensitive() {
        let description = r#"
0:00 Introduction
5:30 SERMON
15:45 Closing
"#;
        let result = extract_sermon_chapter(description, 3000);
        assert_eq!(result, Some((330, 945))); // SERMON in uppercase should still match
    }

    #[test]
    fn test_extract_sermon_chapter_sermon_in_title() {
        let description = r#"
0:00 Pre-Sermon Worship
5:30 Main Sermon
15:45 Post-Sermon Discussion
"#;
        let result = extract_sermon_chapter(description, 3000);
        // Should find first chapter containing "sermon" (Pre-Sermon Worship)
        assert_eq!(result, Some((0, 330)));
    }

    #[test]
    fn test_extract_sermon_chapter_with_hour_format() {
        let description = r#"
0:00 Introduction
0:05:30 Worship
0:15:45 Sermon
1:02:30 Closing
"#;
        let result = extract_sermon_chapter(description, 4000);
        assert_eq!(result, Some((945, 3750))); // Sermon: 15:45 (945s) to 1:02:30 (3750s)
    }

    #[test]
    fn test_extract_sermon_chapter_unsorted_timestamps() {
        let description = r#"
15:45 Sermon
0:00 Introduction
45:20 Closing Prayer
5:30 Worship
"#;
        let result = extract_sermon_chapter(description, 3000);
        // Should still work correctly even if timestamps are not in order
        assert_eq!(result, Some((945, 2720)));
    }
}
