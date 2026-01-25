use super::ImageInfoRepository;
use crate::ImageRepoResult;

use rook_lw_models::image::{Detection, MotionDetectionScore, ImageInfo, ImageInfoSearchOptions};

use rusqlite::Row;
use tracing::{debug, info};
use rusqlite::{params, ToSql};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use serde_json;

pub struct ImageInfoRepositorySqlite {
    pool: Pool<SqliteConnectionManager>,
}

impl ImageInfoRepositorySqlite {
    
    pub fn new(pool: Pool<SqliteConnectionManager>) -> ImageRepoResult<Self> {
        let mut _self = Self { pool };
        _self.initialize()?;
        Ok(_self)
    }

    fn row_to_image_info(row: &Row) -> ImageRepoResult<ImageInfo> {
        let image_id: String = row.get(0)?;
        let event_id: String = row.get(1)?;
        let event_timestamp: String = row.get(2)?;
        let motion_score_json: String = row.get(3)?;
        let detections_json: String = row.get(4)?;
        let capture_index: u32 = row.get(5)?;
        let capture_timestamp: String = row.get(6)?;
        let image_path: String = row.get(7)?;

        let motion_score: MotionDetectionScore = serde_json::from_str(&motion_score_json)?;
        let detections: Option<Vec<Detection>> = serde_json::from_str(&detections_json)?;
        let event_timestamp = chrono::DateTime::parse_from_rfc3339(&event_timestamp)?;
        let capture_timestamp = chrono::DateTime::parse_from_rfc3339(&capture_timestamp)?;

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
            CREATE INDEX IF NOT EXISTS idx_event_timestamp_dt ON image_info(datetime(event_timestamp));
            CREATE INDEX IF NOT EXISTS idx_capture_timestamp_dt ON image_info(datetime(capture_timestamp));
        "#)?;
        Ok(())
    }

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
            r#"SELECT image_id, event_id, event_timestamp, motion_score, detections, capture_index, capture_timestamp, image_path
               FROM image_info WHERE image_id = ?1"#
        )?;
        let mut rows = stmt.query(params![image_id])?;
        if let Some(row_result) = rows.next()? {
            Ok(Some(Self::row_to_image_info(&row_result)?))
        } else {
            Ok(None)
        }
    }

    fn search_image_info(
        &self,
        options: &ImageInfoSearchOptions,
        ) -> ImageRepoResult<Vec<ImageInfo>>
    {
        let conn = self.pool.get()?;

        info!("In search image info!!!!!");

        // Base query selection
        let mut query = String::new();
        query.push_str("SELECT\n");
        query.push_str("  image_id, event_id, event_timestamp, motion_score,\n");
        query.push_str("  detections, capture_index, capture_timestamp, image_path\n");
        query.push_str("FROM image_info AS ii_outer\n");
        query.push_str("WHERE 1=1\n");

        // Hold query parameters
        let mut params_vec: Vec<Box<dyn ToSql>> = Vec::new();

        // capture_timestamp range check
        if let Some(start_dt) = &options.start_date {
            query.push_str("  AND datetime(capture_timestamp) >= datetime(?1)\n");
            params_vec.push(Box::new(start_dt.to_rfc3339()));
        }

        // capture_timestamp range check
        if let Some(end_dt) = &options.end_date {
            query.push_str("  AND datetime(capture_timestamp) <= datetime(?2)\n");
            params_vec.push(Box::new(end_dt.to_rfc3339()));
        }

        // Critera on detections
        query.push_str("  AND EXISTS (\n");
        query.push_str("    SELECT image_id\n");
        query.push_str("    FROM image_info AS ii_inner, json_each(ii_inner.detections) as detection\n");
        query.push_str("    WHERE ii_outer.image_id = ii_inner.image_id\n");

        // Build up detection class name criteria
        if options.detection_classes.len() > 0 {
            query.push_str("      AND json_extract(detection.value, '$.class_name') IN (");
            for (idx, class_name) in options.detection_classes.iter().enumerate() {
                if idx > 0 {
                    query += ",";
                }
                query += "?";
                params_vec.push(Box::new(class_name));
            }
            query += ")\n";
        }

        // detection class confidence
        if let Some(confidence) = options.detection_class_confidence {
            query.push_str("      AND json_extract(detection.value, '$.confidence') >= ?\n");
            params_vec.push(Box::new(confidence));
        }

        // End the exists check.
        query.push_str("  )\n");

        // Reverse order by capture timestamp.
        query.push_str("ORDER BY datetime(capture_timestamp) DESC\n");

        // Add limit and offset
        let limit = options.limit.unwrap_or(500);
        let offset = options.offset.unwrap_or(0);

        query.push_str("LIMIT ?\n");
        params_vec.push(Box::new(limit));
        query.push_str("OFFSET ?\n");
        params_vec.push(Box::new(offset));

        debug!(
            query = query.replace("\n", " "),
            "Built sql query"
        );

        let mut stmt = conn.prepare(&query)?;
        let mut rows = stmt.query(rusqlite::params_from_iter(params_vec.iter()))?;

        let mut results: Vec<ImageInfo> = Vec::new();
        while let Some(row_result) = rows.next()? {
            let image_info = Self::row_to_image_info(&row_result)?;
            results.push(image_info);
        }

        debug!(row_count = results.len(), "Result count");

        Ok(results)
    }

}

impl ImageInfoRepository for ImageInfoRepositorySqlite {
    
    fn save_image_info(&self, info: &ImageInfo) -> ImageRepoResult<()> {
        self.save_image_info(info)
    }

    fn get_image_info(&self, image_id: &str) -> ImageRepoResult<Option<ImageInfo>> {
        self.get_image_info(image_id)
    }

    fn search_image_info(
        &self,
        options: &ImageInfoSearchOptions)
        -> ImageRepoResult<Vec<ImageInfo>>
    {
        self.search_image_info(options)
    }
}