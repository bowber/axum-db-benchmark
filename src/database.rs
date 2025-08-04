use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub age: u32,
}

#[derive(Deserialize, Clone)]
pub struct CreateUser {
    pub username: String,
}

#[derive(Deserialize, Clone)]
pub struct UpdateUser {
    pub age: u32,
}

#[async_trait]
pub trait Database: Send + Sync + Clone {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn init() -> Result<Self, Self::Error>;
    async fn create_user(&self, user: CreateUser) -> Result<String, Self::Error>;
    async fn get_user(&self, username: String) -> Result<User, Self::Error>;
    async fn update_user(&self, username: String, update: UpdateUser) -> Result<(), Self::Error>;
    async fn delete_user(&self, username: String) -> Result<(), Self::Error>;
}