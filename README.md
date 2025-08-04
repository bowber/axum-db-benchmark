# Axum Database Benchmark

A comprehensive database performance benchmark using Axum web framework with support for multiple database backends.

## Supported Databases

This benchmark now supports **5 popular databases**:

1. **SQLite** (default) - File-based SQL database with WAL mode
2. **PostgreSQL** - Popular relational database
3. **MySQL** - Popular relational database  
4. **Redis** - In-memory key-value store
5. **MongoDB** - Document database

## Quick Start

### Default SQLite Setup
```bash
# Uses SQLite by default (no setup required)
cargo run
```

### Using Different Databases

Set the `DATABASE_TYPE` environment variable to use different databases:

```bash
# PostgreSQL
export DATABASE_TYPE=postgres
export POSTGRES_URL="postgresql://postgres:password@localhost:5432/benchmark"
cargo run

# MySQL
export DATABASE_TYPE=mysql
export MYSQL_URL="mysql://root:password@localhost:3306/benchmark"
cargo run

# Redis
export DATABASE_TYPE=redis
export REDIS_URL="redis://localhost:6379"
cargo run

# MongoDB
export DATABASE_TYPE=mongodb
export MONGO_URL="mongodb://localhost:27017"
cargo run
```

## Database Setup

### SQLite (Default)
No setup required. Creates `my_database.db` file automatically.

### PostgreSQL
```bash
# Using Docker
docker run --name postgres-bench -e POSTGRES_PASSWORD=password -e POSTGRES_DB=benchmark -p 5432:5432 -d postgres:13

# Or using local PostgreSQL
createdb benchmark
```

### MySQL
```bash
# Using Docker
docker run --name mysql-bench -e MYSQL_ROOT_PASSWORD=password -e MYSQL_DATABASE=benchmark -p 3306:3306 -d mysql:8

# Or using local MySQL
mysql -u root -p -e "CREATE DATABASE benchmark;"
```

### Redis
```bash
# Using Docker
docker run --name redis-bench -p 6379:6379 -d redis:7

# Or using local Redis
redis-server
```

### MongoDB
```bash
# Using Docker
docker run --name mongo-bench -p 27017:27017 -d mongo:6

# Or using local MongoDB
mongod
```

## API Endpoints

All databases expose the same REST API:

- `GET /` - Health check
- `POST /users` - Create user: `{"username": "john"}`
- `GET /users/{username}` - Get user by username
- `PATCH /users/{username}` - Update user: `{"age": 25}`
- `DELETE /users/{username}` - Delete user by username

## Benchmarking

Use the included wrk scripts for benchmarking:

```bash
# Start the server
cargo run

# In another terminal, run benchmarks:

# GET requests
wrk -t4 -c100 -d10s http://localhost:3000/users/testuser

# POST requests  
wrk -t4 -c100 -d10s -s post.lua http://localhost:3000

# PATCH requests
wrk -t4 -c100 -d10s -s update.lua http://localhost:3000  

# DELETE requests
wrk -t4 -c100 -d10s -s delete.lua http://localhost:3000
```

## Performance Notes

### SQLite Configuration
- **Journal Mode**: WAL2 for better concurrent performance
- **Synchronous**: NORMAL for balanced safety/performance
- **Foreign Keys**: Enabled
- **Connection Pool**: r2d2 with configurable pool size

### Database-Specific Optimizations
- **PostgreSQL/MySQL**: Uses connection pooling with r2d2
- **Redis**: JSON serialization for complex data structures  
- **MongoDB**: Indexes on username field for fast lookups
- **All**: Async operations for non-blocking I/O

## Previous Benchmark Results (SQLite)

### Max Throughput (Single vs Multi-connection):

**Single-connection (r2d2_sqlite with 1 connection):**
- SELECT: 120,943 rows/sec
- INSERT: 22,379 rows/sec  
- UPDATE: 41,299 rows/sec (97,882 without value changes)
- DELETE: 51k to 137k rows/sec

**Multi-connection (r2d2_sqlite with pool):**
- SELECT: 502,142 rows/sec
- Similar INSERT/UPDATE/DELETE performance after configuration fixes

## Testing

```bash
# Run unit tests (uses in-memory SQLite)
cargo test

# Run with specific database for integration testing
DATABASE_TYPE=postgres cargo test
```

## Architecture

The application uses a trait-based database abstraction:

```rust
#[async_trait]
pub trait Database: Send + Sync + Clone {
    type Error: std::error::Error + Send + Sync + 'static;
    
    async fn init() -> Result<Self, Self::Error>;
    async fn create_user(&self, user: CreateUser) -> Result<String, Self::Error>;
    async fn get_user(&self, username: String) -> Result<User, Self::Error>;
    async fn update_user(&self, username: String, update: UpdateUser) -> Result<(), Self::Error>;
    async fn delete_user(&self, username: String) -> Result<(), Self::Error>;
}
```

This allows switching between databases at runtime while maintaining the same API and benchmark scripts.

