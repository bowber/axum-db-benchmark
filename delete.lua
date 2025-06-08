local counter = 0

request = function()
    counter = counter + 1
    local username = "user" .. counter
    local path = string.format("/users/%s", username)

    return wrk.format("DELETE", path)
end

-- With headers + delete existing user
-- ╰─ ❯❯ wrk -t4 -c100 -d10s -s delete.lua http://localhost:3000
-- Running 10s test @ http://localhost:3000
--   4 threads and 100 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency     6.84ms    6.31ms  54.25ms   86.83%
--     Req/Sec     4.25k   610.99     5.55k    76.25%
--   169389 requests in 10.01s, 12.12MB read
-- Requests/sec:  16929.05
-- Transfer/sec:      1.21MB

-- Without headers + delete unexisting user
-- ╰─ ❯❯ wrk -t4 -c100 -d10s -s delete.lua http://localhost:3000
-- Running 10s test @ http://localhost:3000
--   4 threads and 100 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency     6.30ms    5.45ms  34.78ms   77.37%
--     Req/Sec     4.56k   472.55     5.59k    64.50%
--   181618 requests in 10.01s, 12.99MB read
-- Requests/sec:  18145.06
-- Transfer/sec:      1.30MB

-- Without headers + delete existing user
-- ╰─ ❯❯ wrk -t4 -c100 -d10s -s delete.lua http://localhost:3000
-- Running 10s test @ http://localhost:3000
--   4 threads and 100 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency     8.62ms   11.83ms 120.43ms   92.78%
--     Req/Sec     4.02k     1.13k   18.18k    84.63%
--   159639 requests in 10.10s, 11.42MB read
-- Requests/sec:  15805.11
-- Transfer/sec:      1.13MB

---------------------------- With mutex flags ---------------------------
-- delete unexisting user
-- Running 10s test @ http://localhost:3000
--   4 threads and 100 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency    14.12ms   31.29ms 305.59ms   94.75%
--     Req/Sec     3.77k     1.20k    5.75k    64.95%
--   146238 requests in 10.01s, 10.46MB read
-- Requests/sec:  14613.34
-- Transfer/sec:      1.05MB

-- delete existing user
-- Running 10s test @ http://localhost:3000
--   4 threads and 100 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency    10.99ms   15.55ms 117.53ms   90.35%
--     Req/Sec     3.64k     1.09k    5.23k    63.25%
--   144998 requests in 10.01s, 10.37MB read
-- Requests/sec:  14488.11
-- Transfer/sec:      1.04MB

-- ------------------------ ON VPS -------------------------------------
-- root@t1no2:~/test# wrk -t4 -c100 -d10s -s delete.lua http://localhost:3100
-- Running 10s test @ http://localhost:3100
--   4 threads and 100 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency    78.11ms   76.71ms 314.18ms   78.26%
--     Req/Sec   427.76    229.25     1.11k    65.45%
--   15805 requests in 10.02s, 1.13MB read
-- Requests/sec:   1576.96
-- Transfer/sec:    115.50KB

-- Revert to manual mutex
-- root@t1no2:~/test# wrk -t4 -c100 -d10s -s delete.lua http://localhost:3100
-- Running 10s test @ http://localhost:3100
--   4 threads and 100 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency    78.22ms   75.53ms 278.21ms   77.92%
--     Req/Sec   434.03    236.19     1.08k    59.93%
--   15624 requests in 10.01s, 1.12MB read
-- Requests/sec:   1561.04
-- Transfer/sec:    114.33KB

-- Delete unexisting user
-- root@t1no2:~/test# wrk -t4 -c100 -d10s -s delete.lua http://localhost:3100
-- Running 10s test @ http://localhost:3100
--   4 threads and 100 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency    85.06ms   97.65ms 557.50ms   86.29%
--     Req/Sec   475.39    249.20     1.38k    57.68%
--   16394 requests in 10.01s, 1.17MB read
-- Requests/sec:   1637.06
-- Transfer/sec:    119.90KB