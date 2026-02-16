use web_sys::{window, Storage};
use serde::{Serialize, Deserialize};
use crate::RookLWAppResult;

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
        let signing_key = super::RequestSigner::derive_signing_key(&user_id, &password).await?;
        
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
