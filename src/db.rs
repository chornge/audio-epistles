//! Database module for tracking processed video IDs.
//!
//! This module provides functions to initialize the SQLite database with migrations,
//! manage the storage of processed video IDs, and query upload history. It supports
//! schema versioning and automatic migration from older database schemas.

use anyhow::Result;
use sqlx::{Row, Sqlite, SqlitePool, Transaction};

/// Current schema version
const SCHEMA_VERSION: i32 = 1;

/// Represents an upload record in the database
#[derive(Debug)]
pub struct UploadRecord {
    pub id: i64,
    pub video_id: String,
    pub uploaded_at: String,
}

/// Initializes the database with schema migrations.
///
/// This function sets up the database schema and handles migrations from older versions.
/// It creates a `schema_version` table to track the current schema version and applies
/// migrations as needed. If an old `uploaded` table exists from a previous version,
/// it migrates the data to the new `uploads` table structure.
///
/// The current schema (v1) includes:
/// - `uploads` table with auto-incrementing ID, video_id, and timestamp
/// - Index on video_id for faster lookups
/// - `schema_version` table to track migrations
///
/// # Arguments
///
/// * `pool` - A reference to the SQLite connection pool
///
/// # Returns
///
/// Returns `Ok(())` on success.
///
/// # Errors
///
/// Returns an error if:
/// - The schema_version table cannot be created
/// - The migration to v1 fails
/// - Any database query fails during initialization
///
/// # Example
///
/// ```no_run
/// # use sqlx::SqlitePool;
/// # tokio_test::block_on(async {
/// let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
/// audio_epistles::db::init(&pool).await.unwrap();
/// # })
/// ```
pub async fn init(pool: &SqlitePool) -> Result<()> {
    // Create schema_version table if it doesn't exist
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY
        )",
    )
    .execute(pool)
    .await?;

    // Get current schema version
    let current_version = get_schema_version(pool).await?;

    if current_version == 0 {
        // Fresh install or old schema - run migration
        migrate_to_v1(pool).await?;
    }

    Ok(())
}

/// Get the current schema version
async fn get_schema_version(pool: &SqlitePool) -> Result<i32> {
    let row = sqlx::query("SELECT version FROM schema_version LIMIT 1")
        .fetch_optional(pool)
        .await?;

    if let Some(row) = row {
        let version: i32 = row.try_get("version")?;
        Ok(version)
    } else {
        Ok(0) // No version = fresh install or old schema
    }
}

/// Migrate to schema version 1
async fn migrate_to_v1(pool: &SqlitePool) -> Result<()> {
    let mut tx = pool.begin().await?;

    // Check if old 'uploaded' table exists
    let old_table_exists: bool = sqlx::query_scalar(
        "SELECT COUNT(*) > 0 FROM sqlite_master
         WHERE type='table' AND name='uploaded'",
    )
    .fetch_one(&mut *tx)
    .await?;

    // Create new uploads table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS uploads (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            video_id TEXT NOT NULL,
            uploaded_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(&mut *tx)
    .await?;

    // Create index on video_id for faster lookups
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_uploads_video_id ON uploads(video_id)",
    )
    .execute(&mut *tx)
    .await?;

    // Migrate data from old table if it exists
    if old_table_exists {
        // Get the old video ID
        let old_id: Option<String> = sqlx::query_scalar("SELECT id FROM uploaded LIMIT 1")
            .fetch_optional(&mut *tx)
            .await?;

        if let Some(video_id) = old_id {
            // Insert it into the new table
            sqlx::query("INSERT INTO uploads (video_id) VALUES (?)")
                .bind(&video_id)
                .execute(&mut *tx)
                .await?;
        }

        // Drop the old table
        sqlx::query("DROP TABLE uploaded")
            .execute(&mut *tx)
            .await?;
    }

    // Set schema version
    sqlx::query("INSERT OR REPLACE INTO schema_version (version) VALUES (?)")
        .bind(SCHEMA_VERSION)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(())
}

