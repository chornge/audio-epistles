mod db;
mod episode;
mod processor;
mod video;
mod webdriver;

use anyhow::Result;
use db::{get_last_id, save_id};
use dotenvy::dotenv;
use processor::process;
use sqlx::SqlitePool;
use std::env;
use std::time::Instant;
use video::fetch_video;

#[tokio::main]
async fn main() -> Result<()> {
    let timer = Instant::now();

    dotenv().ok();

    let db_url = env::var("DB_URL")?;
    let pool = SqlitePool::connect(&db_url).await?;

    db::init(&pool).await?;

    let last_seen_id = get_last_id(&pool).await?;

    match fetch_video().await {
        Ok(video_id) => {
            if video_id != last_seen_id {
                if let Err(e) = process(&video_id).await {
                    eprintln!("❌ Failed to process new video: {e}");
                } else {
                    let mut transaction = pool.begin().await?;
                    save_id(&mut transaction, &video_id).await?;
                    transaction.commit().await?;
                    println!("✅ Updated DB with video ID: {video_id}");
                }
            } else {
                println!("No new video found since last publish.");
            }
        }
        Err(e) => eprintln!("❌ Failed to fetch new video ID: {e}"),
    }

    let duration = timer.elapsed();
    let total_seconds = duration.as_secs();
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;

    let formatted = if minutes > 0 {
        format!("{minutes}min {seconds}sec")
    } else {
        format!("{seconds}sec")
    };

    println!("Finished in {formatted}");

    Ok(())
}
