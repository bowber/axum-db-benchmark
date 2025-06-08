use std::{rc::Rc, sync::Arc};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, patch, post, put},
};
use serde::{Deserialize, Serialize};
use sqlite::Connection;
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
    connection: Arc<Mutex<sqlite::Connection>>,
}

thread_local! {
    static CONNECTION: Rc<Connection> = Rc::new(Connection::open("my_database.db").unwrap());
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
        let conn = sqlite::Connection::open("my_database.db").unwrap();
        // Use a different database for testing
        #[cfg(test)]
        let conn = sqlite::Connection::open(":memory:").unwrap();
        // Set journal mode to WAL (Write-Ahead Logging) for better concurrency
        conn.execute(
            "PRAGMA journal_mode=WAL;
        PRAGMA foreign_keys=ON;",
        )
        .unwrap();
        conn.execute("PRAGMA synchronous=NORMAL;").unwrap();
        // Create the users table if it doesn't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL UNIQUE,
                age INTEGER DEFAULT 0
            );",
        )
        .unwrap();
        // Create index on username for faster lookups
        conn.execute("CREATE INDEX IF NOT EXISTS idx_username ON users (username);")
            .unwrap();
        AppState {
            connection: Arc::new(Mutex::new(conn)),
        }
    }
}

pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUser>,
) -> Result<String, ServerError> {
    // create a new user in the database
    let conn = state.connection.lock().await;
    let mut statement = conn
        .prepare("INSERT INTO users (username) VALUES (?);")
        .unwrap();
    statement.bind((1, payload.username.as_str())).unwrap();
    let result = statement.next();
    if result.is_err() {
        // if there was an error, return a 500 Internal Server Error
        return Err(format!("Error creating user: {:?}", result.err()).into());
    }
    // return a 201 Created status with the user data
    return Ok(format!("User created with username: {}", payload.username));
}

pub async fn get_user_by_username(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<User>, String> {
    // fetch a user by username from the database
    let conn = state.connection.lock().await;
    let mut statement = conn
        .prepare("SELECT id, username, age FROM users WHERE username = ?;")
        .unwrap();
    statement.bind((1, username.as_str())).unwrap();
    match statement.next() {
        Ok(_) => {
            // if a user was found, return it
            let user = User {
                username: statement
                    .read::<String, _>("username")
                    .map_err(|e| format!("Read username error: {}", e))?,
                id: statement
                    .read::<i64, _>("id")
                    .map_err(|e| format!("Read id error: {}", e))? as u64,
                age: statement
                    .read::<i64, _>("age")
                    .map_err(|e| format!("Read age error: {}", e))? as u32,
            };
            Ok(Json(user))
        }
        Err(e) => Err(format!("Get user by username error: {}", e)),
    }
}

pub async fn update_user_by_username(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Json(payload): Json<UpdateUser>,
) -> Result<StatusCode, String> {
    // update a user by username in the database
    let conn = state.connection.lock().await;
    let mut statement = conn
        .prepare("UPDATE users SET age = ? WHERE username = ?;")
        .unwrap();
    statement.bind((1, payload.age as i64)).unwrap();
    statement.bind((2, username.as_str())).unwrap();
    match statement.next() {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => Err(format!("Update user by username error: {}", e)),
    }
}

pub async fn delete_user_by_username(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<StatusCode, String> {
    // delete a user by username from the database
    let conn = state.connection.lock().await;
    let mut statement = conn
        .prepare("DELETE FROM users WHERE username = ?;")
        .unwrap();
    statement.bind((1, username.as_str())).unwrap();
    match statement.next() {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => Err(format!("Delete user by username error: {}", e)),
    }
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
#[derive(Serialize)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub age: u32,
}

mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::StatusCode;
    use axum::response::Response;

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
    async fn test_get_user_by_username() {
        let state = AppState::init().await;
        let payload = CreateUser {
            username: "testuser".to_string(),
        };
        create_user(State(state.clone()), Json(payload)).await;

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
        create_user(State(state.clone()), Json(payload)).await;

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
        assert!(user_response.is_ok());
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
        create_user(State(state.clone()), Json(payload)).await;

        let response =
            delete_user_by_username(State(state.clone()), Path("testuser".to_string())).await;
        assert!(response.is_ok());
        assert_eq!(response.unwrap(), StatusCode::OK);

        // Verify the deletion
        let user_response = get_user_by_username(State(state), Path("testuser".to_string())).await;
        assert!(user_response.is_err());
    }
}
