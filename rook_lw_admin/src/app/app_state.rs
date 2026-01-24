use std::sync::Arc;

use rook_lw_image_repo::image_info::ImageInfoRepository;

#[derive(Clone)]
pub struct AppState {
    pub admin_static_dir: String,
    pub image_info_repo: Arc<Box<dyn ImageInfoRepository>>,
}
