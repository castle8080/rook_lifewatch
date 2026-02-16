use base64::{Engine as _, engine::general_purpose::STANDARD};
use chrono::Utc;
use hmac::{Hmac, Mac};
use rand::RngCore;
use sha2::Sha256;
use url::Url;

use rook_lw_models::user::{RequestSignature, User};

use crate::{RookLWAdminResult, RookLWAdminError};
use crate::services::password_hash_service::PasswordHashService;

type HmacSha256 = Hmac<Sha256>;

/// Create the signing message string
fn create_signing_message(
    method: &str,
    url: &str,
    created_at: &chrono::DateTime<Utc>,
    salt: &[u8],
    body: &[u8],
) -> RookLWAdminResult<Vec<u8>> {
    let mut message: Vec<u8> = Vec::new();

    message.extend(method.to_uppercase().as_bytes());
    message.push(b'|');

    let url = Url::parse("http://dummy")?.join(url)?;
    message.extend(url.path().as_bytes());

    // Handle query parameters
    // Exclude 'signature' parameter and sort remaining.
    let query = url.query();
    if let Some(q) = query {
        let mut pairs = q.split('&')
            .filter(|param| !param.starts_with("signature=") && param.len() > 0)
            .collect::<Vec<_>>();

        pairs.sort();

        for (idx, pair) in pairs.iter().enumerate() {
            if idx > 0 {
                message.push(b'&');
            }
            else {
                message.push(b'?');
            }
            message.extend(pair.as_bytes());
        }
    }
    message.push(b'|');
    
    message.extend(created_at.to_rfc3339().as_bytes());
    message.push(b'|');

    message.extend(salt);
    message.push(b'|');
    
    message.extend(body);

    Ok(message)
}


/// Create a signature for a request
/// 
/// # Arguments
/// * `user_id` - The ID of the user making the request
/// * `password` - The user's password (for deriving signing key)
/// * `method` - HTTP method (GET, POST, etc.)
/// * `url` - Full URL including path and query params (excluding signature params)
/// * `body` - Request body (empty for GET requests)
pub fn sign_request(
    user_id: &str,
    password: &str,
    method: &str,
    url: &str,
    body: &[u8],
) -> RookLWAdminResult<RequestSignature> {
    // Generate random salt for this request
    let mut salt = vec![0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt);
    
    let created_at = Utc::now();
    
    let message = create_signing_message(method, url, &created_at, &salt, body)?;
    let signing_key = PasswordHashService::derive_signing_key(user_id, password);
    
    // Create HMAC using derived key
    let mut mac = HmacSha256::new_from_slice(&signing_key)
        .map_err(|e| RookLWAdminError::Other(format!("HMAC error: {}", e)))?;

    mac.update(&message);
    let signature = mac.finalize().into_bytes().to_vec();
    
    Ok(RequestSignature {
        user_id: user_id.to_string(),
        created_at,
        salt,
        signature,
    })
}

/// Verify a request signature
/// 
/// # Arguments
/// * `user` - The user who supposedly made the request
/// * `request_sig` - The signature to verify
/// * `method` - HTTP method
/// * `url` - Full URL (excluding signature params)
/// * `body` - Request body
pub fn verify_signature(
    user: &User,
    request_sig: &RequestSignature,
    method: &str,
    url: &str,
    body: &[u8],
) -> RookLWAdminResult<bool> {
    // Check if signature is expired
    if request_sig.is_expired() {
        return Ok(false);
    }
    
    // Check user ID matches
    if request_sig.user_id != user.id {
        return Ok(false);
    }
    
    let message = create_signing_message(method, url, &request_sig.created_at, &request_sig.salt, body)?;
    let signing_key = STANDARD.decode(&user.password_hash)
        .map_err(|_| RookLWAdminError::Other("Invalid key in hash".to_string()))?;

    // Verify HMAC (constant-time)
    let mut mac = HmacSha256::new_from_slice(&signing_key)
        .map_err(|e| RookLWAdminError::Other(format!("HMAC error: {}", e)))?;
    mac.update(&message);

    Ok(mac.verify_slice(request_sig.signature.as_slice()).is_ok())
}