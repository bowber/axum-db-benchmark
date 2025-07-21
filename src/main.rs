use std::{borrow::Cow, error::Error, rc::Rc, sync::Arc};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, patch, post, put},
};

use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use serde::{Deserialize, Serialize};
use std::thread;
use tokio::sync::Mutex;
use tokio_postgres::NoTls;
use tower_http::{
    LatencyUnit,
    trace::{DefaultMakeSpan, DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::{Level, info};
mod err;
use err::ServerError;

#[derive(Clone)]
pub struct AppState {
    pool: Pool,
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
        #[cfg(not(test))]
        let dbname = "mydb";
        #[cfg(test)]
        let random_uuid = uuid::Uuid::new_v4();
        #[cfg(test)]
        let dbname = format!("mydb_{}", random_uuid.to_string().replace("-", ""));
        let mut pg_config = tokio_postgres::Config::new();
        pg_config.host("localhost");
        pg_config.port(5432);
        pg_config.user("myuser");
        pg_config.password("mypassword");
        // Create the database if it doesn't exist (dbname)
        {
            info!("Connecting to PostgreSQL to create database if it doesn't exist");
            let mut tmp_pg_config = pg_config.clone();
            tmp_pg_config.dbname("postgres");
            let tmp_client = tmp_pg_config
                .connect(NoTls)
                .await
                .expect("Failed to connect to PostgreSQL");
            info!("Creating database: {}", dbname);
            tokio::spawn(async move {
                if let Err(e) = tmp_client.1.await {
                    eprintln!("connection error: {}", e);
                }
            });
            // Check if the database already exists
            let query = "SELECT 1 FROM pg_database WHERE datname = $1";
            let exists = tmp_client.0.query_one(query, &[&dbname]).await.is_ok();
            if !exists {
                tmp_client
                    .0
                    .execute(&format!("CREATE DATABASE {};", dbname), &[])
                    .await
                    .expect("Failed to create database");
            }
            info!("Database created or already exists: {}", dbname);
            // Setup connection pool
        }
        info!("Connecting to PostgreSQL database: {}", dbname);
        pg_config.dbname(dbname);
        let mgr_config = ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        };
        let mgr = Manager::from_config(pg_config, NoTls, mgr_config);
        let pool = Pool::builder(mgr).max_size(64).build().unwrap();
        let client = pool.get().await.unwrap();
        // Create the users table if it doesn't exist

        client
            .execute(
                "CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    age INTEGER NOT NULL DEFAULT 0
)",
                &[],
            )
            .await
            .unwrap();
        // Create index on username for faster lookups if it doesn't exist
        client
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_users_username ON users (username)",
                &[],
            )
            .await
            .unwrap();
        AppState { pool }
    }
}

async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUser>,
) -> Result<String, ServerError> {
    // Check if the username is duplicate
    // if state
    //     .db
    //     .get(&payload.username)
    //     .map_err(|e| format!("Database error: {}", e))?
    //     .is_some()
    // {
    //     return Err(format!(
    //         "Create user error: Username '{}' already exists",
    //         payload.username
    //     )
    //     .into());
    // }
    // let user = User {
    //     id: 0, // id will be ignored as we just want to keep the structure of the benchmark
    //     username: payload.username.clone(),
    //     age: 0, // default age
    // };
    // let encoded_user = bitcode::encode(&user);
    // state.db.put(&payload.username, encoded_user)?;

    // // return a 201 Created status with the user data
    // return Ok(format!("User created with username: {}", payload.username));

    // Insert a user into the database
    let client = state
        .pool
        .get()
        .await
        .map_err(|e| format!("Database error: {}", e))?;
    let query = "INSERT INTO users (username, age) VALUES ($1, $2) RETURNING id";
    let is_inserted = client
        .execute(query, &[&payload.username, &0])
        .await
        .map_err(|e| format!("Insert user error: {}", e))
        .is_ok_and(|v| v > 0);
    if !is_inserted {
        return Err(format!("Failed to create user: {}", payload.username).into());
    }
    return Ok(format!("User created with username: {}", payload.username));
}

pub async fn get_user_by_username(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<User>, ServerError> {
    // fetch a user by username from the database
    let client = state
        .pool
        .get()
        .await
        .map_err(|e| format!("Database error: {}", e))?;
    let query = "SELECT id, username, age FROM users WHERE username = $1";
    let row = client
        .query_one(query, &[&username])
        .await
        .map_err(|e| format!("Get user by username error: {}", e))?;
    let user = User {
        id: row.get::<usize, i64>(0) as u64,
        username: row.get(1),
        age: row.get(2),
    };
    Ok(Json(user))
}

async fn update_user_by_username(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Json(payload): Json<UpdateUser>,
) -> Result<StatusCode, ServerError> {
    // update a user by username in the database
    let client = state
        .pool
        .get()
        .await
        .map_err(|e| format!("Database error: {}", e))?;
    let query = "UPDATE users SET age = $1 WHERE username = $2";
    let rows_updated = client
        .execute(query, &[&payload.age, &username])
        .await
        .map_err(|e| format!("Update user by username error: {}", e))?;

    if rows_updated == 0 {
        return Err(format!("User with username '{}' not found", username).into());
    }

    Ok(StatusCode::OK)
}

pub async fn delete_user_by_username(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<StatusCode, ServerError> {
    // delete a user by username from the database
    let client = state
        .pool
        .get()
        .await
        .map_err(|e| format!("Database error: {}", e))?;
    let query = "DELETE FROM users WHERE username = $1";
    let rows_deleted = client
        .execute(query, &[&username])
        .await
        .map_err(|e| format!("Delete user by username error: {}", e))?;

    if rows_deleted == 0 {
        return Err(format!("User with username '{}' not found", username).into());
    }

    Ok(StatusCode::OK)
}

// the input to our `create_user` handler
#[derive(Deserialize, Clone)]
struct CreateUser {
    username: String,
}

#[derive(Deserialize, Clone)]
struct UpdateUser {
    age: i32,
}

// the output to our `create_user` handler
#[derive(Serialize)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub age: i32,
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

        // assert!(response.is_ok());
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
