use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, patch, post},
};
use tower_http::{
    LatencyUnit,
    trace::{DefaultMakeSpan, DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::Level;

mod config;
mod database;
mod databases;
mod err;

use config::DatabaseType;
use database::{CreateUser, Database, UpdateUser, User};
use databases::*;
use err::ServerError;

#[derive(Clone)]
pub struct AppState<T: Database> {
    db: T,
}

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();
    
    let db_type = DatabaseType::from_env();
    println!("Using database type: {:?}", db_type);
    
    // Build our application with a route - need to match on db type
    match db_type {
        DatabaseType::Sqlite => {
            let state = AppState { db: SqliteDatabase::init().await.expect("Failed to initialize SQLite") };
            run_server(state).await;
        },
        DatabaseType::Postgres => {
            let state = AppState { db: PostgresDatabase::init().await.expect("Failed to initialize PostgreSQL") };
            run_server(state).await;
        },
        DatabaseType::MySql => {
            let state = AppState { db: MySqlDatabase::init().await.expect("Failed to initialize MySQL") };
            run_server(state).await;
        },
        DatabaseType::Redis => {
            let state = AppState { db: RedisDatabase::init().await.expect("Failed to initialize Redis") };
            run_server(state).await;
        },
        DatabaseType::MongoDB => {
            let state = AppState { db: MongoDatabase::init().await.expect("Failed to initialize MongoDB") };
            run_server(state).await;
        },
    }
}

async fn run_server<T: Database + 'static>(state: AppState<T>) {
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root::<T>))
        // `GET /users/{username}` goes to `get_user_by_username`
        .route("/users/{username}", get(get_user_by_username::<T>))
        // `POST /users` goes to `create_user`
        .route("/users", post(create_user::<T>))
        // `PATCH /users/{username}` goes to `update_user_by_username`
        .route("/users/{username}", patch(update_user_by_username::<T>))
        // `DELETE /users/{username}` goes to `delete_user_by_username`
        .route("/users/{username}", delete(delete_user_by_username::<T>))
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
async fn root<T: Database>(State(_): State<AppState<T>>) -> &'static str {
    "Hello, World!"
}
async fn create_user<T: Database>(
    State(state): State<AppState<T>>,
    Json(payload): Json<CreateUser>,
) -> Result<String, ServerError> {
    state.db.create_user(payload).await.map_err(|e| ServerError::new(&e.to_string()))
}

pub async fn get_user_by_username<T: Database>(
    State(state): State<AppState<T>>,
    Path(username): Path<String>,
) -> Result<Json<User>, ServerError> {
    let user = state.db.get_user(username).await.map_err(|e| ServerError::new(&e.to_string()))?;
    Ok(Json(user))
}

async fn update_user_by_username<T: Database>(
    State(state): State<AppState<T>>,
    Path(username): Path<String>,
    Json(payload): Json<UpdateUser>,
) -> Result<StatusCode, ServerError> {
    state.db.update_user(username, payload).await.map_err(|e| ServerError::new(&e.to_string()))?;
    Ok(StatusCode::OK)
}

pub async fn delete_user_by_username<T: Database>(
    State(state): State<AppState<T>>,
    Path(username): Path<String>,
) -> Result<StatusCode, ServerError> {
    state.db.delete_user(username).await.map_err(|e| ServerError::new(&e.to_string()))?;
    Ok(StatusCode::OK)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn create_test_state() -> AppState<SqliteDatabase> {
        AppState { db: SqliteDatabase::init().await.unwrap() }
    }

    #[tokio::test]
    async fn test_root() {
        let state = create_test_state().await;
        let response = root(State(state)).await;
        assert_eq!(response, "Hello, World!");
    }
    
    #[tokio::test]
    async fn test_create_user() {
        let payload = CreateUser {
            username: "testuser".to_string(),
        };
        let state = create_test_state().await;
        let status = create_user(State(state), Json(payload)).await;
        assert_eq!(
            status,
            Ok("User created with username: testuser".to_string())
        );
    }
    
    #[tokio::test]
    async fn test_get_user_by_username() {
        let state = create_test_state().await;
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
        let state = create_test_state().await;
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
        let state = create_test_state().await;
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
