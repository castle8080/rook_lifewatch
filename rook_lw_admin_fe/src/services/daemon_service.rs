use gloo_net::http::Request;
use rook_lw_models::Status;
use rook_lw_models::process::ProcessInfo;
use serde::Deserialize;

use crate::RookLWAppResult;
use crate::services::{response_ok, response_ok_or_not_found, UserService, add_signature_if_needed};

#[derive(Debug, Clone)]
pub struct DaemonService {
    pub base_path: String,
    pub user_service: Option<UserService>,
}

impl DaemonService {

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
        let body = b"";

        let request = Request::post(url.as_str());
        let request = add_signature_if_needed(self.user_service.as_ref(), request, "POST", &url, body).await?;
        
        let resp = request.send().await?;
        let resp = response_ok(resp).await?;
        Ok(resp.json::<T>().await?)
    }

    pub async fn status(&self) -> RookLWAppResult<Option<ProcessInfo>> {
        let url = format!("{}/api/daemon/status", &self.base_path);
        let body = b"";

        let request = Request::get(url.as_str());
        let request = add_signature_if_needed(self.user_service.as_ref(), request, "GET", &url, body).await?;
        
        let resp = request.send().await?;
        let resp = response_ok_or_not_found(resp).await?;
        if resp.status() == 404 {
            Ok(None)
        }
        else {
            Ok(Some(resp.json().await?))
        }
    }
}



