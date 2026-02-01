use gloo_net::http::Request;
use serde_qs;

use rook_lw_models::image::{ImageInfo, ImageInfoSearchOptions};

use crate::RookLWAppResult;
use crate::services::response_ok;

#[derive(Debug, Clone)]
pub struct ImageInfoService {
    pub base_path: String,
}

impl ImageInfoService {

    pub fn new(base_path: impl Into<String>) -> Self {
        Self { base_path: base_path.into() }
    }

    pub async fn get(&self, image_id: impl AsRef<str>)
        -> RookLWAppResult<Option<ImageInfo>>
    {
        let url = format!("{}/api/image_info/{}", &self.base_path, image_id.as_ref());

        let resp = Request::get(url.as_str())
            .send()
            .await?;

        let resp = response_ok(resp).await?;

        if resp.ok() {
            Ok(Some(resp.json::<ImageInfo>().await?))
        }
        else {
            Ok(None)
        }
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

