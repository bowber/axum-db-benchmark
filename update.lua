local age = 0
request = function()
    -- age = math.random(10000, 20000) -- Random age between 10,000 and 20,000
    age = age + 1 -- Incremental age for each request
    local body = string.format('{"age": %d}', age)

    local headers = {
        ["Content-Type"] = "application/json",
        ["Content-Length"] = tostring(#body)
    }

    return wrk.format("PATCH", "/users/hello", headers, body)
end

-- ╰─ ❯❯ wrk -t4 -c100 -d10s -s update.lua http://localhost:3000
-- Running 10s test @ http://localhost:3000
--   4 threads and 100 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency    61.37ms   29.44ms 228.51ms   66.23%
--     Req/Sec   411.71     33.40   525.00     75.00%
--   16400 requests in 10.01s, 1.17MB read
-- Requests/sec:   1638.74
-- Transfer/sec:    120.02KB

-- max map size = 1GB | same result as before
-- ╰─ ❯❯ wrk -t4 -c100 -d10s -s update.lua http://localhost:3000
-- Running 10s test @ http://localhost:3000
--   4 threads and 100 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency    61.62ms   29.33ms 222.39ms   66.13%
--     Req/Sec   409.45     30.51   494.00     71.50%
--   16309 requests in 10.01s, 1.17MB read
-- Requests/sec:   1630.08
-- Transfer/sec:    119.39KB

--------------------------------------RocksDB Benchmark-------------------------------
-- ╰─ ❯❯ wrk -t4 -c100 -d10s -s update.lua http://localhost:3000
-- Running 10s test @ http://localhost:3000
--   4 threads and 100 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency   434.47us  233.91us   5.68ms   76.80%
--     Req/Sec    56.12k     6.97k   91.54k    80.65%
--   2250021 requests in 10.10s, 160.93MB read
-- Requests/sec: 222772.26
-- Transfer/sec:     15.93MB
--------------------------------PostgreSQL Benchmark---------------------------------
-- ╰─ ❯❯ wrk -t4 -c100 -d10s -s update.lua http://localhost:3000
-- Running 10s test @ http://localhost:3000
--   4 threads and 100 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency    41.27ms   24.46ms 283.89ms   81.01%
--     Req/Sec   635.64     46.62   757.00     70.50%
--   25320 requests in 10.01s, 1.81MB read
-- Requests/sec:   2530.48
-- Transfer/sec:    185.34KB