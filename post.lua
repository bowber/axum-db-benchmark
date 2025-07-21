local counter = 0

request = function()
    counter = counter + 1
    local username = "user" .. counter
    local body = string.format('{"username": "%s"}', username)

    local headers = {
        ["Content-Type"] = "application/json",
        ["Content-Length"] = tostring(#body)
    }

    return wrk.format("POST", "/users", headers, body)
end

-- ╰─ ❯❯ wrk -t1 -c400 -d10s -s post.lua http://localhost:3000
-- Running 10s test @ http://localhost:3000
--   1 threads and 400 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency   242.99ms  115.35ms 948.32ms   68.53%
--     Req/Sec     1.65k    27.60     1.69k    76.00%
--   16378 requests in 10.03s, 2.39MB read
-- Requests/sec:   1632.21
-- Transfer/sec:    244.39KB

-- -- max map size = 1GB | same result as before
-- ╰─ ❯❯ wrk -t1 -c400 -d10s -s post.lua http://localhost:3000
-- Running 10s test @ http://localhost:3000
--   1 threads and 400 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency   252.13ms  144.50ms   1.36s    80.08%
--     Req/Sec     1.62k    36.51     1.69k    82.00%
--   16159 requests in 10.02s, 2.36MB read
-- Requests/sec:   1612.90
-- Transfer/sec:    241.48KB

-- -------------------------------RocksDB Benchmark-------------------------------
-- ╰─ ❯❯ wrk -t1 -c400 -d10s -s post.lua http://localhost:3000
-- Running 10s test @ http://localhost:3000
--   1 threads and 400 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency     1.47ms  421.93us   5.96ms   71.16%
--     Req/Sec   137.36k     7.59k  150.49k    70.00%
--   1371148 requests in 10.05s, 202.93MB read
-- Requests/sec: 136387.47
-- Transfer/sec:     20.19MB

---------------------------------PostgreSQL Benchmark---------------------------------
-- ╰─ ❯❯ wrk -t1 -c400 -d10s -s post.lua http://localhost:3000
-- Running 10s test @ http://localhost:3000
--   1 threads and 400 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency    17.23ms    2.06ms  40.95ms   74.09%
--     Req/Sec    23.29k     1.89k   25.85k    73.00%
--   231756 requests in 10.03s, 34.15MB read
-- Requests/sec:  23112.42
-- Transfer/sec:      3.41MB

-- Increase the max size of the connection pool to 64 and add an index on username for faster lookups
-- ╰─ ❯❯ wrk -t1 -c400 -d10s -s post.lua http://localhost:3000
-- Running 10s test @ http://localhost:3000
--   1 threads and 400 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency    10.48ms    1.82ms  33.21ms   92.49%
--     Req/Sec    38.24k     2.57k   43.29k    77.00%
--   380317 requests in 10.02s, 56.11MB read
-- Requests/sec:  37937.69
-- Transfer/sec:      5.60MB