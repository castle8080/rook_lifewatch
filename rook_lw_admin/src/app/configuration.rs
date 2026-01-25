use crate::RookLWAdminResult;
use crate::app::AppState;
use crate::services::DaemonService;

use rook_lw_image_repo::sqlite::create_pool;
use rook_lw_image_repo::image_info::{ImageInfoRepository, ImageInfoRepositorySqlite};
use rook_lw_image_repo::image_store::{ImageStoreRepository, ImageStoreRepositoryFile};

use std::sync::Arc;
use r2d2_sqlite::SqliteConnectionManager;
use r2d2::Pool;

pub fn create_app(var_dir: &str, admin_dir: &str, app_dir: &str) -> RookLWAdminResult<AppState> {
    let sqlite_pool = create_sqlite_pool(var_dir)?;
    let image_info_repo = create_image_info_repository(sqlite_pool)?;

    let app = AppState {
        admin_static_dir: admin_dir.to_string(),
        image_info_repo: Arc::new(image_info_repo),
        image_store_repo: Arc::new(create_image_store_repository(var_dir)?),
        daemon_service: Arc::new(create_daemon_service(app_dir)?),
    };

    Ok(app)
}

fn create_sqlite_pool(var_dir: &str) -> RookLWAdminResult<Pool<SqliteConnectionManager>> {
    let db_path = format!("{}/db/image_info.db", var_dir);
    let pool = create_pool(&db_path)?;
    Ok(pool)
}

fn create_image_info_repository(pool: Pool<SqliteConnectionManager>) -> RookLWAdminResult<Box<dyn ImageInfoRepository>> {
    let repo = ImageInfoRepositorySqlite::new(
        pool
    )?;

    Ok(Box::new(repo))
}

fn create_image_store_repository(var_dir: &str) -> RookLWAdminResult<Box<dyn ImageStoreRepository>> {
    let images_path = format!("{}/images", var_dir);
    let repo = ImageStoreRepositoryFile::new(
        images_path.into()
    )?;

    Ok(Box::new(repo))
}

fn create_daemon_service(app_dir: &str) -> RookLWAdminResult<DaemonService> {
    DaemonService::create(app_dir)
}