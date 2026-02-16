//! Shared utilities for signing HTTP requests

use gloo_net::http::RequestBuilder;
use crate::RookLWAppResult;
use super::{UserService, RequestSigner};

/// Add signature header to request builder if user is authenticated and path requires it
/// 
/// # Arguments
/// * `user_service` - Optional user service with cached credentials
/// * `request` - The request builder to modify
/// * `method` - HTTP method (GET, POST, etc.)
/// * `url` - Full URL of the request
/// * `body` - Request body bytes
/// 
/// # Returns
/// Modified request builder with X-Signature header added if authenticated
pub async fn add_signature_if_needed(
    user_service: Option<&UserService>,
    mut request: RequestBuilder,
    method: &str,
    url: &str,
    body: &[u8],
) -> RookLWAppResult<RequestBuilder> {
    // Check if this is an API path that needs signing
    if url.contains("/api/") {
        if let Some(user_service) = user_service {
            if let Some(signing_key) = user_service.signing_key() {
                let user_id = user_service.user_id().expect("user_id should exist with signing_key");
                
                // Sign the request using cached key
                let signature = RequestSigner::sign_request(
                    user_id,
                    signing_key,
                    method,
                    url,
                    body,
                ).await?;
                
                // Add signature header
                let sig_header = signature.to_base64url()
                    .map_err(|e| crate::RookLWAppError::Other(format!("Failed to encode signature: {}", e)))?;
                request = request.header("X-Signature", &sig_header);
            }
        }
    }
    Ok(request)
}
