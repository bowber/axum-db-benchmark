use async_trait::async_trait;
use r2d2::Pool;
use r2d2_postgres::{postgres::NoTls as R2D2NoTls, PostgresConnectionManager};
use std::sync::Arc;

use crate::database::{CreateUser, Database, UpdateUser, User};
use crate::err::ServerError;

#[derive(Clone)]
pub struct PostgresDatabase {
    pool: Arc<Pool<PostgresConnectionManager<R2D2NoTls>>>,
}

#[async_trait]
impl Database for PostgresDatabase {
    type Error = ServerError;

    async fn init() -> Result<Self, Self::Error> {
        // Use environment variable or default to localhost for connection
        let database_url = std::env::var("POSTGRES_URL")
            .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/benchmark".to_string());
        
        let manager = PostgresConnectionManager::new(
            database_url.parse()
                .map_err(|e| ServerError::new(&format!("Invalid PostgreSQL URL: {}", e)))?,
            R2D2NoTls,
        );

        let pool = r2d2::Pool::builder()
            .build(manager)
            .map_err(|e| ServerError::new(&format!("Failed to create PostgreSQL connection pool: {}", e)))?;

        let mut conn = pool.get().map_err(|e| ServerError::new(&format!("Failed to get PostgreSQL connection: {}", e)))?;
        
        // Create the users table if it doesn't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id SERIAL PRIMARY KEY,
                username VARCHAR(255) NOT NULL UNIQUE,
                age INTEGER DEFAULT 0
            );",
            &[],
        ).map_err(|e| ServerError::new(&format!("Failed to create PostgreSQL table: {}", e)))?;
        
        // Create index on username for faster lookups
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_username ON users (username);",
            &[],
        ).map_err(|e| ServerError::new(&format!("Failed to create PostgreSQL index: {}", e)))?;

        Ok(PostgresDatabase {
            pool: Arc::new(pool),
        })
    }

    async fn create_user(&self, user: CreateUser) -> Result<String, Self::Error> {
        let mut conn = self.pool.get()?;
        let result = conn.execute(
            "INSERT INTO users (username) VALUES ($1);",
            &[&user.username],
        );
        let changed_row = result.map_err(|e| format!("Create user `{}` error: {}", user.username, e))?;
        if changed_row == 0 {
            return Err(format!("Error creating user: No rows changed").into());
        }
        Ok(format!("User created with username: {}", user.username))
    }

    async fn get_user(&self, username: String) -> Result<User, Self::Error> {
        let mut conn = self.pool.get()?;
        let rows = conn.query(
            "SELECT id, username, age FROM users WHERE username = $1;",
            &[&username],
        ).map_err(|e| format!("Get user by username error: {}", e))?;
        
        if rows.is_empty() {
            return Err(format!("User not found: {}", username).into());
        }
        
        let row = &rows[0];
        Ok(User {
            id: row.get::<_, i32>(0) as u64,
            username: row.get(1),
            age: row.get::<_, i32>(2) as u32,
        })
    }

    async fn update_user(&self, username: String, update: UpdateUser) -> Result<(), Self::Error> {
        let mut conn = self.pool.get()?;
        let statement = conn.execute(
            "UPDATE users SET age = $1 WHERE username = $2;",
            &[&(update.age as i32), &username],
        );
        match statement {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Update user by username error: {}", e).into()),
        }
    }

    async fn delete_user(&self, username: String) -> Result<(), Self::Error> {
        let mut conn = self.pool.get()?;
        let statement = conn.execute("DELETE FROM users WHERE username = $1;", &[&username]);
        match statement {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Delete user by username error: {}", e).into()),
        }
    }
}