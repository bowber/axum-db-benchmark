use async_trait::async_trait;
use redis::{AsyncCommands, Client};
use serde_json;
use std::sync::Arc;

use crate::database::{CreateUser, Database, UpdateUser, User};
use crate::err::ServerError;

#[derive(Clone)]
pub struct RedisDatabase {
    client: Arc<Client>,
}

#[async_trait]
impl Database for RedisDatabase {
    type Error = ServerError;

    async fn init() -> Result<Self, Self::Error> {
        // Use environment variable or default to localhost for connection
        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string());
        
        let client = Client::open(redis_url)
            .map_err(|e| ServerError::new(&format!("Failed to create Redis client: {}", e)))?;

        Ok(RedisDatabase {
            client: Arc::new(client),
        })
    }

    async fn create_user(&self, user: CreateUser) -> Result<String, Self::Error> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| ServerError::new(&format!("Failed to get Redis connection: {}", e)))?;
        
        // Check if user already exists
        let exists: bool = conn.exists(&format!("user:{}", user.username)).await
            .map_err(|e| ServerError::new(&format!("Failed to check user existence: {}", e)))?;
        
        if exists {
            return Err(format!("User already exists: {}", user.username).into());
        }
        
        // Generate a simple ID (in a real system you'd use a proper ID generator)
        let id: u64 = conn.incr("user:id_counter", 1).await
            .map_err(|e| ServerError::new(&format!("Failed to generate user ID: {}", e)))?;
        
        let user_data = User {
            id,
            username: user.username.clone(),
            age: 0,
        };
        
        let user_json = serde_json::to_string(&user_data)
            .map_err(|e| ServerError::new(&format!("Failed to serialize user: {}", e)))?;
        
        // Store user data with username as key
        let _: () = conn.set(&format!("user:{}", user.username), &user_json).await
            .map_err(|e| ServerError::new(&format!("Failed to store user: {}", e)))?;
        
        // Also store username with id as key for potential id-based lookups
        let _: () = conn.set(&format!("user_id:{}", id), &user.username).await
            .map_err(|e| ServerError::new(&format!("Failed to store user ID mapping: {}", e)))?;

        Ok(format!("User created with username: {}", user.username))
    }

    async fn get_user(&self, username: String) -> Result<User, Self::Error> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| ServerError::new(&format!("Failed to get Redis connection: {}", e)))?;
        
        let user_json: Option<String> = conn.get(&format!("user:{}", username)).await
            .map_err(|e| ServerError::new(&format!("Failed to get user: {}", e)))?;
        
        match user_json {
            Some(json) => {
                let user: User = serde_json::from_str(&json)
                    .map_err(|e| ServerError::new(&format!("Failed to deserialize user: {}", e)))?;
                Ok(user)
            },
            None => Err(format!("User not found: {}", username).into()),
        }
    }

    async fn update_user(&self, username: String, update: UpdateUser) -> Result<(), Self::Error> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| ServerError::new(&format!("Failed to get Redis connection: {}", e)))?;
        
        // Get existing user
        let user_json: Option<String> = conn.get(&format!("user:{}", username)).await
            .map_err(|e| ServerError::new(&format!("Failed to get user: {}", e)))?;
        
        match user_json {
            Some(json) => {
                let mut user: User = serde_json::from_str(&json)
                    .map_err(|e| ServerError::new(&format!("Failed to deserialize user: {}", e)))?;
                
                user.age = update.age;
                
                let updated_json = serde_json::to_string(&user)
                    .map_err(|e| ServerError::new(&format!("Failed to serialize updated user: {}", e)))?;
                
                let _: () = conn.set(&format!("user:{}", username), &updated_json).await
                    .map_err(|e| ServerError::new(&format!("Failed to update user: {}", e)))?;
                
                Ok(())
            },
            None => Err(format!("User not found: {}", username).into()),
        }
    }

    async fn delete_user(&self, username: String) -> Result<(), Self::Error> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| ServerError::new(&format!("Failed to get Redis connection: {}", e)))?;
        
        // First get the user to find their ID
        let user_json: Option<String> = conn.get(&format!("user:{}", username)).await
            .map_err(|e| ServerError::new(&format!("Failed to get user: {}", e)))?;
        
        match user_json {
            Some(json) => {
                let user: User = serde_json::from_str(&json)
                    .map_err(|e| ServerError::new(&format!("Failed to deserialize user: {}", e)))?;
                
                // Delete both the user data and the ID mapping
                let _: () = conn.del(&format!("user:{}", username)).await
                    .map_err(|e| ServerError::new(&format!("Failed to delete user: {}", e)))?;
                
                let _: () = conn.del(&format!("user_id:{}", user.id)).await
                    .map_err(|e| ServerError::new(&format!("Failed to delete user ID mapping: {}", e)))?;
                
                Ok(())
            },
            None => Err(format!("User not found: {}", username).into()),
        }
    }
}