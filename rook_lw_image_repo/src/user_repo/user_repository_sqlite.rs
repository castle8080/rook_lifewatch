use super::UserRepository;
use crate::ImageRepoResult;
use rook_lw_models::user::User;
use rusqlite::{params, Row};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use tracing::info;

pub struct UserRepositorySqlite {
    pool: Pool<SqliteConnectionManager>,
}

impl UserRepositorySqlite {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> ImageRepoResult<Self> {
        let mut repo = Self { pool };
        repo.initialize()?;
        Ok(repo)
    }
    
    fn initialize(&mut self) -> ImageRepoResult<()> {
        let conn = self.pool.get()?;
        info!("Initializing user_repository database");
        
        conn.execute_batch(r#"
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                permission_level TEXT NOT NULL,
                created_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_user_name ON users(name);
        "#)?;
        
        Ok(())
    }
    
    fn row_to_user(row: &Row) -> ImageRepoResult<User> {
        let id: String = row.get(0)?;
        let name: String = row.get(1)?;
        let password_hash: String = row.get(2)?;
        let permission_level: String = row.get(3)?;
        let created_at_str: String = row.get(4)?;
        
        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)?
            .with_timezone(&chrono::Utc);
        
        Ok(User {
            id,
            name,
            password_hash,
            permission_level,
            created_at,
        })
    }
}

