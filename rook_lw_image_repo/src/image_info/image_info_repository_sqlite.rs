use super::ImageInfoRepository;
use crate::ImageRepoResult;

use rook_lw_models::image::{Detection, MotionDetectionScore, ImageInfo};

use rusqlite::Row;
use tracing::info;
use rusqlite::{params};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use serde_json;
use chrono::Local;

pub struct ImageInfoRepositorySqlite {
    pool: Pool<SqliteConnectionManager>,
}

impl ImageInfoRepositorySqlite {
    
    pub fn new(pool: Pool<SqliteConnectionManager>) -> ImageRepoResult<Self> {
        let mut _self = Self { pool };
        _self.initialize()?;
        Ok(_self)
    }

    pub fn new_from_path(db_path: &str) -> ImageRepoResult<Self> {
        ImageInfoRepositorySqlite::new(
            ImageInfoRepositorySqlite::create_pool(db_path)?
        )
    }

    fn row_to_image_info(row: &Row, image_id: &str) -> ImageRepoResult<ImageInfo> {
        let event_id: String = row.get(0)?;
        let event_timestamp: String = row.get(1)?;
        let motion_score_json: String = row.get(2)?;
        let detections_json: String = row.get(3)?;
        let capture_index: u32 = row.get(4)?;
        let capture_timestamp: String = row.get(5)?;
        let image_path: String = row.get(6)?;

        let motion_score: MotionDetectionScore = serde_json::from_str(&motion_score_json)?;
        let detections: Option<Vec<Detection>> = serde_json::from_str(&detections_json)?;
        let event_timestamp = chrono::DateTime::parse_from_rfc3339(&event_timestamp)?.with_timezone(&Local);
        let capture_timestamp = chrono::DateTime::parse_from_rfc3339(&capture_timestamp)?.with_timezone(&Local);

        Ok(ImageInfo {
            image_id: image_id.to_string(),
            event_id,
            event_timestamp,
            motion_score,
            capture_index,
            capture_timestamp,
            detections,
            image_path,
        })
    }

    fn create_pool(db_path: &str) -> ImageRepoResult<Pool<SqliteConnectionManager>> {
        // Ensure parent directory exists
        if let Some(parent) = std::path::Path::new(db_path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Create connection manager and ensure connections use WAL journal mode.
        // This improves concurrency for reads and writes.
        let manager = SqliteConnectionManager::file(db_path)
            .with_init(|c| {
                c.pragma_update(None, "journal_mode", &"WAL")?;
                Ok(())
            });

        let pool = Pool::new(manager)?;
        Ok(pool)
    }

    fn initialize(&mut self) -> ImageRepoResult<()> {
        let conn = self.pool.get()?;
        info!("Initializing image_info_repository database");
        conn.execute_batch(r#"
            CREATE TABLE IF NOT EXISTS image_info (
                image_id TEXT PRIMARY KEY,
                event_id TEXT NOT NULL,
                event_timestamp TEXT NOT NULL,
                motion_score TEXT NOT NULL,
                detections TEXT NOT NULL,
                capture_index INTEGER NOT NULL,
                capture_timestamp TEXT NOT NULL,
                image_path TEXT NOT NULL
            );
        "#)?;
        Ok(())
    }
}

impl ImageInfoRepository for ImageInfoRepositorySqlite {

    fn save_image_info(&self, info: &ImageInfo) -> ImageRepoResult<()> {
        let motion_score_json = serde_json::to_string(&info.motion_score)?;
        let detections_json = serde_json::to_string(&info.detections)?;
        let conn = self.pool.get()?;
        conn.execute(
            r#"INSERT INTO image_info (
                image_id, event_id, event_timestamp, motion_score, detections, capture_index, capture_timestamp, image_path
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(image_id) DO UPDATE SET
                event_id=excluded.event_id,
                event_timestamp=excluded.event_timestamp,
                motion_score=excluded.motion_score,
                detections=excluded.detections,
                capture_index=excluded.capture_index,
                capture_timestamp=excluded.capture_timestamp,
                image_path=excluded.image_path
            "#,
            params![
                &info.image_id,
                &info.event_id,
                info.event_timestamp.to_rfc3339(),
                motion_score_json,
                detections_json,
                info.capture_index,
                info.capture_timestamp.to_rfc3339(),
                &info.image_path,
            ],
        )?;
        Ok(())
    }

    fn get_image_info(&self, image_id: &str) -> ImageRepoResult<Option<ImageInfo>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            r#"SELECT event_id, event_timestamp, motion_score, detections, capture_index, capture_timestamp, image_path
               FROM image_info WHERE image_id = ?1"#
        )?;
        let mut rows = stmt.query(params![image_id])?;
        if let Some(row_result) = rows.next()? {
            Ok(Some(Self::row_to_image_info(&row_result, image_id)?))
        } else {
            Ok(None)
        }
    }
}