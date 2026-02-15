use std::sync::Arc;
use rook_lw_image_repo::user_repo::UserRepository;
use rook_lw_models::user::User;
use crate::{RookLWAdminResult, RookLWAdminError};
use crate::services::password_hash_service::PasswordHashService;
use chrono::Utc;
use uuid::Uuid;

pub struct UserService {
    user_repo: Arc<Box<dyn UserRepository>>,
}

impl UserService {
    pub fn new(user_repo: Arc<Box<dyn UserRepository>>) -> Self {
        Self { user_repo }
    }
    
    /// Create a new user with the given username and password
    pub fn create_user(
        &self,
        name: &str,
        password: &str,
        permission_level: &str,
    ) -> RookLWAdminResult<User> {
        // Check if user already exists
        if self.user_repo.get_user_by_name(name)?.is_some() {
            return Err(RookLWAdminError::Other(
                format!("User '{}' already exists", name)
            ));
        }
        
        let user_id = Uuid::new_v4().to_string();
        
        // Hash the password
        let password_hash = PasswordHashService::hash_password(&user_id, password)?;
        
        let user = User {
            id: user_id,
            name: name.to_string(),
            password_hash,
            permission_level: permission_level.to_string(),
            created_at: Utc::now(),
        };
        
        self.user_repo.create_user(&user)?;
        
        Ok(user)
    }
    
    /// Authenticate a user by username and password
    pub fn authenticate(&self, name: &str, password: &str) -> RookLWAdminResult<Option<User>> {
        let user = self.user_repo.get_user_by_name(name)?;
        
        if let Some(user) = user {
            if PasswordHashService::verify_password(&user.id, password, &user.password_hash)? {
                return Ok(Some(user));
            }
        }
        
        Ok(None)
    }
    
    /// Get user by ID
    pub fn get_user(&self, user_id: &str) -> RookLWAdminResult<Option<User>> {
        Ok(self.user_repo.get_user(user_id)?)
    }
    
    /// Change a user's password
    pub fn change_password(&self, user_id: &str, new_password: &str) -> RookLWAdminResult<()> {
        let mut user = self.user_repo.get_user(user_id)?
            .ok_or_else(|| RookLWAdminError::Other(format!("User not found: {}", user_id)))?;
        
        user.password_hash = PasswordHashService::hash_password(&user.id, new_password)?;
        self.user_repo.update_user(&user)?;
        
        Ok(())
    }
    
    /// List all users
    pub fn list_users(&self) -> RookLWAdminResult<Vec<User>> {
        Ok(self.user_repo.list_users()?)
    }
    
    /// Delete a user
    pub fn delete_user(&self, user_id: &str) -> RookLWAdminResult<()> {
        Ok(self.user_repo.delete_user(user_id)?)
    }
    
    /// Initialize a default admin user if no users exist
    pub fn initialize_default_admin(&self) -> RookLWAdminResult<()> {
        use tracing::{info, warn};
        
        if !self.user_repo.has_any_users()? {
            info!("No users found. Creating default admin user.");
            
            let admin_id = "admin".to_string();
            
            // Use hostname as the default password
            let hostname = hostname::get()
                .map_err(|e| RookLWAdminError::Io(format!("Failed to get hostname: {}", e)))?
                .into_string()
                .unwrap_or_else(|_| "localhost".to_string());
            
            let password_hash = PasswordHashService::hash_password(&admin_id, &hostname)?;
            
            let admin_user = User {
                id: admin_id,
                name: "admin".to_string(),
                password_hash,
                permission_level: "admin".to_string(),
                created_at: Utc::now(),
            };
            
            self.user_repo.create_user(&admin_user)?;
            
            warn!("Created default admin user with password: {}", hostname);
            warn!("PLEASE CHANGE THIS PASSWORD IMMEDIATELY!");
        }
        
        Ok(())
    }
}
