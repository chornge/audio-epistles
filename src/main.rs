use anyhow::{anyhow, Result};

use std::env;
use tokio::time::{sleep, Duration};

mod publish;
mod video;

use publish::process_new_videos;
use video::{fetch_new_videos, VideoEntry};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    load_environment_variables()?;

    let sermon_playlist_id =
        env::var("SERMON_PLAYLIST_ID").map_err(|_| anyhow!("SERMON_PLAYLIST_ID not set"))?;

    loop {
        let new_videos = fetch_new_videos(&sermon_playlist_id).await?;
        process_new_videos(new_videos).await?;

        // Loop every hour (or use cron job)
        sleep(Duration::from_secs(60 * 60)).await;
    }
}

fn load_environment_variables() -> Result<()> {
    let _ = env::var("ANCHOR_EMAIL").map_err(|_| anyhow!("ANCHOR_EMAIL not set"))?;
    let _ = env::var("ANCHOR_PASSWORD").map_err(|_| anyhow!("ANCHOR_PASSWORD not set"))?;
    let _ = env::var("SPOTIFY_EMAIL").map_err(|_| anyhow!("SPOTIFY_EMAIL not set"))?;
    let _ = env::var("SPOTIFY_PASSWORD").map_err(|_| anyhow!("SPOTIFY_PASSWORD not set"))?;
    Ok(())
}
