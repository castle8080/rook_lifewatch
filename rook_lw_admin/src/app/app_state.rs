use std::sync::Arc;

use rook_lw_image_repo::image_info::ImageInfoRepository;
use rook_lw_image_repo::image_store::ImageStoreRepository;

use crate::services::DaemonService;

#[derive(Clone)]
pub struct AppState {
    pub admin_static_dir: String,
    pub image_info_repo: Arc<Box<dyn ImageInfoRepository>>,
    pub image_store_repo: Arc<Box<dyn ImageStoreRepository>>,
    pub daemon_service: Arc<DaemonService>,
}
