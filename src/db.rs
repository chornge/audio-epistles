use anyhow::Result;
use sqlx::{Row, Sqlite, SqlitePool, Transaction};

pub async fn init_db(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS published (
            id TEXT PRIMARY KEY
        )",
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn last_seen_video_id(pool: &SqlitePool) -> Result<String> {
    let row = sqlx::query("SELECT id FROM published LIMIT 1")
        .fetch_optional(pool)
        .await?;

    if let Some(row) = row {
        let id: String = row.try_get("id")?;
        Ok(id)
    } else {
        Ok(String::new()) // First run — no ID yet
    }
}

pub async fn save_video_id(tx: &mut Transaction<'_, Sqlite>, id: &str) -> Result<()> {
    sqlx::query("INSERT OR REPLACE INTO published (id) VALUES (?)")
        .bind(id)
        .execute(&mut **tx)
        .await?;
    Ok(())
}
