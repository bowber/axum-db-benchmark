use std::env;

#[derive(Clone, Debug)]
pub enum DatabaseType {
    Sqlite,
    Postgres,
    MySql,
    Redis,
    MongoDB,
}

impl DatabaseType {
    pub fn from_env() -> Self {
        match env::var("DATABASE_TYPE").unwrap_or_else(|_| "sqlite".to_string()).to_lowercase().as_str() {
            "postgres" | "postgresql" => DatabaseType::Postgres,
            "mysql" => DatabaseType::MySql,
            "redis" => DatabaseType::Redis,
            "mongo" | "mongodb" => DatabaseType::MongoDB,
            _ => DatabaseType::Sqlite, // Default
        }
    }
}