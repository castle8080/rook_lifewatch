use gloo_net::http::Request;
use rook_lw_models::Status;

use crate::RookLWAppResult;
use crate::services::{response_ok, add_signature, UserService};

/// Service for handling login API calls
#[derive(Debug, Clone)]
pub struct LoginService {
    pub base_path: String,
}

impl LoginService {
    pub fn new(base_path: impl Into<String>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }

    /// Verify login credentials against the server
    /// 
    /// This endpoint expects the request to be signed with the user's credentials.
    /// The server will validate the signature to authenticate the user.
    /// 
    /// # Arguments
    /// * `user_service` - UserService that has been initialized with credentials
    /// 
    /// # Returns
    /// Ok(()) if authentication is successful, error otherwise
    pub async fn verify_login(&self, user_service: &UserService) -> RookLWAppResult<()> {
        let url = format!("{}/api/login", &self.base_path);
        let body = b"";

        let request = Request::post(url.as_str());
        let request = add_signature(Some(user_service), request, "POST", &url, body)?;

        let resp = request.send().await?;
        let resp = response_ok(resp).await?;
        
        // Parse response to ensure it's valid
        let _status: Status = resp.json().await?;
        Ok(())
    }
}
