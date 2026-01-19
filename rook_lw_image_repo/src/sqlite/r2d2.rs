use crate::ImageRepoResult;

use r2d2::{Pool};
use r2d2_sqlite::SqliteConnectionManager;

// Creates a new r2d2 connection pool for the given SQLite database path.
pub fn create_pool(db_path: &str) -> ImageRepoResult<Pool<SqliteConnectionManager>> {
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

    Ok(Pool::new(manager)?)
}
