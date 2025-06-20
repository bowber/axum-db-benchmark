use std::{borrow::Cow, error::Error, rc::Rc, sync::Arc};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, patch, post, put},
};

use bitcode::{Decode, Encode};
use rocksdb::{DB, DBCommon, DBWithThreadMode, MultiThreaded, SingleThreaded};
use serde::{Deserialize, Serialize};
use std::thread;
use tokio::sync::Mutex;
use tower_http::{
    LatencyUnit,
    trace::{DefaultMakeSpan, DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::Level;
mod err;
use err::ServerError;

#[derive(Clone)]
pub struct AppState {
    db: Arc<DBWithThreadMode<MultiThreaded>>,
}

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();
    let state = AppState::init().await;
    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        // `GET /users/{username}` goes to `get_user_by_username`
        .route("/users/{username}", get(get_user_by_username))
        // `POST /users` goes to `create_user`
        .route("/users", post(create_user))
        // `PUT /users/{username}` goes to `update_user_by_username`
        .route("/users/{username}", patch(update_user_by_username))
        // `DELETE /users/{username}` goes to `delete_user_by_username`
        .route("/users/{username}", delete(delete_user_by_username))
        .with_state(state)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().include_headers(true))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(LatencyUnit::Micros),
                )
                .on_failure(
                    DefaultOnFailure::new()
                        .level(Level::ERROR)
                        .latency_unit(LatencyUnit::Micros),
                ),
        );

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
async fn root(State(_): State<AppState>) -> &'static str {
    "Hello, World!"
}
impl AppState {
    pub async fn init() -> Self {
        // Initialize the SQLite database connection
        #[cfg(not(test))]
        let dir = "./data";
        #[cfg(test)]
        let tmp = tempfile::tempdir().unwrap();
        #[cfg(test)]
        let dir = tmp.path();
        let db = Arc::new(DBWithThreadMode::<MultiThreaded>::open_default(dir).unwrap());

        AppState { db }
    }
}

async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUser>,
) -> Result<String, ServerError> {
    // Check if the username is duplicate
    if state
        .db
        .get(&payload.username)
        .map_err(|e| format!("Database error: {}", e))?
        .is_some()
    {
        return Err(format!(
            "Create user error: Username '{}' already exists",
            payload.username
        )
        .into());
    }
    let user = User {
        id: 0, // id will be ignored as we just want to keep the structure of the benchmark
        username: payload.username.clone(),
        age: 0, // default age
    };
    let encoded_user = bitcode::encode(&user);
    state.db.put(&payload.username, encoded_user)?;

    // return a 201 Created status with the user data
    return Ok(format!("User created with username: {}", payload.username));
}

pub async fn get_user_by_username(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<User>, ServerError> {
    // fetch a user by username from the database
    let result = state
        .db
        .get(&username)
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or(format!("Get user by username error: User not found"))?;
    let decoded_user: User =
        bitcode::decode(&result).map_err(|e| format!("Failed to decode User: {}", e))?;
    Ok(Json(decoded_user))
}

async fn update_user_by_username(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Json(payload): Json<UpdateUser>,
) -> Result<StatusCode, ServerError> {
    // update a user by username in the database
    let result = state
        .db
        .get(&username)
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or(format!("Update user by username error: User not found"))?;
    let decoded_user: User =
        bitcode::decode(&result).map_err(|e| format!("Failed to decode User: {}", e))?;
    // Update the user's age
    let encoded_user = bitcode::encode(&User {
        id: decoded_user.id,                     // keep the same id
        username: decoded_user.username.clone(), // keep the same username
        age: payload.age,                        // update the age
    });
    state.db.put(&username, &encoded_user)?;
    Ok(StatusCode::OK)
}

pub async fn delete_user_by_username(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<StatusCode, ServerError> {
    // delete a user by username from the database
    state.db.delete(&username)?;
    Ok(StatusCode::OK)
}

// the input to our `create_user` handler
#[derive(Deserialize, Clone)]
struct CreateUser {
    username: String,
}

#[derive(Deserialize, Clone)]
struct UpdateUser {
    age: u32,
}

// the output to our `create_user` handler
#[derive(Encode, Decode, Serialize)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub age: u32,
}

// impl<'a> BytesDecode<'a> for User {
//     type DItem = Self;
//     fn bytes_decode(bytes: &[u8]) -> Result<Self, Box<dyn Error + Send + Sync + 'static>> {
//         match bitcode::decode(&bytes) {
//             Ok(user) => Ok(user),
//             Err(e) => Err(format!("Failed to decode User: {}", e).into()),
//         }
//     }
// }

// impl<'a> BytesEncode<'a> for User {
//     type EItem = Self;
//     fn bytes_encode(item: &'a Self::EItem) -> Result<Cow<'a, [u8]>, BoxedError> {
//         let v = bitcode::encode(item);
//         Ok(Cow::Owned(v))
//     }
// }

mod tests {
    #[cfg(test)]
    use super::*;

    #[tokio::test]
    async fn test_root() {
        let state = AppState::init().await;
        let response = root(State(state)).await;
        assert_eq!(response, "Hello, World!");
    }
    #[tokio::test]
    async fn test_create_user() {
        let payload = CreateUser {
            username: "testuser".to_string(),
        };
        let state = AppState::init().await;
        let status = create_user(State(state), Json(payload)).await;
        assert_eq!(
            status,
            Ok("User created with username: testuser".to_string())
        );
    }
    #[tokio::test]
    async fn test_create_user_upplicated() {
        let payload = CreateUser {
            username: "this is the dup key".to_string(), // Invalid username
        };
        let state = AppState::init().await;
        create_user(State(state.clone()), Json(payload.clone()))
            .await
            .unwrap();
        let status = create_user(State(state), Json(payload)).await;
        // Expect an error due to invalid username

        assert!(status.is_err());
    }
    #[tokio::test]
    async fn test_get_user_by_username() {
        let state = AppState::init().await;
        let payload = CreateUser {
            username: "testuser".to_string(),
        };
        create_user(State(state.clone()), Json(payload))
            .await
            .unwrap();

        let response = get_user_by_username(State(state), Path("testuser".to_string())).await;
        assert!(response.is_ok());
        let user = response.unwrap().0;
        assert_eq!(user.username, "testuser");
    }

    #[tokio::test]
    async fn test_update_user_by_username() {
        let state = AppState::init().await;
        let payload = CreateUser {
            username: "testuser".to_string(),
        };
        create_user(State(state.clone()), Json(payload))
            .await
            .unwrap();

        let update_payload = UpdateUser { age: 30 };
        let response = update_user_by_username(
            State(state.clone()),
            Path("testuser".to_string()),
            Json(update_payload),
        )
        .await;

        assert!(response.is_ok());
        assert_eq!(response.unwrap(), StatusCode::OK);

        // Verify the update
        let user_response = get_user_by_username(State(state), Path("testuser".to_string())).await;
        let user = user_response.unwrap().0;
        assert_eq!(user.username, "testuser");
        assert_eq!(user.age, 30);
    }
    #[tokio::test]
    async fn test_delete_user_by_username() {
        let state = AppState::init().await;
        let payload = CreateUser {
            username: "testuser".to_string(),
        };
        create_user(State(state.clone()), Json(payload))
            .await
            .unwrap();

        let response =
            delete_user_by_username(State(state.clone()), Path("testuser".to_string())).await;
        assert!(response.is_ok());
        assert_eq!(response.unwrap(), StatusCode::OK);

        // Verify the deletion
        let user_response = get_user_by_username(State(state), Path("testuser".to_string())).await;
        assert!(user_response.is_err());
    }
}
