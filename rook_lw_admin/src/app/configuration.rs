use crate::RookLWAdminResult;
use crate::app::AppState;
use crate::services::{DaemonService, UserService};

use rook_lw_image_repo::sqlite::create_pool;
use rook_lw_image_repo::image_info::{ImageInfoRepository, ImageInfoRepositorySqlite};
use rook_lw_image_repo::image_store::{ImageStoreRepository, ImageStoreRepositoryFile};
use rook_lw_image_repo::user_repo::{UserRepository, UserRepositorySqlite};

use std::sync::Arc;
use r2d2_sqlite::SqliteConnectionManager;
use r2d2::Pool;

pub fn create_app(var_dir: &str, admin_dir: &str, app_dir: &str) -> RookLWAdminResult<AppState> {
    // Create separate database pools
    let image_sqlite_pool = create_sqlite_pool(&format!("{}/db/image_info.db", var_dir))?;
    let user_sqlite_pool = create_sqlite_pool(&format!("{}/db/users.db", var_dir))?;
    
    // Create repositories
    let image_info_repo = create_image_info_repository(image_sqlite_pool)?;
    let user_repo = create_user_repository(user_sqlite_pool)?;
    
    // Create services
    let user_service = Arc::new(UserService::new(Arc::clone(&user_repo)));
    
    // Initialize default admin user if no users exist
    user_service.initialize_default_admin()?;

    let app = AppState {
        admin_static_dir: admin_dir.to_string(),
        image_info_repo: Arc::new(image_info_repo),
        image_store_repo: Arc::new(create_image_store_repository(var_dir)?),
        daemon_service: Arc::new(create_daemon_service(app_dir)?),
        user_repo,
        user_service,
    };

    Ok(app)
}

fn create_sqlite_pool(db_path: &str) -> RookLWAdminResult<Pool<SqliteConnectionManager>> {
    let pool = create_pool(db_path)?;
    Ok(pool)
}

fn create_image_info_repository(pool: Pool<SqliteConnectionManager>) -> RookLWAdminResult<Box<dyn ImageInfoRepository>> {
    let repo = ImageInfoRepositorySqlite::new(pool)?;
    Ok(Box::new(repo))
}

fn create_user_repository(pool: Pool<SqliteConnectionManager>) -> RookLWAdminResult<Arc<Box<dyn UserRepository>>> {
    let repo = UserRepositorySqlite::new(pool)?;
    Ok(Arc::new(Box::new(repo)))
}

fn create_image_store_repository(var_dir: &str) -> RookLWAdminResult<Box<dyn ImageStoreRepository>> {
    let images_path = format!("{}/images", var_dir);
    let repo = ImageStoreRepositoryFile::new(
        images_path.into()
    )?;

    Ok(Box::new(repo))
}

fn create_daemon_service(app_dir: &str) -> RookLWAdminResult<DaemonService> {
    DaemonService::new(app_dir)
}
