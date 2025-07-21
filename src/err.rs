use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse,
};

#[derive(PartialEq, Debug)]
pub struct ServerError {
    pub message: String,
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

impl From<sqlx::Error> for ServerError {
    fn from(err: sqlx::Error) -> Self {
        ServerError::new(&format!("Database error: {}", err))
    }
}

// impl From<heed::Error> for ServerError {
//     fn from(err: heed::Error) -> Self {
//         ServerError::new(&format!("Database error: {}", err))
//     }
// }

// impl From<r2d2::Error> for ServerError {
//     fn from(err: r2d2::Error) -> Self {
//         ServerError::new(&format!("Database connection error: {}", err))
//     }
// }

// impl From<rusqlite::Error> for ServerError {
//     fn from(err: rusqlite::Error) -> Self {
//         ServerError::new(&format!("Database error: {}", err))
//     }
// }
