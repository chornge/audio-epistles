mod publish;
mod video;

use publish::process_video;
use video::{fetch_new_video, last_seen_upload};

#[tokio::main]
async fn main() {
    match fetch_new_video().await {
        Ok(video_id) => {
            if video_id != last_seen_upload() {
                if let Err(e) = process_video(video_id).await {
                    eprintln!("Failed to process new video: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to fetch new video ID: {}", e);
        }
    }
}
