use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse,
};
use std::fmt;

#[derive(Debug)]
pub struct ServerError {
    pub message: String,
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ServerError {}

impl PartialEq for ServerError {
    fn eq(&self, other: &Self) -> bool {
        self.message == other.message
    }
}

impl ServerError {
    pub fn new(message: &str) -> Self {
        ServerError {
            message: message.to_string(),
        }
    }
}

impl From<String> for ServerError {
    fn from(message: String) -> Self {
        ServerError { message }
    }
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response<Body> {
        // Log the error message
        tracing::error!("Server error: {}", self.message);
        let body = Body::from(self.message);
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}

impl From<r2d2::Error> for ServerError {
    fn from(err: r2d2::Error) -> Self {
        ServerError::new(&format!("Database connection error: {}", err))
    }
}

impl From<rusqlite::Error> for ServerError {
    fn from(err: rusqlite::Error) -> Self {
        ServerError::new(&format!("SQLite error: {}", err))
    }
}

impl From<postgres::Error> for ServerError {
    fn from(err: postgres::Error) -> Self {
        ServerError::new(&format!("PostgreSQL error: {}", err))
    }
}

impl From<mysql_async::Error> for ServerError {
    fn from(err: mysql_async::Error) -> Self {
        ServerError::new(&format!("MySQL error: {}", err))
    }
}

impl From<redis::RedisError> for ServerError {
    fn from(err: redis::RedisError) -> Self {
        ServerError::new(&format!("Redis error: {}", err))
    }
}

impl From<mongodb::error::Error> for ServerError {
    fn from(err: mongodb::error::Error) -> Self {
        ServerError::new(&format!("MongoDB error: {}", err))
    }
}
