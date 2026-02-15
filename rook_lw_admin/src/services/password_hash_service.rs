use base64::{Engine as _, engine::general_purpose::STANDARD};
use sha2::Sha256;
use pbkdf2::pbkdf2_hmac;

use crate::RookLWAdminResult;

/// Service for hashing and verifying passwords using PBKDF2
pub struct PasswordHashService;

const PBKDF2_ITERATIONS: u32 = 600_000;
const SALT_LENGTH: usize = 32;
const KEY_LENGTH: usize = 32;

impl PasswordHashService {
    /// Generate deterministic salt from user_id
    fn generate_salt(user_id: &str) -> Vec<u8> {
        use sha2::Digest;
        let mut hasher = sha2::Sha256::new();
        hasher.update(b"rook_lw_user_salt_");
        hasher.update(user_id.as_bytes());
        let hash = hasher.finalize();
        hash[0..SALT_LENGTH].to_vec()
    }
    
    /// Derive a key from a password and user_id using PBKDF2
    /// Returns base64-encoded derived key only (user_id is stored separately in DB)
    pub fn hash_password(user_id: &str, password: &str) -> RookLWAdminResult<String> {
        let salt = Self::generate_salt(user_id);
        
        let mut key = vec![0u8; KEY_LENGTH];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, PBKDF2_ITERATIONS, &mut key);
        
        // Return just base64(key) - user_id is already stored in the users table
        Ok(STANDARD.encode(&key))
    }
    
    /// Verify a password against a stored hash
    /// 
    /// # Arguments
    /// * `user_id` - The user's ID (used to derive deterministic salt)
    /// * `password` - The password to verify
    /// * `stored_hash` - The base64-encoded stored key
    pub fn verify_password(user_id: &str, password: &str, stored_hash: &str) -> RookLWAdminResult<bool> {
        let salt = Self::generate_salt(user_id);
        let stored_key = STANDARD.decode(stored_hash)
            .map_err(|e| crate::RookLWAdminError::Other(
                format!("Failed to decode key: {}", e)
            ))?;
        
        let mut derived_key = vec![0u8; KEY_LENGTH];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, PBKDF2_ITERATIONS, &mut derived_key);
        
        // Constant-time comparison
        Ok(derived_key == stored_key)
    }
    
    /// Derive the HMAC signing key from a password and user_id
    /// This is used for request signing - both client and server can derive the same key
    pub fn derive_signing_key(user_id: &str, password: &str) -> Vec<u8> {
        let salt = Self::generate_salt(user_id);
        let mut key = vec![0u8; KEY_LENGTH];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, PBKDF2_ITERATIONS, &mut key);
        key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password_format() {
        let hash = PasswordHashService::hash_password("user123", "my_password")
            .expect("Failed to hash password");
        
        // Should be just base64-encoded key
        let decoded = base64::engine::general_purpose::STANDARD.decode(&hash);
        assert!(decoded.is_ok());
        assert_eq!(decoded.unwrap().len(), KEY_LENGTH);
    }

    #[test]
    fn test_verify_password_correct() {
        let user_id = "test_user";
        let password = "correct_password";
        
        let hash = PasswordHashService::hash_password(user_id, password)
            .expect("Failed to hash password");
        
        let is_valid = PasswordHashService::verify_password(user_id, password, &hash)
            .expect("Failed to verify password");
        
        assert!(is_valid);
    }

    #[test]
    fn test_verify_password_incorrect() {
        let user_id = "test_user";
        let password = "correct_password";
        let wrong_password = "wrong_password";
        
        let hash = PasswordHashService::hash_password(user_id, password)
            .expect("Failed to hash password");
        
        let is_valid = PasswordHashService::verify_password(user_id, wrong_password, &hash)
            .expect("Failed to verify password");
        
        assert!(!is_valid);
    }

    #[test]
    fn test_deterministic_salt() {
        let user_id = "same_user";
        let password = "test_pass";
        
        // Hash the same password twice
        let hash1 = PasswordHashService::hash_password(user_id, password)
            .expect("Failed to hash password");
        let hash2 = PasswordHashService::hash_password(user_id, password)
            .expect("Failed to hash password");
        
        // Should produce identical hashes (deterministic salt)
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_different_users_different_hashes() {
        let password = "same_password";
        
        let hash1 = PasswordHashService::hash_password("user1", password)
            .expect("Failed to hash password");
        let hash2 = PasswordHashService::hash_password("user2", password)
            .expect("Failed to hash password");
        
        // Different users should have different hashes even with same password
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_derive_signing_key() {
        let user_id = "test_user";
        let password = "test_password";
        
        let key = PasswordHashService::derive_signing_key(user_id, password);
        
        // Should produce a key of correct length
        assert_eq!(key.len(), KEY_LENGTH);
        
        // Deriving again should produce same key (deterministic)
        let key2 = PasswordHashService::derive_signing_key(user_id, password);
        assert_eq!(key, key2);
    }

    #[test]
    fn test_signing_key_matches_stored_hash() {
        let user_id = "test_user";
        let password = "test_password";
        
        // Hash the password
        let hash = PasswordHashService::hash_password(user_id, password)
            .expect("Failed to hash password");
        
        // Derive signing key
        let signing_key = PasswordHashService::derive_signing_key(user_id, password);
        
        // Decode stored hash
        let stored_key = base64::engine::general_purpose::STANDARD.decode(&hash)
            .expect("Failed to decode stored key");
        
        // Should be identical
        assert_eq!(signing_key, stored_key);
    }

    #[test]
    fn test_verify_invalid_hash_format() {
        let result = PasswordHashService::verify_password("user", "password", "invalid_base64!!!");
        assert!(result.is_err());
    }

    #[test]
    fn test_special_characters_in_user_id() {
        let user_id = "user-with_special.chars@123";
        let password = "test_password";
        
        let hash = PasswordHashService::hash_password(user_id, password)
            .expect("Failed to hash password");
        
        let is_valid = PasswordHashService::verify_password(user_id, password, &hash)
            .expect("Failed to verify password");
        
        assert!(is_valid);
    }

    #[test]
    fn test_unicode_password() {
        let user_id = "test_user";
        let password = "パスワード🔐";
        
        let hash = PasswordHashService::hash_password(user_id, password)
            .expect("Failed to hash password");
        
        let is_valid = PasswordHashService::verify_password(user_id, password, &hash)
            .expect("Failed to verify password");
        
        assert!(is_valid);
        
        // Wrong password should fail
        let is_valid = PasswordHashService::verify_password(user_id, "different", &hash)
            .expect("Failed to verify password");
        assert!(!is_valid);
    }

    #[test]
    fn test_empty_password() {
        let user_id = "test_user";
        let password = "";
        
        let hash = PasswordHashService::hash_password(user_id, password)
            .expect("Failed to hash password");
        
        let is_valid = PasswordHashService::verify_password(user_id, password, &hash)
            .expect("Failed to verify password");
        
        assert!(is_valid);
    }

    #[test]
    fn test_very_long_password() {
        let user_id = "test_user";
        let password = "a".repeat(10000);
        
        let hash = PasswordHashService::hash_password(user_id, &password)
            .expect("Failed to hash password");
        
        let is_valid = PasswordHashService::verify_password(user_id, &password, &hash)
            .expect("Failed to verify password");
        
        assert!(is_valid);
    }
}
