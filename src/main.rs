mod db;
mod episode;
mod processor;
mod video;
mod webdriver;

use anyhow::Result;
use db::{last_seen_video_id, save_video_id};
use dotenvy::dotenv;
use processor::process_video;
use sqlx::SqlitePool;
use std::env;
use std::time::Instant;
use video::fetch_new_video;

#[tokio::main]
async fn main() -> Result<()> {
    let timer = Instant::now();

    dotenv().ok();

    let db_url = env::var("DB_URL")?;
    let pool = SqlitePool::connect(&db_url).await?;

    db::init_db(&pool).await?;

    let last_seen_id = last_seen_video_id(&pool).await?;

    match fetch_new_video().await {
        Ok(video_id) => {
            if video_id != last_seen_id {
                if let Err(e) = process_video(&video_id).await {
                    eprintln!("❌ Failed to process new video: {e}");
                } else {
                    let mut transaction = pool.begin().await?;
                    save_video_id(&mut transaction, &video_id).await?;
                    transaction.commit().await?;
                    println!("✅ Updated DB with video ID: {video_id}");
                }
            } else {
                println!("No new video found since last publish.");
            }
        }
        Err(e) => eprintln!("❌ Failed to fetch new video ID: {e}"),
    }

    println!("🚀 App ran in: {:?}", timer.elapsed());

    Ok(())
}
