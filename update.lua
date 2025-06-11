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

---------------------------Without mutex flags (manual mutex)---------------------------
-- ─ ❯❯ wrk -t4 -c100 -d10s -s update.lua http://localhost:3
-- 000
-- Running 10s test @ http://localhost:3000
--   4 threads and 100 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency     6.59ms    5.69ms  43.92ms   80.99%
--     Req/Sec     4.33k   464.79     5.51k    68.00%
--   172188 requests in 10.01s, 12.32MB read
-- Requests/sec:  17209.63
-- Transfer/sec:      1.23MB

-- ╰─ ❯❯ wrk -t4 -c100 -d10s -s update.lua http://localhost:3000
-- Running 10s test @ http://localhost:3000
--   4 threads and 100 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency     6.59ms    5.64ms  41.44ms   80.65%
--     Req/Sec     4.32k   435.43     5.42k    64.75%
--   171950 requests in 10.01s, 12.30MB read
-- Requests/sec:  17184.62
-- Transfer/sec:      1.23MB

-------------------------With mutex flags ---------------------------
-- ─ ❯❯ wrk -t4 -c100 -d10s -s update.lua http://localhost:3
-- 000
-- Running 10s test @ http://localhost:3000
--   4 threads and 100 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency     6.41ms    5.85ms  43.91ms   71.30%
--     Req/Sec     4.61k   490.99     5.72k    63.00%
--   183534 requests in 10.01s, 13.13MB read
-- Requests/sec:  18338.61
-- Transfer/sec:      1.31MB


-- --------------------------On VPS -------------------------------------
-- root@t1no2:~/test# wrk -t4 -c100 -d10s -s update.lua http://localhost:3100
-- Running 10s test @ http://localhost:3100
--   4 threads and 100 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency    77.19ms   80.59ms 394.93ms   84.80%
--     Req/Sec   488.13    258.12     1.66k    61.09%
--   17101 requests in 10.03s, 1.22MB read
-- Requests/sec:   1704.41
-- Transfer/sec:    124.83KB

-- Revert to manual mutex
-- root@t1no2:~/test# wrk -t4 -c100 -d10s -s update.lua http://loc
-- alhost:3100
-- Running 10s test @ http://localhost:3100
--   4 threads and 100 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency    70.95ms   69.62ms 314.94ms   81.43%
--     Req/Sec   475.91    238.84     0.99k    57.53%
--   17616 requests in 10.02s, 1.26MB read
-- Requests/sec:   1758.16
-- Transfer/sec:    128.77KB

----------------------Back to my PC-----------------------------------
-- Using local_thread!
-- ╰─ ❯❯ wrk -t4 -c100 -d10s -s update.lua http://localhost:3000
-- Running 10s test @ http://localhost:3000
--   4 threads and 100 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency     6.52ms    7.13ms  80.34ms   85.20%
--     Req/Sec     4.93k   693.51     6.05k    70.50%
--   196451 requests in 10.01s, 19.59MB read
-- Requests/sec:  19632.18
-- Transfer/sec:      1.96MB

--------------------Using r2d2_sqlite pool-----------------------------
------------- Without random age ------------------
-- ─ ❯❯ wrk -t4 -c100 -d10s -s update.lua http://localhost:3000
-- Running 10s test @ http://localhost:3000
--   4 threads and 100 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency     0.95ms    1.26ms  78.82ms   96.40%
--     Req/Sec    30.05k   681.58    31.80k    71.04%
--   1207786 requests in 10.10s, 86.39MB read
-- Requests/sec: 119582.00
-- Transfer/sec:      8.55MB

