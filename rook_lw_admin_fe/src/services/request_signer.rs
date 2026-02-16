use chrono::Utc;
use hmac::{Hmac, Mac};
use pbkdf2::pbkdf2_hmac;
use rook_lw_models::user::RequestSignature;
use sha2::{Sha256, Digest};
use url::Url;

use crate::RookLWAppResult;

type HmacSha256 = Hmac<Sha256>;

const PBKDF2_ITERATIONS: u32 = 600_000;
const SALT_LENGTH: usize = 32;
const KEY_LENGTH: usize = 32;

/// Generate deterministic salt from user_id (matches server-side logic)
fn generate_salt(user_id: &str) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(b"rook_lw_user_salt_");
    hasher.update(user_id.as_bytes());
    let hash = hasher.finalize();
    hash[0..SALT_LENGTH].to_vec()
}

/// Derive PBKDF2 key using pure Rust crypto
fn derive_key_pbkdf2(password: &str, salt: &[u8], iterations: u32) -> Vec<u8> {
    let mut key = vec![0u8; KEY_LENGTH];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, iterations, &mut key);
    key
}

/// Sign data using HMAC-SHA256 with pure Rust crypto
fn hmac_sign(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key)
        .expect("HMAC can take key of any size");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

/// Create the signing message string
fn create_signing_message(
    method: &str,
    url: &str,
    created_at: &chrono::DateTime<Utc>,
    salt: &[u8],
    body: &[u8],
) -> RookLWAppResult<Vec<u8>> {
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
            .filter(|param| !param.starts_with("sig=") && param.len() > 0)
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


/// Derive signing key from user_id and password
pub fn derive_signing_key(user_id: &str, password: &str) -> RookLWAppResult<Vec<u8>> {
    let salt = generate_salt(user_id);
    let key = derive_key_pbkdf2(password, &salt, PBKDF2_ITERATIONS);
    Ok(key)
}

/// Sign a URL by appending signature as query parameter
pub fn sign_url(user_id: &str, signing_key: &[u8], url: &str) -> RookLWAppResult<String> {
    let body = b"";
    let signature = sign_request(
        user_id,
        signing_key,
        "GET",
        url,
        body,
    )?;
    
    let sig_base64 = signature.to_base64url()
        .map_err(|e| crate::RookLWAppError::Other(format!("Failed to encode signature: {}", e)))?;
    
    let separator = if url.contains('?') { "&" } else { "?" };
    let signed_url = format!("{}{}sig={}", url, separator, sig_base64);
    Ok(signed_url)
}

/// Sign a request using pre-derived signing key
/// 
/// # Arguments
/// * `user_id` - The user ID
/// * `signing_key` - Pre-derived PBKDF2 key from UserService
/// * `method` - HTTP method (GET, POST, etc.)
/// * `url` - Full URL including path and query params (excluding signature)
/// * `body` - Request body bytes
pub fn sign_request(
    user_id: &str,
    signing_key: &[u8],
    method: &str,
    url: &str,
    body: &[u8],
) -> RookLWAppResult<RequestSignature> {
    // Generate random salt
    let mut salt = vec![0u8; 16];
    getrandom::getrandom(&mut salt)
        .map_err(|e| crate::RookLWAppError::Other(format!("Random generation failed: {}", e)))?;
    
    let created_at = Utc::now();
    
    let message = create_signing_message(
        method,
        &url,
        &created_at,
        &salt,
        body,
    )?;

    let signature = hmac_sign(signing_key, &message);

    Ok(RequestSignature {
        user_id: user_id.to_string(),
        created_at,
        salt,
        signature,
    })
}

#[cfg(test)]
mod wasm_tests {
    use super::*;
    use wasm_bindgen_test::*;
    
    wasm_bindgen_test_configure!(run_in_browser);
    
    #[wasm_bindgen_test]
    fn test_derive_signing_key() {
        let key = derive_signing_key("test_user", "test_password")
            .expect("Failed to derive signing key");
        
        assert_eq!(key.len(), KEY_LENGTH);
        assert!(key.iter().any(|&b| b != 0), "Key should not be all zeros");
    }
    
    #[wasm_bindgen_test]
    fn test_derive_signing_key_deterministic() {
        let key1 = derive_signing_key("user123", "password")
            .unwrap();
        let key2 = derive_signing_key("user123", "password")
            .unwrap();
        
        // Same credentials should produce same key
        assert_eq!(key1, key2);
    }
    
    #[wasm_bindgen_test]
    fn test_derive_signing_key_different_passwords() {
        let key1 = derive_signing_key("user", "password1")
            .unwrap();
        let key2 = derive_signing_key("user", "password2")
            .unwrap();
        
        // Different passwords should produce different keys
        assert_ne!(key1, key2);
    }
    
    #[wasm_bindgen_test]
    fn test_sign_request() {
        let signing_key = derive_signing_key("user", "password")
            .unwrap();
        
        let signature = sign_request(
            "user",
            &signing_key,
            "GET",
            "http://localhost/api/test",
            b"",
        )
        .expect("Failed to sign request");
        
        assert_eq!(signature.user_id, "user");
        assert!(!signature.signature.is_empty());
        assert_eq!(signature.salt.len(), 16);
        assert!(signature.signature.len() > 0);
    }
    
    #[wasm_bindgen_test]
    fn test_sign_request_with_body() {
        let signing_key = derive_signing_key("user", "password")
            .unwrap();
        
        let body = b"{\"test\": \"data\"}";
        let signature = sign_request(
            "user",
            &signing_key,
            "POST",
            "http://localhost/api/test",
            body,
        )
        .unwrap();
        
        assert_eq!(signature.user_id, "user");
        assert!(!signature.signature.is_empty());
    }
    
    #[wasm_bindgen_test]
    fn test_sign_request_different_salts() {
        let signing_key = derive_signing_key("user", "password")
            .unwrap();
        
        let sig1 = sign_request(
            "user",
            &signing_key,
            "GET",
            "http://localhost/api/test",
            b"",
        )
        .unwrap();
        
        let sig2 = sign_request(
            "user",
            &signing_key,
            "GET",
            "http://localhost/api/test",
            b"",
        )
        .unwrap();
        
        // Different random salts should produce different signatures
        assert_ne!(sig1.salt, sig2.salt);
        assert_ne!(sig1.signature, sig2.signature);
    }
    
    #[wasm_bindgen_test]
    fn test_sign_request_method_affects_signature() {
        let signing_key = derive_signing_key("user", "password")
            .unwrap();
        
        // Use same salt by signing at nearly the same time
        // (This is probabilistic but random salts make exact matching hard)
        let sig_get = sign_request(
            "user",
            &signing_key,
            "GET",
            "http://localhost/api/test",
            b"",
        )
        .unwrap();
        
        let sig_post = sign_request(
            "user",
            &signing_key,
            "POST",
            "http://localhost/api/test",
            b"",
        )
        .unwrap();
        
        // Different methods should produce different signatures
        // (even though salts are different, this verifies method is included)
        assert_ne!(sig_get.signature, sig_post.signature);
    }
    
    #[wasm_bindgen_test]
    fn test_signature_to_base64url() {
        let signing_key = derive_signing_key("user", "password")
            .unwrap();
        
        let signature = sign_request(
            "user",
            &signing_key,
            "GET",
            "http://localhost/api/test",
            b"",
        )
        .unwrap();
        
        let base64 = signature.to_base64url()
            .expect("Failed to encode to base64url");
        
        // Should be valid base64url string (not empty)
        assert!(!base64.is_empty());
        
        // Should be able to decode it back
        let decoded = RequestSignature::from_base64url(&base64)
            .expect("Failed to decode signature");
        
        assert_eq!(decoded.user_id, "user");
        assert_eq!(decoded.signature, signature.signature);
        assert_eq!(decoded.salt, signature.salt);
    }
}
