use std::fs::{File, write, create_dir_all};
use std::io::Read;
use std::path::PathBuf;

use super::ImageStoreRepository;
use crate::ImageRepoResult;

use tracing::info;

pub struct ImageStoreRepositoryFile {
    image_directory: PathBuf,
}

impl ImageStoreRepositoryFile {

    pub fn new(image_directory: PathBuf) -> ImageRepoResult<Self> {
        create_dir_all(&image_directory)?;
        Ok(Self { image_directory })
    }
    
}

impl ImageStoreRepository for ImageStoreRepositoryFile {
    fn store(&self, image_name: &str, image_data: &[u8]) -> ImageRepoResult<()> {
        let image_path = self.image_directory.join(image_name);
        info!("Storing image at {:?}", image_path);
        if let Some(parent_dir) = image_path.parent() {
            create_dir_all(parent_dir)?;
        }
        write(image_path, image_data)?;
        Ok(())
    }
    
    fn read<'a>(&'a self, image_name: &str) -> ImageRepoResult<Box<dyn Read + Send + 'a>> {
        let image_path = self.image_directory.join(image_name);
        info!("Reading image from {:?}", image_path);
        let file = File::open(image_path)?;
        Ok(Box::new(file))
    }
}