/// Retrieves the most recently processed video ID from the database.
///
/// This function queries the uploads table for the latest video ID, ordered by
/// the auto-incrementing ID field. On the first run when the database is empty,
/// it returns an empty string.
///
/// # Arguments
///
/// * `pool` - A reference to the SQLite connection pool
///
/// # Returns
///
/// Returns the last video ID as a `String`. Returns an empty string if no
/// video has been processed yet.
///
/// # Errors
///
/// Returns an error if:
/// - The database query fails to execute
/// - The video_id column cannot be extracted from the result row
///
/// # Example
///
/// ```no_run
/// # use sqlx::SqlitePool;
/// # tokio_test::block_on(async {
/// let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
/// let last_id = audio_epistles::db::get_last_id(&pool).await.unwrap();
/// println!("Last processed video: {}", last_id);
/// # })
/// ```
pub async fn get_last_id(pool: &SqlitePool) -> Result<String> {
    let row = sqlx::query("SELECT video_id FROM uploads ORDER BY id DESC LIMIT 1")
        .fetch_optional(pool)
        .await?;

    if let Some(row) = row {
        let id: String = row.try_get("video_id")?;
        Ok(id)
    } else {
        Ok(String::new()) // First run — no ID yet
    }
}

/// Saves a video ID to the database within a transaction.
///
/// This function inserts a new video ID into the uploads table along with
/// an automatic timestamp. Unlike the old schema which used INSERT OR REPLACE
/// to maintain a single record, this version keeps a full history of all uploads.
/// The operation is performed within a transaction to ensure atomicity.
///
/// # Arguments
///
/// * `tx` - A mutable reference to the database transaction
/// * `id` - The video ID string to save
///
/// # Returns
///
/// Returns `Ok(())` on success.
///
/// # Errors
///
/// Returns an error if the INSERT query fails to execute.
///
/// # Example
///
/// ```no_run
/// # use sqlx::SqlitePool;
/// # tokio_test::block_on(async {
/// let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
/// let mut tx = pool.begin().await.unwrap();
/// audio_epistles::db::save_id(&mut tx, "dQw4w9WgXcQ").await.unwrap();
/// tx.commit().await.unwrap();
/// # })
/// ```
pub async fn save_id(tx: &mut Transaction<'_, Sqlite>, id: &str) -> Result<()> {
    sqlx::query("INSERT INTO uploads (video_id) VALUES (?)")
        .bind(id)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

/// Retrieves upload history with a specified limit.
///
/// This function fetches the most recent upload records from the database,
/// ordered by ID in descending order (most recent first). Each record includes
/// the database ID, video ID, and upload timestamp.
///
/// # Arguments
///
/// * `pool` - A reference to the SQLite connection pool
/// * `limit` - The maximum number of records to retrieve
///
/// # Returns
///
/// Returns a vector of `UploadRecord` structs containing upload history.
///
/// # Errors
///
/// Returns an error if:
/// - The database query fails to execute
/// - Any column cannot be extracted from the result rows
///
/// # Example
///
/// ```no_run
/// # use sqlx::SqlitePool;
/// # tokio_test::block_on(async {
/// let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
/// let history = audio_epistles::db::get_upload_history(&pool, 10).await.unwrap();
/// for record in history {
///     println!("{}: {} at {}", record.id, record.video_id, record.uploaded_at);
/// }
/// # })
/// ```
pub async fn get_upload_history(pool: &SqlitePool, limit: u32) -> Result<Vec<UploadRecord>> {
    let rows = sqlx::query(
        "SELECT id, video_id, uploaded_at FROM uploads
         ORDER BY id DESC LIMIT ?",
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    let mut records = Vec::new();
    for row in rows {
        records.push(UploadRecord {
            id: row.try_get("id")?,
            video_id: row.try_get("video_id")?,
            uploaded_at: row.try_get("uploaded_at")?,
        });
    }

    Ok(records)
}

/// Checks if a video ID has already been uploaded.
///
/// This function queries the uploads table to determine if a specific video ID
/// has been processed before. This can be useful for preventing duplicate uploads
/// or checking processing status.
///
/// # Arguments
///
/// * `pool` - A reference to the SQLite connection pool
/// * `video_id` - The video ID to check
///
/// # Returns
///
/// Returns `true` if the video has been uploaded before, `false` otherwise.
///
/// # Errors
///
/// Returns an error if the database query fails to execute.
///
/// # Example
///
/// ```no_run
/// # use sqlx::SqlitePool;
/// # tokio_test::block_on(async {
/// let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
/// let is_uploaded = audio_epistles::db::is_video_uploaded(&pool, "dQw4w9WgXcQ").await.unwrap();
/// if is_uploaded {
///     println!("Video already processed!");
/// }
/// # })
/// ```
pub async fn is_video_uploaded(pool: &SqlitePool, video_id: &str) -> Result<bool> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM uploads WHERE video_id = ?")
        .bind(video_id)
        .fetch_one(pool)
        .await?;

    Ok(count > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_test_db() -> SqlitePool {
        // Create an in-memory SQLite database for testing
        SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create test database")
    }

    #[tokio::test]
    async fn test_init_creates_tables() {
        let pool = setup_test_db().await;

        // Initialize the database
        let result = init(&pool).await;
        assert!(result.is_ok());

        // Verify the schema_version table exists
        let schema_version_exists: bool = sqlx::query_scalar(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='schema_version'",
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to check schema_version table");

        assert!(schema_version_exists);

        // Verify the uploads table exists
        let uploads_exists: bool = sqlx::query_scalar(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='uploads'",
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to check uploads table");

        assert!(uploads_exists);

        // Verify schema version is set correctly
        let version = get_schema_version(&pool)
            .await
            .expect("Failed to get schema version");
        assert_eq!(version, SCHEMA_VERSION);
    }

    #[tokio::test]
    async fn test_get_last_id_empty_database() {
        let pool = setup_test_db().await;
        init(&pool).await.expect("Failed to init database");

        // Get last ID from empty database
        let result = get_last_id(&pool).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ""); // Should return empty string
    }

    #[tokio::test]
    async fn test_save_and_get_id() {
        let pool = setup_test_db().await;
        init(&pool).await.expect("Failed to init database");

        // Start a transaction
        let mut tx = pool.begin().await.expect("Failed to begin transaction");

        // Save an ID
        let test_id = "test_video_123";
        let save_result = save_id(&mut tx, test_id).await;
        assert!(save_result.is_ok());

        // Commit the transaction
        tx.commit().await.expect("Failed to commit transaction");

        // Retrieve the ID
        let get_result = get_last_id(&pool).await;
        assert!(get_result.is_ok());
        assert_eq!(get_result.unwrap(), test_id);
    }

    #[tokio::test]
    async fn test_save_multiple_ids_returns_most_recent() {
        let pool = setup_test_db().await;
        init(&pool).await.expect("Failed to init database");

        let video_ids = vec!["video1", "video2", "video3"];

        for video_id in &video_ids {
            let mut tx = pool.begin().await.expect("Failed to begin transaction");
            save_id(&mut tx, video_id)
                .await
                .expect("Failed to save ID");
            tx.commit().await.expect("Failed to commit transaction");
        }

        // Get last ID should return the most recent
        let result = get_last_id(&pool).await.expect("Failed to get last ID");
        assert_eq!(result, "video3");
    }

    #[tokio::test]
    async fn test_transaction_rollback() {
        let pool = setup_test_db().await;
        init(&pool).await.expect("Failed to init database");

        // Start a transaction and save an ID but don't commit
        let mut tx = pool.begin().await.expect("Failed to begin transaction");
        save_id(&mut tx, "uncommitted_id")
            .await
            .expect("Failed to save ID");
        // Explicitly drop the transaction without committing (simulates rollback)
        drop(tx);

        // Verify the ID was not saved
        let result = get_last_id(&pool).await.expect("Failed to get ID");
        assert_eq!(result, ""); // Should still be empty
    }

    #[tokio::test]
    async fn test_is_video_uploaded() {
        let pool = setup_test_db().await;
        init(&pool).await.expect("Failed to init database");

        let test_id = "test_video_456";

        // Initially should not be uploaded
        let is_uploaded = is_video_uploaded(&pool, test_id)
            .await
            .expect("Failed to check if uploaded");
        assert!(!is_uploaded);

        // Save the ID
        let mut tx = pool.begin().await.expect("Failed to begin transaction");
        save_id(&mut tx, test_id).await.expect("Failed to save ID");
        tx.commit().await.expect("Failed to commit transaction");

        // Now should be uploaded
        let is_uploaded = is_video_uploaded(&pool, test_id)
            .await
            .expect("Failed to check if uploaded");
        assert!(is_uploaded);
    }

    #[tokio::test]
    async fn test_get_upload_history() {
        let pool = setup_test_db().await;
        init(&pool).await.expect("Failed to init database");

        // Save multiple IDs
        let video_ids = vec!["video1", "video2", "video3", "video4", "video5"];
        for video_id in &video_ids {
            let mut tx = pool.begin().await.expect("Failed to begin transaction");
            save_id(&mut tx, video_id)
                .await
                .expect("Failed to save ID");
            tx.commit().await.expect("Failed to commit transaction");
        }

        // Get history with limit
        let history = get_upload_history(&pool, 3)
            .await
            .expect("Failed to get upload history");

        assert_eq!(history.len(), 3);
        // Should be in reverse chronological order (most recent first)
        assert_eq!(history[0].video_id, "video5");
        assert_eq!(history[1].video_id, "video4");
        assert_eq!(history[2].video_id, "video3");
    }

    #[tokio::test]
    async fn test_get_upload_history_empty() {
        let pool = setup_test_db().await;
        init(&pool).await.expect("Failed to init database");

        // Get history from empty database
        let history = get_upload_history(&pool, 10)
            .await
            .expect("Failed to get upload history");

        assert_eq!(history.len(), 0);
    }

    #[tokio::test]
    async fn test_migration_from_old_schema() {
        let pool = setup_test_db().await;

        // Create old schema manually
        sqlx::query(
            "CREATE TABLE uploaded (
                id TEXT PRIMARY KEY
            )",
        )
        .execute(&pool)
        .await
        .expect("Failed to create old table");

        // Insert old data
        sqlx::query("INSERT INTO uploaded (id) VALUES (?)")
            .bind("old_video_id")
            .execute(&pool)
            .await
            .expect("Failed to insert old data");

        // Run init (should trigger migration)
        init(&pool).await.expect("Failed to init database");

        // Verify old table is gone
        let old_table_exists: bool = sqlx::query_scalar(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='uploaded'",
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to check old table");

        assert!(!old_table_exists);

        // Verify new table exists
        let new_table_exists: bool = sqlx::query_scalar(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='uploads'",
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to check new table");

        assert!(new_table_exists);

        // Verify data was migrated
        let last_id = get_last_id(&pool).await.expect("Failed to get last ID");
        assert_eq!(last_id, "old_video_id");
    }

    #[tokio::test]
    async fn test_duplicate_video_ids_allowed() {
        let pool = setup_test_db().await;
        init(&pool).await.expect("Failed to init database");

        let test_id = "duplicate_video";

        // Save the same ID twice
        for _ in 0..2 {
            let mut tx = pool.begin().await.expect("Failed to begin transaction");
            save_id(&mut tx, test_id).await.expect("Failed to save ID");
            tx.commit().await.expect("Failed to commit transaction");
        }

        // Verify both records exist
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM uploads WHERE video_id = ?")
            .bind(test_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to count records");

        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_upload_record_has_timestamp() {
        let pool = setup_test_db().await;
        init(&pool).await.expect("Failed to init database");

        // Save an ID
        let mut tx = pool.begin().await.expect("Failed to begin transaction");
        save_id(&mut tx, "timestamped_video")
            .await
            .expect("Failed to save ID");
        tx.commit().await.expect("Failed to commit transaction");

        // Get the record
        let history = get_upload_history(&pool, 1)
            .await
            .expect("Failed to get upload history");

        assert_eq!(history.len(), 1);
        assert_eq!(history[0].video_id, "timestamped_video");
        // Verify timestamp is not empty
        assert!(!history[0].uploaded_at.is_empty());
    }

    #[tokio::test]
    async fn test_index_exists() {
        let pool = setup_test_db().await;
        init(&pool).await.expect("Failed to init database");

        // Verify index was created
        let index_exists: bool = sqlx::query_scalar(
            "SELECT COUNT(*) > 0 FROM sqlite_master
             WHERE type='index' AND name='idx_uploads_video_id'",
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to check index");

        assert!(index_exists);
    }
}
