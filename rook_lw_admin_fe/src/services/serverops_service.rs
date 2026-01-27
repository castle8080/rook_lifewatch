use gloo_net::http::Request;
use rook_lw_models::Status;

use crate::RookLWAppResult;
use crate::services::response_ok;

#[derive(Debug, Clone)]
pub struct ServeropsService {
    pub base_path: String,
}

impl ServeropsService {
    pub fn new(base_path: impl Into<String>) -> Self {
        Self {
            base_path: base_path.into()
        }
    }

    pub async fn shutdown(&self) -> RookLWAppResult<Status> {
        let url = format!("{}/api/server/shutdown", &self.base_path);

        let resp = Request::post(url.as_str())
            .send()
            .await?;
        let resp = response_ok(resp).await?;
        Ok(resp.json::<Status>().await?)
    }
}