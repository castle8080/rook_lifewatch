use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct User {
    /// Unique user identifier
    pub id: String,
    
    /// Display name for the user
    pub name: String,
    
    /// PBKDF2-derived key from password (base64-encoded, 256-bit)
    pub password_hash: String,
    
    /// Permission level (e.g., "admin", "viewer", "operator")
    pub permission_level: String,
    
    /// When the user account was created
    pub created_at: DateTime<Utc>,
}
