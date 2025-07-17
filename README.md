## SQLite vs RocksDB
### Performance (small VPS 4 cores CPU + SSD)
- READ: ROCKSDB ~ 60k ops/s
- CREATE: ROCKSDB  ~ 22k ops/s
- UPDATE: ROCKSDB ~ 23k ops/s
- DELETE: ROCKSDB  ~ 26k ops/s
### Performance (high-end laptop 16 cores CPU + NVME SSD)
- READ: ROCKSDB ~ 1.2 x SQLITE ~ 600k ops/s
- CREATE: ROCKSDB ~ 6x SQLITE ~ 136k ops/s
- UPDATE: ROCKSDB ~ 5x SQLITE ~ 222k ops/s
- DELETE: ROCKSDB ~ 5x SQLITE ~ 250k ops/s
### Scalability
- SQLite scales well for reads with the number of CPU cores, but not for writes.
- RocksDB scales well for both reads and writes.
- Both are difficult to scale across multiple machines because they are embedded databases.
### RAM usage
- sqlite should always have low RAM usage
- RocksDB should use more, but this test only get and update a single record, so the RAM usage is negligible.
### Setup (both are production-ready)
- RocksDB use default setup via rust-rocksdb crate version 0.23.0
- Sqlite via r2d2-sqlite crate and tuning these PRAGMA:
```
journal_mode = WAL
synchronous = NORMAL
foreign_keys = ON
```
