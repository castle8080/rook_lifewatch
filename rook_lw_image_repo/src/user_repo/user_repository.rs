use rook_lw_models::user::User;
use crate::ImageRepoResult;

pub trait UserRepository: Send + Sync {
    /// Create a new user
    fn create_user(&self, user: &User) -> ImageRepoResult<()>;
    
    /// Get user by ID
    fn get_user(&self, user_id: &str) -> ImageRepoResult<Option<User>>;
    
    /// Get user by name (for login)
    fn get_user_by_name(&self, name: &str) -> ImageRepoResult<Option<User>>;
    
    /// Update existing user
    fn update_user(&self, user: &User) -> ImageRepoResult<()>;
    
    /// Delete user by ID
    fn delete_user(&self, user_id: &str) -> ImageRepoResult<()>;
    
    /// List all users
    fn list_users(&self) -> ImageRepoResult<Vec<User>>;
    
    /// Check if any users exist in the system
    fn has_any_users(&self) -> ImageRepoResult<bool>;
}
