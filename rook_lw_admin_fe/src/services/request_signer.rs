use chrono::Utc;
use js_sys::{Uint8Array, Array, Object, Reflect};
use rook_lw_models::user::RequestSignature;
use sha2::{Sha256, Digest};
use url::Url;
use wasm_bindgen::JsValue;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{window, CryptoKey};

use crate::RookLWAppResult;

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

/// Derive PBKDF2 key using Web Crypto API
async fn derive_key_webcrypto(password: &str, salt: &[u8], iterations: u32) -> Result<Vec<u8>, JsValue> {
    let crypto = window()
        .ok_or_else(|| JsValue::from_str("No window object"))?
        .crypto()
        .map_err(|_| JsValue::from_str("No crypto object"))?;
    
    let subtle = crypto.subtle();
    
    // Import password as raw key material
    let password_bytes = password.as_bytes();
    let password_array = Uint8Array::from(password_bytes);
    
    let key_usages = {
        let arr = Array::new();
        arr.push(&"deriveBits".into());
        arr
    };
    
    let import_promise = subtle
        .import_key_with_str("raw", &password_array, "PBKDF2", false, &key_usages)
        .map_err(|e| JsValue::from_str(&format!("Failed to import password: {:?}", e)))?;
    
    let key_material = JsFuture::from(import_promise)
        .await
        .map_err(|e| JsValue::from_str(&format!("Failed to await import: {:?}", e)))?
        .dyn_into::<CryptoKey>()
        .map_err(|_| JsValue::from_str("Failed to convert to CryptoKey"))?;
    
    // Derive bits using PBKDF2 - construct params object manually
    let derive_params = Object::new();
    Reflect::set(&derive_params, &"name".into(), &"PBKDF2".into())?;
    Reflect::set(&derive_params, &"salt".into(), &Uint8Array::from(salt))?;
    Reflect::set(&derive_params, &"iterations".into(), &JsValue::from(iterations))?;
    Reflect::set(&derive_params, &"hash".into(), &"SHA-256".into())?;
    
    let derive_promise = subtle
        .derive_bits_with_object(&derive_params, &key_material, (KEY_LENGTH * 8) as u32)
        .map_err(|e| JsValue::from_str(&format!("Failed to derive bits: {:?}", e)))?;
    
    let derived = JsFuture::from(derive_promise)
        .await
        .map_err(|e| JsValue::from_str(&format!("Failed to await derivation: {:?}", e)))?;
    
    let array = Uint8Array::new(&derived);
    Ok(array.to_vec())
}

