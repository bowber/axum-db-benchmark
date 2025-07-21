local counter = 0

request = function()
    counter = counter + 1
    local username = "user" .. counter
    local path = string.format("/users/%s", username)

    return wrk.format("DELETE", path)
end

-- ╰─ ❯❯ wrk -t4 -c100 -d10s -s delete.lua http://localhost:3000
-- Running 10s test @ http://localhost:3000
--   4 threads and 100 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency    15.76ms    6.05ms  47.18ms   69.27%
--     Req/Sec     1.59k    67.05     1.80k    70.75%
--   63453 requests in 10.01s, 4.54MB read
-- Requests/sec:   6342.09
-- Transfer/sec:    464.51KB

------------------------------ RocksDB Benchmark ------------------------------
-- ╰─ ❯❯ wrk -t4 -c100 -d10s -s delete.lua http://localhost:3000
-- Running 10s test @ http://localhost:3000
--   4 threads and 100 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency   419.69us  404.54us   9.16ms   94.17%
--     Req/Sec    63.38k    11.08k  110.06k    75.00%
--   2523214 requests in 10.01s, 180.47MB read
-- Requests/sec: 252060.77
-- Transfer/sec:     18.03MB

----------------------------- PostgreSQL Benchmark -----------------------------
-- ╰─ ❯❯ wrk -t1 -c100 -d10s -s delete.lua http://localhost:3000
-- Running 10s test @ http://localhost:3000
--   1 threads and 100 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency     2.69ms    1.94ms  29.05ms   93.94%
--     Req/Sec    40.02k     8.86k   46.83k    89.00%
--   398220 requests in 10.01s, 29.51MB read
--   Non-2xx or 3xx responses: 17730
-- Requests/sec:  39801.95
-- Transfer/sec:      2.95MB