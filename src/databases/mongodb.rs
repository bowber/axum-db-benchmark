use async_trait::async_trait;
use mongodb::{
    bson::{doc, oid::ObjectId},
    options::{ClientOptions, IndexOptions},
    Client, Collection, IndexModel,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::database::{CreateUser, Database, UpdateUser, User};
use crate::err::ServerError;

#[derive(Serialize, Deserialize)]
struct MongoUser {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub username: String,
    pub age: u32,
}

#[derive(Clone)]
pub struct MongoDatabase {
    collection: Arc<Collection<MongoUser>>,
}

#[async_trait]
impl Database for MongoDatabase {
    type Error = ServerError;

    async fn init() -> Result<Self, Self::Error> {
        // Use environment variable or default to localhost for connection
        let mongo_url = std::env::var("MONGO_URL")
            .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
        
        let client_options = ClientOptions::parse(&mongo_url).await
            .map_err(|e| ServerError::new(&format!("Failed to parse MongoDB URL: {}", e)))?;
        
        let client = Client::with_options(client_options)
            .map_err(|e| ServerError::new(&format!("Failed to create MongoDB client: {}", e)))?;
        
        let database = client.database("benchmark");
        let collection = database.collection::<MongoUser>("users");
        
        // Create index on username for faster lookups
        let index = IndexModel::builder()
            .keys(doc! { "username": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build();
        
        collection.create_index(index).await
            .map_err(|e| ServerError::new(&format!("Failed to create MongoDB index: {}", e)))?;

        Ok(MongoDatabase {
            collection: Arc::new(collection),
        })
    }

    async fn create_user(&self, user: CreateUser) -> Result<String, Self::Error> {
        let mongo_user = MongoUser {
            id: None,
            username: user.username.clone(),
            age: 0,
        };
        
        let result = self.collection.insert_one(mongo_user).await;
        
        match result {
            Ok(_) => Ok(format!("User created with username: {}", user.username)),
            Err(e) => {
                if e.to_string().contains("duplicate key") {
                    Err(format!("User already exists: {}", user.username).into())
                } else {
                    Err(format!("Create user `{}` error: {}", user.username, e).into())
                }
            }
        }
    }

    async fn get_user(&self, username: String) -> Result<User, Self::Error> {
        let filter = doc! { "username": &username };
        
        let result = self.collection.find_one(filter).await
            .map_err(|e| format!("Get user by username error: {}", e))?;
        
        match result {
            Some(mongo_user) => {
                // Use a simple hash of the ObjectId as the ID
                let id = mongo_user.id
                    .map(|oid| {
                        use std::collections::hash_map::DefaultHasher;
                        use std::hash::{Hash, Hasher};
                        let mut hasher = DefaultHasher::new();
                        oid.hash(&mut hasher);
                        hasher.finish()
                    })
                    .unwrap_or(0);
                
                Ok(User {
                    id,
                    username: mongo_user.username,
                    age: mongo_user.age,
                })
            },
            None => Err(format!("User not found: {}", username).into()),
        }
    }

    async fn update_user(&self, username: String, update: UpdateUser) -> Result<(), Self::Error> {
        let filter = doc! { "username": &username };
        let update_doc = doc! { "$set": { "age": update.age as i32 } };
        
        let result = self.collection.update_one(filter, update_doc).await
            .map_err(|e| format!("Update user by username error: {}", e))?;
        
        if result.matched_count == 0 {
            return Err(format!("User not found: {}", username).into());
        }
        
        Ok(())
    }

    async fn delete_user(&self, username: String) -> Result<(), Self::Error> {
        let filter = doc! { "username": &username };
        
        let result = self.collection.delete_one(filter).await
            .map_err(|e| format!("Delete user by username error: {}", e))?;
        
        if result.deleted_count == 0 {
            return Err(format!("User not found: {}", username).into());
        }
        
        Ok(())
    }
}