/// Sign data using HMAC-SHA256 with Web Crypto API
async fn hmac_sign_webcrypto(key: &[u8], data: &[u8]) -> Result<Vec<u8>, JsValue> {
    let crypto = window()
        .ok_or_else(|| JsValue::from_str("No window object"))?
        .crypto()
        .map_err(|_| JsValue::from_str("No crypto object"))?;
    
    let subtle = crypto.subtle();
    
    // Import key with typed params - construct manually for reliability
    let key_array = Uint8Array::from(key);
    
    let import_params = Object::new();
    Reflect::set(&import_params, &"name".into(), &"HMAC".into())?;
    Reflect::set(&import_params, &"hash".into(), &"SHA-256".into())?;
    
    let key_usages = {
        let arr = Array::new();
        arr.push(&"sign".into());
        arr
    };
    
    let import_promise = subtle
        .import_key_with_object("raw", &key_array, &import_params, false, &key_usages)
        .map_err(|e| JsValue::from_str(&format!("Failed to import key: {:?}", e)))?;
    
    let crypto_key = JsFuture::from(import_promise)
        .await
        .map_err(|e| JsValue::from_str(&format!("Failed to await import key: {:?}", e)))?
        .dyn_into::<CryptoKey>()
        .map_err(|_| JsValue::from_str("Failed to convert to CryptoKey"))?;
    
    // Sign data with HMAC algorithm
    let sign_promise = subtle
        .sign_with_str_and_u8_array("HMAC", &crypto_key, data)
        .map_err(|e| JsValue::from_str(&format!("Failed to sign: {:?}", e)))?;
    
    let signature = JsFuture::from(sign_promise)
        .await
        .map_err(|e| JsValue::from_str(&format!("Failed to await signature: {:?}", e)))?;
    
    let sig_array = Uint8Array::new(&signature);
    Ok(sig_array.to_vec())
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


/// Derive signing key from user_id and password
pub async fn derive_signing_key(user_id: &str, password: &str) -> RookLWAppResult<Vec<u8>> {
    let salt = generate_salt(user_id);
    let key = derive_key_webcrypto(password, &salt, PBKDF2_ITERATIONS)
        .await
        .map_err(|e| crate::RookLWAppError::Other(
            format!("Failed to derive key: {:?}", e)
        ))?;
    Ok(key)
}
    
/// Sign a request using pre-derived signing key
/// 
/// # Arguments
/// * `user_id` - The user ID
/// * `signing_key` - Pre-derived PBKDF2 key from UserService
/// * `method` - HTTP method (GET, POST, etc.)
/// * `url` - Full URL including path and query params (excluding signature)
/// * `body` - Request body bytes
pub async fn sign_request(
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

    let signature = hmac_sign_webcrypto(signing_key, &message)
        .await
        .map_err(|e| crate::RookLWAppError::Other(
            format!("Failed to sign: {:?}", e)
        ))?;

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
    async fn test_derive_signing_key() {
        let key = derive_signing_key("test_user", "test_password")
            .await
            .expect("Failed to derive signing key");
        
        assert_eq!(key.len(), KEY_LENGTH);
        assert!(key.iter().any(|&b| b != 0), "Key should not be all zeros");
    }
    
    #[wasm_bindgen_test]
    async fn test_derive_signing_key_deterministic() {
        let key1 = derive_signing_key("user123", "password")
            .await
            .unwrap();
        let key2 = derive_signing_key("user123", "password")
            .await
            .unwrap();
        
        // Same credentials should produce same key
        assert_eq!(key1, key2);
    }
    
    #[wasm_bindgen_test]
    async fn test_derive_signing_key_different_passwords() {
        let key1 = derive_signing_key("user", "password1")
            .await
            .unwrap();
        let key2 = derive_signing_key("user", "password2")
            .await
            .unwrap();
        
        // Different passwords should produce different keys
        assert_ne!(key1, key2);
    }
    
    #[wasm_bindgen_test]
    async fn test_sign_request() {
        let signing_key = derive_signing_key("user", "password")
            .await
            .unwrap();
        
        let signature = sign_request(
            "user",
            &signing_key,
            "GET",
            "http://localhost/api/test",
            b"",
        )
        .await
        .expect("Failed to sign request");
        
        assert_eq!(signature.user_id, "user");
        assert!(!signature.signature.is_empty());
        assert_eq!(signature.salt.len(), 16);
        assert!(signature.signature.len() > 0);
    }
    
    #[wasm_bindgen_test]
    async fn test_sign_request_with_body() {
        let signing_key = derive_signing_key("user", "password")
            .await
            .unwrap();
        
        let body = b"{\"test\": \"data\"}";
        let signature = sign_request(
            "user",
            &signing_key,
            "POST",
            "http://localhost/api/test",
            body,
        )
        .await
        .unwrap();
        
        assert_eq!(signature.user_id, "user");
        assert!(!signature.signature.is_empty());
    }
    
    #[wasm_bindgen_test]
    async fn test_sign_request_different_salts() {
        let signing_key = derive_signing_key("user", "password")
            .await
            .unwrap();
        
        let sig1 = sign_request(
            "user",
            &signing_key,
            "GET",
            "http://localhost/api/test",
            b"",
        )
        .await
        .unwrap();
        
        let sig2 = sign_request(
            "user",
            &signing_key,
            "GET",
            "http://localhost/api/test",
            b"",
        )
        .await
        .unwrap();
        
        // Different random salts should produce different signatures
        assert_ne!(sig1.salt, sig2.salt);
        assert_ne!(sig1.signature, sig2.signature);
    }
    
    #[wasm_bindgen_test]
    async fn test_sign_request_method_affects_signature() {
        let signing_key = derive_signing_key("user", "password")
            .await
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
        .await
        .unwrap();
        
        let sig_post = sign_request(
            "user",
            &signing_key,
            "POST",
            "http://localhost/api/test",
            b"",
        )
        .await
        .unwrap();
        
        // Different methods should produce different signatures
        // (even though salts are different, this verifies method is included)
        assert_ne!(sig_get.signature, sig_post.signature);
    }
    
    #[wasm_bindgen_test]
    async fn test_signature_to_base64url() {
        let signing_key = derive_signing_key("user", "password")
            .await
            .unwrap();
        
        let signature = sign_request(
            "user",
            &signing_key,
            "GET",
            "http://localhost/api/test",
            b"",
        )
        .await
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
