use wasm_bindgen::JsValue;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use js_sys::{Uint8Array, Object, Reflect, Array};
use web_sys::{window, CryptoKey};
use rook_lw_models::user::RequestSignature;
use chrono::Utc;
use sha2::{Sha256, Digest};
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
    
    let import_params = Object::new();
    Reflect::set(&import_params, &"name".into(), &"PBKDF2".into())?;
    
    let key_usages = {
        let arr = Array::new();
        arr.push(&"deriveBits".into());
        arr
    };
    
    let import_promise = subtle
        .import_key_with_object("raw", &password_array, &import_params, false, &key_usages)
        .map_err(|e| JsValue::from_str(&format!("Failed to import password: {:?}", e)))?;
    
    let key_material = JsFuture::from(import_promise)
        .await
        .map_err(|e| JsValue::from_str(&format!("Failed to await import: {:?}", e)))?
        .dyn_into::<CryptoKey>()
        .map_err(|_| JsValue::from_str("Failed to convert to CryptoKey"))?;
    
    // Derive bits using PBKDF2
    let derive_params = Object::new();
    Reflect::set(&derive_params, &"name".into(), &"PBKDF2".into())?;
    Reflect::set(&derive_params, &"salt".into(), &Uint8Array::from(salt))?;
    Reflect::set(&derive_params, &"iterations".into(), &JsValue::from(iterations))?;
    
    let hash_obj = Object::new();
    Reflect::set(&hash_obj, &"name".into(), &"SHA-256".into())?;
    Reflect::set(&derive_params, &"hash".into(), &hash_obj)?;
    
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
    
    // Import key
    let key_array = Uint8Array::from(key);
    let import_params = Object::new();
    Reflect::set(&import_params, &"name".into(), &"HMAC".into())?;
    
    let hash_obj = Object::new();
    Reflect::set(&hash_obj, &"name".into(), &"SHA-256".into())?;
    Reflect::set(&import_params, &"hash".into(), &hash_obj)?;
    
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
    
    // Sign data
    let sign_params = Object::new();
    Reflect::set(&sign_params, &"name".into(), &"HMAC".into())?;
    
    let sign_promise = subtle
        .sign_with_object_and_u8_array(&sign_params, &crypto_key, data)
        .map_err(|e| JsValue::from_str(&format!("Failed to sign: {:?}", e)))?;
    
    let signature = JsFuture::from(sign_promise)
        .await
        .map_err(|e| JsValue::from_str(&format!("Failed to await signature: {:?}", e)))?;
    
    let sig_array = Uint8Array::new(&signature);
    Ok(sig_array.to_vec())
}

/// Request signer using Web Crypto API
pub struct RequestSigner;

impl RequestSigner {
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
        
        // Sign message with pre-derived key
        let signature = hmac_sign_webcrypto(signing_key, message.as_bytes())
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
}

// Need hex crate for encoding - add simple implementation
mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }
}
