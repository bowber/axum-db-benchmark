## Conclusion:
- Write performance (INSERT, UPDATE, DELETE) is significantly better with the single-connection approach. (8x)
- Read performance (SELECT, UPDATE without changing value) is significantly better with the multi-connection approach. (5x)

## Max throughput:
### Single-connection (best result when using r2d2_sqlite with a single connection):
- SELECT: 120,943 rows/sec
- INSERT: 22,379 rows/sec
- UPDATE: 41,299 rows/sec (97,882 without changing value)
- DELETE: 51k to 137k rows/sec (the less successful deletion requests, the more throughput you get)

### Multi-connection (best result when using r2d2_sqlite with multiple connections):
- SELECT: 502,142 rows/sec
- INSERT: 3,047 rows/sec
- UPDATE: 3,172 rows/sec (119,582 without changing value)
- DELETE: 9k to 73k rows/sec (the less successful deletion requests, the more throughput you get)

## Pragma settings (good enough for a persistent database & can be used in production):
- journal_mode = WAL
- synchronous = NORMAL
- foreign_keys = ON

## Some little notes::
- Benchmarking using wrk calling Axum http server (for more realistic results).
- r2d2_sqlite results are better in most cases compared to rusqlite or bare sqlite crate.
- rusqlite is better for single-connection use cases (but not much compared to r2d2_sqlite).
- use sqlite crate with manual mutex locking sometimes results in lock error.
- 2 connections dramatically decreases performance for INSERT, UPDATE, DELETE operations -> so don't use it if you just want a balanced point.
- every UPDATE operation only update 1 row, bulk updates are not used in this benchmark.

