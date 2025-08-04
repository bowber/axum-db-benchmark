use async_trait::async_trait;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use std::sync::Arc;

use crate::database::{CreateUser, Database, UpdateUser, User};
use crate::err::ServerError;

#[derive(Clone)]
pub struct SqliteDatabase {
    pool: Arc<Pool<SqliteConnectionManager>>,
}

#[async_trait]
impl Database for SqliteDatabase {
    type Error = ServerError;

    async fn init() -> Result<Self, Self::Error> {
        #[cfg(not(test))]
        let manager = SqliteConnectionManager::file("my_database.db");
        #[cfg(test)]
        let manager = SqliteConnectionManager::memory();

        let pool = r2d2::Pool::builder()
            .build(manager.with_init(|c| {
                c.pragma_update(None, "foreign_keys", "ON")?;
                c.pragma_update(None, "journal_mode", "WAL2")?;
                c.pragma_update(None, "synchronous", "NORMAL")?;
                Ok(())
            }))
            .map_err(|e| ServerError::new(&format!("Failed to create connection pool: {}", e)))?;

        let conn = pool.get().map_err(|e| ServerError::new(&format!("Failed to get connection: {}", e)))?;
        
        // Create the users table if it doesn't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL UNIQUE,
                age INTEGER DEFAULT 0
            );",
            params![],
        ).map_err(|e| ServerError::new(&format!("Failed to create table: {}", e)))?;
        
        // Create index on username for faster lookups
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_username ON users (username);",
            params![],
        ).map_err(|e| ServerError::new(&format!("Failed to create index: {}", e)))?;

        Ok(SqliteDatabase {
            pool: Arc::new(pool),
        })
    }

    async fn create_user(&self, user: CreateUser) -> Result<String, Self::Error> {
        let conn = self.pool.get()?;
        let result = conn.execute(
            "INSERT INTO users (username) VALUES (?);",
            params![user.username],
        );
        let changed_row = result.map_err(|e| format!("Create user `{}` error: {}", user.username, e))?;
        if changed_row == 0 {
            return Err(format!("Error creating user: No rows changed").into());
        }
        Ok(format!("User created with username: {}", user.username))
    }

    async fn get_user(&self, username: String) -> Result<User, Self::Error> {
        let conn = self.pool.get()?;
        let result = conn.query_one(
            "SELECT id, username, age FROM users WHERE username = ?;",
            params![username],
            |row| {
                Ok(User {
                    id: row.get(0)?,
                    username: row.get(1)?,
                    age: row.get(2)?,
                })
            },
        );
        match result {
            Ok(user) => Ok(user),
            Err(e) => Err(format!("Get user by username error: {}", e).into()),
        }
    }

    async fn update_user(&self, username: String, update: UpdateUser) -> Result<(), Self::Error> {
        let conn = self.pool.get()?;
        let statement = conn.execute(
            "UPDATE users SET age = ? WHERE username = ?;",
            params![update.age, username],
        );
        match statement {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Update user by username error: {}", e).into()),
        }
    }

    async fn delete_user(&self, username: String) -> Result<(), Self::Error> {
        let conn = self.pool.get()?;
        let statement = conn.execute("DELETE FROM users WHERE username = ?;", params![username]);
        match statement {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Delete user by username error: {}", e).into()),
        }
    }
}