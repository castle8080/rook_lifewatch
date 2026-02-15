use rook_lw_models::user::{RequestSignature, User};
use crate::{RookLWAdminResult, RookLWAdminError};
use crate::services::password_hash_service::PasswordHashService;
use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use rand::RngCore;
use base64::{Engine as _, engine::general_purpose::STANDARD};

type HmacSha256 = Hmac<Sha256>;

pub struct RequestSignatureService;

impl RequestSignatureService {
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
        
        // Construct message to sign: METHOD|URL|TIMESTAMP|SALT|BODY
        let timestamp_str = created_at.to_rfc3339();
        let salt_hex = hex::encode(&salt);
        let message = format!(
            "{}|{}|{}|{}|{}",
            method.to_uppercase(),
            url,
            timestamp_str,
            salt_hex,
            hex::encode(body)
        );
        
        // Derive signing key from password and user_id
        let signing_key = PasswordHashService::derive_signing_key(user_id, password);
        
        // Create HMAC using derived key
        let mut mac = HmacSha256::new_from_slice(&signing_key)
            .map_err(|e| RookLWAdminError::Other(format!("HMAC error: {}", e)))?;
        mac.update(message.as_bytes());
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
        
        // Reconstruct the message
        let timestamp_str = request_sig.created_at.to_rfc3339();
        let salt_hex = hex::encode(&request_sig.salt);
        let message = format!(
            "{}|{}|{}|{}|{}",
            method.to_uppercase(),
            url,
            timestamp_str,
            salt_hex,
            hex::encode(body)
        );
        
        // Extract the derived key directly from the stored hash (just base64 now)
        let signing_key = STANDARD.decode(&user.password_hash)
            .map_err(|_| RookLWAdminError::Other("Invalid key in hash".to_string()))?;
        
        // Verify HMAC
        let mut mac = HmacSha256::new_from_slice(&signing_key)
            .map_err(|e| RookLWAdminError::Other(format!("HMAC error: {}", e)))?;
        mac.update(message.as_bytes());
        let expected_signature = mac.finalize().into_bytes();
        
        Ok(expected_signature.as_slice() == request_sig.signature.as_slice())
    }
}
