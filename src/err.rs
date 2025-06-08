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
        let body = Body::from(self.message);
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}
