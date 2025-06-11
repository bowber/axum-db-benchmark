use std::{rc::Rc, sync::Arc};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, patch, post, put},
};

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
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
    connection: Pool<SqliteConnectionManager>,
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
        let manager = SqliteConnectionManager::file("my_database.db");
        // Use a different database for testing
        #[cfg(test)]
        let manager = SqliteConnectionManager::memory();
        let pool = r2d2::Pool::builder().max_size(1).build(manager).unwrap();
        let conn = pool.get().unwrap();
        conn.pragma_update(None, "journal_mode", "WAL").unwrap();
        conn.pragma_update(None, "synchronous", "NORMAL").unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        // Create the users table if it doesn't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL UNIQUE,
                age INTEGER DEFAULT 0
            );",
            params![],
        )
        .unwrap();
        // Create index on username for faster lookups
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_username ON users (username);",
            params![],
        )
        .unwrap();
        AppState { connection: pool }
    }
}

async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUser>,
) -> Result<String, ServerError> {
    // create a new user in the database
    let conn = state.connection.get()?;
    let result = conn.execute(
        "INSERT INTO users (username) VALUES (?);",
        params![payload.username],
    );
    let changed_row =
        result.map_err(|e| format!("Create user `{}` error: {}", payload.username, e))?;
    if changed_row == 0 {
        // if there was an error, return a 500 Internal Server Error
        return Err(format!("Error creating user: No rows changed").into());
    }
    // return a 201 Created status with the user data
    return Ok(format!("User created with username: {}", payload.username));
}

pub async fn get_user_by_username(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<User>, ServerError> {
    // fetch a user by username from the database
    let conn = state.connection.get()?;
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
        Ok(user) => Ok(Json(user)),
        Err(e) => Err(format!("Get user by username error: {}", e).into()),
    }
}

async fn update_user_by_username(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Json(payload): Json<UpdateUser>,
) -> Result<StatusCode, ServerError> {
    // update a user by username in the database
    let conn = state.connection.get()?;
    let statement = conn.execute(
        "UPDATE users SET age = ? WHERE username = ?;",
        params![payload.age, username],
    );
    // statement.bind((1, payload.age as i64)).unwrap();
    // statement.bind((2, username.as_str())).unwrap();
    match statement {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => Err(format!("Update user by username error: {}", e).into()),
    }
}

pub async fn delete_user_by_username(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<StatusCode, ServerError> {
    // delete a user by username from the database
    let conn = state.connection.get()?;
    let statement = conn.execute("DELETE FROM users WHERE username = ?;", params![username]);

    match statement {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => Err(format!("Delete user by username error: {}", e).into()),
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
