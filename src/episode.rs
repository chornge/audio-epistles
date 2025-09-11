use anyhow::{anyhow, Result};
use regex::Regex;
use std::process::Command;
use std::{env, fs};
use tokio::task;
use youtube_dl::{YoutubeDl, YoutubeDlOutput};

/// Download video + return (title, description, downloaded_path and duration)
pub async fn fetch_metadata(video_id: &str) -> Result<(String, String, String, u32)> {
    let video_url = format!("https://www.youtube.com/watch?v={video_id}");
    println!("🎥 Downloading video and metadata from {video_url}");

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

            println!("✅ Downloaded {}", final_video_path.display());

            Ok((
                title,
                desc,
                final_video_path.to_string_lossy().to_string(),
                duration,
            ))
        }
        _ => Err(anyhow!("Expected single video, got playlist.")),
    }
}

/// Parses timestamp string ("12:34" or "1:02:03") into seconds
fn parse_timestamp(ts: &str) -> u32 {
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

/// Extracts sermon chapter OR full length if no chapters exist.
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

/// Trim audio using ffmpeg and save as MP3
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
