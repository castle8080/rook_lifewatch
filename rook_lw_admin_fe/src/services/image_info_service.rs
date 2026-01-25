use gloo_net::http::Request;
use serde_qs;

use rook_lw_models::image::{ImageInfo, ImageInfoSearchOptions};

use crate::{RookLWAppResult, services::response_ok};

#[derive(Debug, Clone)]
pub struct ImageInfoService {
    pub base_path: String,
}

impl ImageInfoService {

    pub fn new(base_path: impl Into<String>) -> Self {
        Self { base_path: base_path.into() }
    }

    pub async fn search(&self, search_options: &ImageInfoSearchOptions)
        -> RookLWAppResult<Vec<ImageInfo>>
    {
        let query_str = serde_qs::to_string(search_options)?;
        let url = format!("{}/api/image_info?{}", &self.base_path, &query_str);

        let resp = Request::get(url.as_str())
            .send()
            .await?;

        let resp = response_ok(resp).await?;
            
        let images = resp.json::<Vec<ImageInfo>>().await?;
        Ok(images)
    }
}



