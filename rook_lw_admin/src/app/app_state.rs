use std::sync::Arc;

use rook_lw_image_repo::image_info::ImageInfoRepository;
use rook_lw_image_repo::image_store::ImageStoreRepository;
use rook_lw_image_repo::user_repo::UserRepository;

use crate::services::{DaemonService, UserService};

#[derive(Clone)]
pub struct AppState {
    pub admin_static_dir: String,
    pub image_info_repo: Arc<Box<dyn ImageInfoRepository>>,
    pub image_store_repo: Arc<Box<dyn ImageStoreRepository>>,
    pub daemon_service: Arc<DaemonService>,
    pub user_repo: Arc<Box<dyn UserRepository>>,
    pub user_service: Arc<UserService>,
}
