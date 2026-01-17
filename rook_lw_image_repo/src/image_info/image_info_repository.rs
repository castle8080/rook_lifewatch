
use rook_lw_models::image::ImageInfo;

use crate::ImageRepoResult;

pub trait ImageInfoRepository: Send + Sync {
    fn save_image_info(&self, info: &ImageInfo) -> ImageRepoResult<()>;
    fn get_image_info(&self, image_id: &str) -> ImageRepoResult<Option<ImageInfo>>;
}
