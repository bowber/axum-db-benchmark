request = function()
    local body = '{"age": 22222}'

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