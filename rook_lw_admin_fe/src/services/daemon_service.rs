use gloo_net::http::Request;
use rook_lw_models::process::ProcessInfo;

use crate::RookLWAppResult;
use crate::services::{response_ok, response_ok_or_not_found};

#[derive(Debug, Clone)]
pub struct DaemonService {
    pub base_path: String,
}

impl DaemonService {

    pub fn new(base_path: impl Into<String>) -> Self {
        Self { base_path: base_path.into() }
    }

    pub async fn status(&self) -> RookLWAppResult<Option<ProcessInfo>> {
        let url = format!("{}/api/daemon/status", &self.base_path);

        let resp = Request::get(url.as_str())
            .send()
            .await?;

        let resp = response_ok_or_not_found(resp).await?;

        if resp.status() == 404 {
            Ok(None)
        }
        else {
            Ok(Some(resp.json::<ProcessInfo>().await?))
        }
    }
}



