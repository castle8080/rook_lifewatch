use web_sys::{window, Storage};
use serde::{Serialize, Deserialize};
use crate::RookLWAppResult;

use crate::services::derive_signing_key;

const SESSION_KEY: &str = "rook_lw_user_session";

/// User credentials stored in memory/session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCredentials {
    pub user_id: String,
    pub password: String,
    #[serde(with = "super::base64_serde")]
    pub signing_key: Vec<u8>,
}

/// Client-side user service for managing authentication state
#[derive(Debug, Clone)]
pub struct UserService {
    credentials: Option<UserCredentials>,
}

impl UserService {
    /// Create a new user service
    pub fn new() -> Self {
        Self {
            credentials: Self::load_from_session(),
        }
    }
    
    /// Log in a user and derive signing key
    pub async fn login(&mut self, user_id: String, password: String) -> RookLWAppResult<()> {
        // Derive signing key (expensive operation, done once)
        let signing_key = derive_signing_key(&user_id, &password).await?;
        
        let creds = UserCredentials { 
            user_id, 
            password,
            signing_key,
        };
        self.credentials = Some(creds.clone());
        Self::save_to_session(&creds)?;
        Ok(())
    }
    
    /// Log out the current user
    pub fn logout(&mut self) -> RookLWAppResult<()> {
        self.credentials = None;
        Self::clear_session()?;
        Ok(())
    }
    
    /// Check if user is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.credentials.is_some()
    }
    
    /// Get user credentials (if authenticated)
    pub fn get_credentials(&self) -> Option<&UserCredentials> {
        self.credentials.as_ref()
    }
    
    /// Get user ID (if authenticated)
    pub fn user_id(&self) -> Option<&str> {
        self.credentials.as_ref().map(|c| c.user_id.as_str())
    }
    
    /// Get cached signing key (if authenticated)
    pub fn signing_key(&self) -> Option<&[u8]> {
        self.credentials.as_ref().map(|c| c.signing_key.as_slice())
    }
    
    // SessionStorage persistence
    
    fn get_session_storage() -> RookLWAppResult<Storage> {
        window()
            .ok_or_else(|| crate::RookLWAppError::Other("No window object".to_string()))?
            .session_storage()
            .map_err(|_| crate::RookLWAppError::Other("No session storage".to_string()))?
            .ok_or_else(|| crate::RookLWAppError::Other("Session storage not available".to_string()))
    }
    
    fn load_from_session() -> Option<UserCredentials> {
        let storage = Self::get_session_storage().ok()?;
        let json = storage.get_item(SESSION_KEY).ok()??;
        serde_json::from_str(&json).ok()
    }
    
    fn save_to_session(creds: &UserCredentials) -> RookLWAppResult<()> {
        let storage = Self::get_session_storage()?;
        let json = serde_json::to_string(creds)
            .map_err(|e| crate::RookLWAppError::Other(format!("Serialization error: {}", e)))?;
        storage.set_item(SESSION_KEY, &json)
            .map_err(|_| crate::RookLWAppError::Other("Failed to save to session".to_string()))?;
        Ok(())
    }
    
    fn clear_session() -> RookLWAppResult<()> {
        let storage = Self::get_session_storage()?;
        storage.remove_item(SESSION_KEY)
            .map_err(|_| crate::RookLWAppError::Other("Failed to clear session".to_string()))?;
        Ok(())
    }
}

