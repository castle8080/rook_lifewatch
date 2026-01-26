use gloo_net::http::Request;
use rook_lw_models::Status;
use rook_lw_models::process::ProcessInfo;
use serde::Deserialize;

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

    pub async fn start(&self) -> RookLWAppResult<ProcessInfo> {
        self.basic_post_command("start").await
    }

    pub async fn stop(&self) -> RookLWAppResult<Status> {
        self.basic_post_command("stop").await
    }

    async fn basic_post_command<T>(&self, cmd: &str) -> RookLWAppResult<T>
    where
        for<'de> T: Deserialize<'de>,
    {
        let url = format!("{}/api/daemon/{}", &self.base_path, cmd);

        let resp = Request::post(url.as_str())
            .send()
            .await?;
        let resp = response_ok(resp).await?;
        Ok(resp.json::<T>().await?)
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
            Ok(Some(resp.json().await?))
        }
    }
}



