## SQLite vs PostgreSQL
### Performance (high-end laptop 16 cores CPU + NVME SSD)
- READ: POSTGRESQL ~ 0.1 x SQLITE ~ 50k ops/s
- CREATE: POSTGRESQL ~ 1.7x SQLITE ~ 37k ops/s
- UPDATE: POSTGRESQL ~ 0.05x SQLITE ~ 2k ops/s
- DELETE: POSTGRESQL ~ 0.76x SQLITE ~ 39k ops/s
### Scalability
- SQLite scales well for reads with the number of CPU cores, but not for writes.
- In theory, PostgreSQL scales well for both reads and writes vertically, but in this benchmark, both reads and writes have very low throughput despite the high-end hardware. It seems that the bottleneck is in the network, serialization, authentication and pooling overhead. But for this benchmark, real-world throughput is more important than theoretical maximums.
### RAM usage
- sqlite should always have low RAM usage
- PostgreSQL is the same but will use some extra for multi-process architecture.
### Setup (both are production-ready)
- PostgreSQL via `tokio-postgres` and `deadpool-postgres` crates.
- Sqlite via r2d2-sqlite crate and tuning these PRAGMA:
```
journal_mode = WAL
synchronous = NORMAL
foreign_keys = ON
```
