
use crate::ImageRepoResult;

use std::io::Read;

pub trait ImageStoreRepository : Send + Sync {
    fn store(&self, image_name: &str, image_data: &[u8]) -> ImageRepoResult<()>;
    fn read<'a>(&'a self, image_name: &str) -> ImageRepoResult<Box<dyn Read + Send + 'a>>;
}