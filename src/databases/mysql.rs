use async_trait::async_trait;
use mysql_async::{prelude::*, Pool, OptsBuilder};
use std::sync::Arc;

use crate::database::{CreateUser, Database, UpdateUser, User};
use crate::err::ServerError;

#[derive(Clone)]
pub struct MySqlDatabase {
    pool: Arc<Pool>,
}

#[async_trait]
impl Database for MySqlDatabase {
    type Error = ServerError;

    async fn init() -> Result<Self, Self::Error> {
        // Use environment variable or default to localhost for connection
        let database_url = std::env::var("MYSQL_URL")
            .unwrap_or_else(|_| "mysql://root:password@localhost:3306/benchmark".to_string());
        
        let opts = OptsBuilder::from_opts(
            database_url.parse::<mysql_async::Opts>()
                .map_err(|e| ServerError::new(&format!("Invalid MySQL URL: {}", e)))?
        );
        
        let pool = Pool::new(opts);
        let pool = Arc::new(pool);
        
        // Get a connection to set up the table
        let mut conn = pool.get_conn().await
            .map_err(|e| ServerError::new(&format!("Failed to get MySQL connection: {}", e)))?;
        
        // Create the users table if it doesn't exist
        conn.query_drop(
            "CREATE TABLE IF NOT EXISTS users (
                id INT AUTO_INCREMENT PRIMARY KEY,
                username VARCHAR(255) NOT NULL UNIQUE,
                age INT DEFAULT 0
            );"
        ).await.map_err(|e| ServerError::new(&format!("Failed to create MySQL table: {}", e)))?;
        
        // Create index on username for faster lookups
        conn.query_drop(
            "CREATE INDEX IF NOT EXISTS idx_username ON users (username);"
        ).await.map_err(|e| ServerError::new(&format!("Failed to create MySQL index: {}", e)))?;

        drop(conn);

        Ok(MySqlDatabase { pool })
    }

    async fn create_user(&self, user: CreateUser) -> Result<String, Self::Error> {
        let mut conn = self.pool.get_conn().await
            .map_err(|e| ServerError::new(&format!("Failed to get MySQL connection: {}", e)))?;
        
        let result = conn.exec_drop(
            "INSERT INTO users (username) VALUES (?);",
            (user.username.clone(),)
        ).await;
        
        match result {
            Ok(_) => Ok(format!("User created with username: {}", user.username)),
            Err(e) => Err(format!("Create user `{}` error: {}", user.username, e).into()),
        }
    }

    async fn get_user(&self, username: String) -> Result<User, Self::Error> {
        let mut conn = self.pool.get_conn().await
            .map_err(|e| ServerError::new(&format!("Failed to get MySQL connection: {}", e)))?;
        
        let result: Option<(u32, String, u32)> = conn.exec_first(
            "SELECT id, username, age FROM users WHERE username = ?;",
            (username.clone(),)
        ).await.map_err(|e| format!("Get user by username error: {}", e))?;
        
        match result {
            Some((id, username, age)) => Ok(User {
                id: id as u64,
                username,
                age,
            }),
            None => Err(format!("User not found: {}", username).into()),
        }
    }

    async fn update_user(&self, username: String, update: UpdateUser) -> Result<(), Self::Error> {
        let mut conn = self.pool.get_conn().await
            .map_err(|e| ServerError::new(&format!("Failed to get MySQL connection: {}", e)))?;
        
        let result = conn.exec_drop(
            "UPDATE users SET age = ? WHERE username = ?;",
            (update.age, username.clone())
        ).await;
        
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Update user by username error: {}", e).into()),
        }
    }

    async fn delete_user(&self, username: String) -> Result<(), Self::Error> {
        let mut conn = self.pool.get_conn().await
            .map_err(|e| ServerError::new(&format!("Failed to get MySQL connection: {}", e)))?;
        
        let result = conn.exec_drop(
            "DELETE FROM users WHERE username = ?;",
            (username.clone(),)
        ).await;
        
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Delete user by username error: {}", e).into()),
        }
    }
}