impl Default for UserService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Native tests for UserCredentials serialization
    #[test]
    fn test_user_credentials_serialization() {
        let creds = UserCredentials {
            user_id: "test_user".to_string(),
            password: "test_pass".to_string(),
            signing_key: vec![1, 2, 3, 4, 5],
        };
        
        // Serialize to JSON
        let json = serde_json::to_string(&creds).unwrap();
        
        // Should contain base64-encoded signing_key
        assert!(json.contains("\"signing_key\""));
        assert!(json.contains("\"user_id\":\"test_user\""));
        assert!(json.contains("\"password\":\"test_pass\""));
        
        // Deserialize back
        let restored: UserCredentials = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.user_id, "test_user");
        assert_eq!(restored.password, "test_pass");
        assert_eq!(restored.signing_key, vec![1, 2, 3, 4, 5]);
    }
    
    #[test]
    fn test_user_credentials_base64_encoding() {
        let creds = UserCredentials {
            user_id: "user1".to_string(),
            password: "pass1".to_string(),
            signing_key: vec![255, 128, 64, 32, 16, 8, 4, 2, 1],
        };
        
        let json = serde_json::to_string(&creds).unwrap();
        let restored: UserCredentials = serde_json::from_str(&json).unwrap();
        
        // Binary data should survive round-trip
        assert_eq!(restored.signing_key, vec![255, 128, 64, 32, 16, 8, 4, 2, 1]);
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;
    use wasm_bindgen_test::*;
    
    wasm_bindgen_test_configure!(run_in_browser);
    
    // Helper to clear session storage before each test
    fn clear_test_storage() {
        if let Ok(storage) = UserService::get_session_storage() {
            let _ = storage.remove_item(SESSION_KEY);
        }
    }
    
    #[wasm_bindgen_test]
    fn test_new_user_service_empty() {
        clear_test_storage();
        let service = UserService::new();
        assert!(!service.is_authenticated());
        assert!(service.get_credentials().is_none());
        assert!(service.user_id().is_none());
        assert!(service.signing_key().is_none());
    }
    
    #[wasm_bindgen_test]
    fn test_user_service_default() {
        clear_test_storage();
        let service = UserService::default();
        assert!(!service.is_authenticated());
    }
    
    #[wasm_bindgen_test]
    async fn test_login_success() {
        clear_test_storage();
        let mut service = UserService::new();
        
        // Login with test credentials
        let result = service.login("test_user".to_string(), "test_password".to_string()).await;
        assert!(result.is_ok(), "Login should succeed");
        
        // Check authentication state
        assert!(service.is_authenticated());
        assert_eq!(service.user_id(), Some("test_user"));
        
        // Should have signing key
        let key = service.signing_key();
        assert!(key.is_some());
        assert_eq!(key.unwrap().len(), 32); // PBKDF2 produces 32-byte key
        
        // Credentials should be stored
        let creds = service.get_credentials();
        assert!(creds.is_some());
        assert_eq!(creds.unwrap().user_id, "test_user");
        assert_eq!(creds.unwrap().password, "test_password");
    }
    
    #[wasm_bindgen_test]
    async fn test_login_persistence() {
        clear_test_storage();
        let mut service = UserService::new();
        
        // Login
        service.login("persistent_user".to_string(), "persistent_pass".to_string()).await.unwrap();
        
        // Create new service instance (should load from session)
        let new_service = UserService::new();
        assert!(new_service.is_authenticated());
        assert_eq!(new_service.user_id(), Some("persistent_user"));
        
        // Signing key should be restored
        let key = new_service.signing_key();
        assert!(key.is_some());
        assert_eq!(key.unwrap().len(), 32);
    }
    
    #[wasm_bindgen_test]
    async fn test_logout() {
        clear_test_storage();
        let mut service = UserService::new();
        
        // Login first
        service.login("logout_user".to_string(), "logout_pass".to_string()).await.unwrap();
        assert!(service.is_authenticated());
        
        // Logout
        let result = service.logout();
        assert!(result.is_ok());
        assert!(!service.is_authenticated());
        assert!(service.user_id().is_none());
        assert!(service.signing_key().is_none());
        
        // Session storage should be cleared
        let new_service = UserService::new();
        assert!(!new_service.is_authenticated());
    }
    
    #[wasm_bindgen_test]
    async fn test_signing_key_consistency() {
        clear_test_storage();
        let mut service = UserService::new();
        
        // Login
        service.login("key_test_user".to_string(), "key_test_pass".to_string()).await.unwrap();
        let key1 = service.signing_key().unwrap().to_vec();
        
        // Key should be consistent across service instances
        let service2 = UserService::new();
        let key2 = service2.signing_key().unwrap().to_vec();
        assert_eq!(key1, key2, "Signing key should be consistent across instances");
    }
    
    #[wasm_bindgen_test]
    async fn test_multiple_login_overwrites() {
        clear_test_storage();
        let mut service = UserService::new();
        
        // First login
        service.login("user1".to_string(), "pass1".to_string()).await.unwrap();
        assert_eq!(service.user_id(), Some("user1"));
        let key1 = service.signing_key().unwrap().to_vec();
        
        // Second login (different user)
        service.login("user2".to_string(), "pass2".to_string()).await.unwrap();
        assert_eq!(service.user_id(), Some("user2"));
        let key2 = service.signing_key().unwrap().to_vec();
        
        // Keys should be different for different users
        assert_ne!(key1, key2, "Different users should have different signing keys");
    }
    
    #[wasm_bindgen_test]
    async fn test_get_credentials_returns_clone() {
        clear_test_storage();
        let mut service = UserService::new();
        
        service.login("clone_test".to_string(), "clone_pass".to_string()).await.unwrap();
        
        let creds1 = service.get_credentials().unwrap();
        let creds2 = service.get_credentials().unwrap();
        
        // Should be the same data
        assert_eq!(creds1.user_id, creds2.user_id);
        assert_eq!(creds1.password, creds2.password);
        assert_eq!(creds1.signing_key, creds2.signing_key);
    }
    
    #[wasm_bindgen_test]
    async fn test_signing_key_derivation_deterministic() {
        clear_test_storage();
        
        // Login twice with same credentials
        let mut service1 = UserService::new();
        service1.login("deterministic_user".to_string(), "deterministic_pass".to_string()).await.unwrap();
        let key1 = service1.signing_key().unwrap().to_vec();
        
        service1.logout().unwrap();
        
        let mut service2 = UserService::new();
        service2.login("deterministic_user".to_string(), "deterministic_pass".to_string()).await.unwrap();
        let key2 = service2.signing_key().unwrap().to_vec();
        
        // Same user/password should produce same signing key (deterministic salt)
        assert_eq!(key1, key2, "Signing key derivation should be deterministic");
    }
}