-------------- With random age ------------------ (some results in `database is locked` Err)
-- Running 10s test @ http://localhost:3000
--   4 threads and 100 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency    37.75ms   89.51ms   1.77s    98.59%
--     Req/Sec   797.28     75.08     0.99k    70.00%
--   31747 requests in 10.01s, 2.27MB read
--   Socket errors: connect 0, read 0, write 0, timeout 13
--   Non-2xx or 3xx responses: 3
-- Requests/sec:   3172.99
-- Transfer/sec:    232.42KB
------------ With incremental age ------------------
-- Running 10s test @ http://localhost:3000
--   4 threads and 100 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency    43.29ms  113.27ms   1.85s    97.86%
--     Req/Sec   801.28     70.15     1.04k    70.25%
--   31915 requests in 10.01s, 2.28MB read
--   Socket errors: connect 0, read 0, write 0, timeout 14
--   Non-2xx or 3xx responses: 2
-- Requests/sec:   3189.11
-- Transfer/sec:    233.59KB
----------- With the same age ------------------
-- Running 10s test @ http://localhost:3000
--   4 threads and 100 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency     0.96ms    1.16ms  54.79ms   95.87%
--     Req/Sec    29.78k     1.33k   41.86k    88.09%
--   1194174 requests in 10.10s, 85.41MB read
-- Requests/sec: 118238.18
-- Transfer/sec:      8.46MB

--------------------Using r2d2_sqlite single connection-----------------------------
-------------- With random age ------------------ 
-- ╰─ ❯❯ wrk -t4 -c100 -d10s -s update.lua http://localhost:3000
-- Running 10s test @ http://localhost:3000
--   4 threads and 100 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency     2.51ms    1.48ms  16.48ms   72.17%
--     Req/Sec    10.38k   523.02    11.79k    74.75%
--   417121 requests in 10.10s, 29.83MB read
-- Requests/sec:  41299.26
-- Transfer/sec:      2.95MB
--------------- With fixed age ------------------
-- ╰─ ❯❯ wrk -t4 -c100 -d10s -s update.lua http://localhost:3000
-- Running 10s test @ http://localhost:3000
--   4 threads and 100 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency     1.04ms  541.86us   6.12ms   70.02%
--     Req/Sec    24.60k     1.48k   27.56k    76.50%
--   978855 requests in 10.00s, 70.01MB read
-- Requests/sec:  97882.71
-- Transfer/sec:      7.00MB
--------------- With incremental age ------------------
-- ╰─ ❯❯ wrk -t4 -c100 -d10s -s update.lua http://localhost:3000
-- Running 10s test @ http://localhost:3000
--   4 threads and 100 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency     2.37ms    1.43ms  17.20ms   73.56%
--     Req/Sec    11.02k     0.86k   21.61k    96.77%
--   440528 requests in 10.10s, 31.51MB read
-- Requests/sec:  43618.34
-- Transfer/sec:      3.12MB

-------------------------------Using rusqlite connection directly with Arc<Mutex>------------------------
----------- With the same age ------------------
-- Running 10s test @ http://localhost:3000
--   4 threads and 100 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency     1.01ms  122.41us   4.58ms   84.43%
--     Req/Sec    24.95k     1.78k   46.67k    92.29%
--   997774 requests in 10.10s, 71.37MB read
-- Requests/sec:  98795.97
-- Transfer/sec:      7.07MB
------------ With incremental age ------------------
-- ╰─ ❯❯ wrk -t4 -c100 -d10s -s update.lua http://localhost:3000
-- Running 10s test @ http://localhost:3000
--   4 threads and 100 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency     2.02ms  824.03us  12.39ms   89.84%
--     Req/Sec    12.64k   646.46    13.76k    69.75%
--   502811 requests in 10.00s, 35.96MB read
-- Requests/sec:  50273.96
-- Transfer/sec:      3.60MB
-------------- With random age ------------------
-- Running 10s test @ http://localhost:3000
--   4 threads and 100 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency     2.06ms    0.86ms  14.26ms   88.60%
--     Req/Sec    12.38k     1.88k   47.00k    98.00%
--   493877 requests in 10.10s, 35.32MB read
-- Requests/sec:  48899.24
-- Transfer/sec:      3.50MB