impl UserRepository for UserRepositorySqlite {
    fn create_user(&self, user: &User) -> ImageRepoResult<()> {
        let conn = self.pool.get()?;
        conn.execute(
            r#"INSERT INTO users (id, name, password_hash, permission_level, created_at)
               VALUES (?1, ?2, ?3, ?4, ?5)"#,
            params![
                &user.id,
                &user.name,
                &user.password_hash,
                &user.permission_level,
                user.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }
    
    fn get_user(&self, user_id: &str) -> ImageRepoResult<Option<User>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, password_hash, permission_level, created_at FROM users WHERE id = ?1"
        )?;
        let mut rows = stmt.query(params![user_id])?;
        
        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_user(row)?))
        } else {
            Ok(None)
        }
    }
    
    fn get_user_by_name(&self, name: &str) -> ImageRepoResult<Option<User>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, password_hash, permission_level, created_at FROM users WHERE name = ?1"
        )?;
        let mut rows = stmt.query(params![name])?;
        
        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_user(row)?))
        } else {
            Ok(None)
        }
    }
    
    fn update_user(&self, user: &User) -> ImageRepoResult<()> {
        let conn = self.pool.get()?;
        conn.execute(
            r#"UPDATE users SET name = ?2, password_hash = ?3, permission_level = ?4
               WHERE id = ?1"#,
            params![
                &user.id,
                &user.name,
                &user.password_hash,
                &user.permission_level,
            ],
        )?;
        Ok(())
    }
    
    fn delete_user(&self, user_id: &str) -> ImageRepoResult<()> {
        let conn = self.pool.get()?;
        conn.execute("DELETE FROM users WHERE id = ?1", params![user_id])?;
        Ok(())
    }
    
    fn list_users(&self) -> ImageRepoResult<Vec<User>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, password_hash, permission_level, created_at FROM users ORDER BY created_at"
        )?;
        let rows = stmt.query_map([], |row| Ok(Self::row_to_user(row)))?;
        
        let mut users = Vec::new();
        for row_result in rows {
            users.push(row_result??);
        }
        
        Ok(users)
    }
    
    fn has_any_users(&self) -> ImageRepoResult<bool> {
        let conn = self.pool.get()?;
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))?;
        Ok(count > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_pool() -> Pool<SqliteConnectionManager> {
        let manager = SqliteConnectionManager::memory();
        Pool::new(manager).expect("Failed to create pool")
    }

    fn create_test_user(id: &str, name: &str) -> User {
        User {
            id: id.to_string(),
            name: name.to_string(),
            password_hash: "test_hash:abc123".to_string(),
            permission_level: "viewer".to_string(),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_create_and_get_user() {
        let pool = create_test_pool();
        let repo = UserRepositorySqlite::new(pool).expect("Failed to create repo");
        
        let user = create_test_user("user1", "alice");
        
        // Create user
        repo.create_user(&user).expect("Failed to create user");
        
        // Get user by ID
        let retrieved = repo.get_user("user1").expect("Failed to get user");
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, "user1");
        assert_eq!(retrieved.name, "alice");
        assert_eq!(retrieved.password_hash, "test_hash:abc123");
        assert_eq!(retrieved.permission_level, "viewer");
    }

    #[test]
    fn test_get_user_by_name() {
        let pool = create_test_pool();
        let repo = UserRepositorySqlite::new(pool).expect("Failed to create repo");
        
        let user = create_test_user("user2", "bob");
        repo.create_user(&user).expect("Failed to create user");
        
        // Get user by name
        let retrieved = repo.get_user_by_name("bob").expect("Failed to get user");
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, "user2");
        assert_eq!(retrieved.name, "bob");
    }

    #[test]
    fn test_get_nonexistent_user() {
        let pool = create_test_pool();
        let repo = UserRepositorySqlite::new(pool).expect("Failed to create repo");
        
        let retrieved = repo.get_user("nonexistent").expect("Failed to query");
        assert!(retrieved.is_none());
        
        let retrieved = repo.get_user_by_name("nonexistent").expect("Failed to query");
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_update_user() {
        let pool = create_test_pool();
        let repo = UserRepositorySqlite::new(pool).expect("Failed to create repo");
        
        let mut user = create_test_user("user3", "charlie");
        repo.create_user(&user).expect("Failed to create user");
        
        // Update user
        user.name = "charles".to_string();
        user.password_hash = "new_hash:xyz789".to_string();
        user.permission_level = "admin".to_string();
        
        repo.update_user(&user).expect("Failed to update user");
        
        // Verify update
        let retrieved = repo.get_user("user3").expect("Failed to get user").unwrap();
        assert_eq!(retrieved.name, "charles");
        assert_eq!(retrieved.password_hash, "new_hash:xyz789");
        assert_eq!(retrieved.permission_level, "admin");
    }

    #[test]
    fn test_delete_user() {
        let pool = create_test_pool();
        let repo = UserRepositorySqlite::new(pool).expect("Failed to create repo");
        
        let user = create_test_user("user4", "dave");
        repo.create_user(&user).expect("Failed to create user");
        
        // Verify user exists
        assert!(repo.get_user("user4").expect("Failed to get user").is_some());
        
        // Delete user
        repo.delete_user("user4").expect("Failed to delete user");
        
        // Verify user is gone
        assert!(repo.get_user("user4").expect("Failed to get user").is_none());
    }

    #[test]
    fn test_list_users() {
        let pool = create_test_pool();
        let repo = UserRepositorySqlite::new(pool).expect("Failed to create repo");
        
        // Create multiple users
        let user1 = create_test_user("user5", "eve");
        let user2 = create_test_user("user6", "frank");
        let user3 = create_test_user("user7", "grace");
        
        repo.create_user(&user1).expect("Failed to create user1");
        repo.create_user(&user2).expect("Failed to create user2");
        repo.create_user(&user3).expect("Failed to create user3");
        
        // List all users
        let users = repo.list_users().expect("Failed to list users");
        assert_eq!(users.len(), 3);
        
        // Verify names
        let names: Vec<String> = users.iter().map(|u| u.name.clone()).collect();
        assert!(names.contains(&"eve".to_string()));
        assert!(names.contains(&"frank".to_string()));
        assert!(names.contains(&"grace".to_string()));
    }

    #[test]
    fn test_has_any_users() {
        let pool = create_test_pool();
        let repo = UserRepositorySqlite::new(pool).expect("Failed to create repo");
        
        // Initially no users
        assert!(!repo.has_any_users().expect("Failed to check users"));
        
        // Add a user
        let user = create_test_user("user8", "helen");
        repo.create_user(&user).expect("Failed to create user");
        
        // Now should have users
        assert!(repo.has_any_users().expect("Failed to check users"));
    }

    #[test]
    fn test_unique_name_constraint() {
        let pool = create_test_pool();
        let repo = UserRepositorySqlite::new(pool).expect("Failed to create repo");
        
        let user1 = create_test_user("user9", "duplicate_name");
        let user2 = create_test_user("user10", "duplicate_name");
        
        // First user should succeed
        repo.create_user(&user1).expect("Failed to create first user");
        
        // Second user with same name should fail
        let result = repo.create_user(&user2);
        assert!(result.is_err());
    }

    #[test]
    fn test_unique_id_constraint() {
        let pool = create_test_pool();
        let repo = UserRepositorySqlite::new(pool).expect("Failed to create repo");
        
        let user1 = create_test_user("duplicate_id", "user_a");
        let user2 = create_test_user("duplicate_id", "user_b");
        
        // First user should succeed
        repo.create_user(&user1).expect("Failed to create first user");
        
        // Second user with same ID should fail
        let result = repo.create_user(&user2);
        assert!(result.is_err());
    }
}
