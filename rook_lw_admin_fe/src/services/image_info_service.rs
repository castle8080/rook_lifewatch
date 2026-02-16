use gloo_net::http::Request;
use rook_lw_models::image::{ImageInfo, ImageInfoSearchOptions};

use crate::RookLWAppResult;
use crate::services::{response_ok, response_ok_or_not_found, UserService, add_signature};

#[derive(Debug, Clone)]
pub struct ImageInfoService {
    pub base_path: String,
    pub user_service: Option<UserService>,
}

impl ImageInfoService {

    pub fn new(base_path: impl Into<String>) -> Self {
        Self { 
            base_path: base_path.into(),
            user_service: None,
        }
    }
    
    pub fn with_user_service(mut self, user_service: UserService) -> Self {
        self.user_service = Some(user_service);
        self
    }

    pub async fn get(&self, image_id: impl AsRef<str>)
        -> RookLWAppResult<Option<ImageInfo>>
    {
        let url = format!("{}/api/image_info/{}", &self.base_path, image_id.as_ref());
        let body = b"";

        let request = Request::get(url.as_str());
        let request = add_signature(self.user_service.as_ref(), request, "GET", &url, body).await?;

        let resp = request.send().await?;
        let resp = response_ok_or_not_found(resp).await?;
        
        if resp.status() == 404 {
            Ok(None)
        }
        else {
            Ok(Some(resp.json().await?))
        }
    }

    pub async fn search(&self, search_options: &ImageInfoSearchOptions)
        -> RookLWAppResult<Vec<ImageInfo>>
    {
        let url = format!(
            "{}/api/image_info?{}",
            &self.base_path,
            serde_qs::to_string(search_options)?
        );
        let body = b"";

        let request = Request::get(url.as_str());
        let request = add_signature(self.user_service.as_ref(), request, "GET", &url, body).await?;

        tracing::info!("Sending image search request to URL: {}", url);

        let resp = request.send().await?;
        let resp = response_ok(resp).await?;
            
        let images = resp.json::<Vec<ImageInfo>>().await?;
        Ok(images)
    }
}

