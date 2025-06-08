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

-- Running 10s test @ http://localhost:3000
--   4 threads and 100 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency    23.57ms   71.57ms 580.30ms   94.77%
--     Req/Sec     3.61k     1.03k    5.62k    66.67%
--   135629 requests in 10.01s, 23.21MB read
-- Requests/sec:  13554.23
-- Transfer/sec:      2.32MB

-- ╰─ ❯❯ wrk -t4 -c100 -d10s -s post.lua http://localhost:300
-- 0
-- Running 10s test @ http://localhost:3000
--   4 threads and 100 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency     6.40ms    5.13ms  62.13ms   88.51%
--     Req/Sec     4.31k   514.92     5.48k    70.25%
--   171631 requests in 10.00s, 29.37MB read
-- Requests/sec:  17158.05
-- Transfer/sec:      2.94MB

-- ╰─ ❯❯ wrk -t4 -c100 -d10s -s post.lua http://localhost:3000
-- Running 10s test @ http://localhost:3000
--   4 threads and 100 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency    14.00ms   28.40ms 272.89ms   94.98%
--     Req/Sec     3.51k     1.18k    5.47k    67.53%
--   136315 requests in 10.01s, 23.32MB read
-- Requests/sec:  13619.99
-- Transfer/sec:      2.33MB


-- ------------------Above is single connection test using sqlite + Mutex------------------
-- ------------------Below is single connection test using sqlite + serialize multithreading mode------------------
-- Running 10s test @ http://localhost:3000
--   4 threads and 100 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency     6.55ms    6.01ms  50.44ms   76.36%
--     Req/Sec     4.49k   522.02     5.84k    75.25%
--   178789 requests in 10.01s, 31.69MB read
-- Requests/sec:  17865.22
-- Transfer/sec:      3.17MB

-- ╰─ ❯❯ wrk -t4 -c100 -d10s -s post.lua http://localhost:3000
-- Running 10s test @ http://localhost:3000
--   4 threads and 100 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency     6.50ms    5.53ms  49.51ms   77.54%
--     Req/Sec     4.34k   442.25     5.33k    66.75%
--   172927 requests in 10.01s, 29.51MB read
-- Requests/sec:  17280.14
-- Transfer/sec:      2.95MB

-- ---------------------------On VPS-------------------------------------
-- root@t1no2:~/test# wrk -t4 -c100 -d10s -s post.lua http://localhos
-- t:3100
-- Running 10s test @ http://localhost:3100
--   4 threads and 100 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency    81.67ms   79.10ms 315.11ms   78.73%
--     Req/Sec   396.07    194.17     1.16k    72.08%
--   15200 requests in 10.01s, 2.59MB read
-- Requests/sec:   1517.80
-- Transfer/sec:    264.92KB

-- Revert to manual mutex
-- root@t1no2:~/test# wrk -t4 -c100 -d10s -s post.lua http://localhost:3100
-- Running 10s test @ http://localhost:3100
--   4 threads and 100 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency    70.18ms   65.96ms 265.68ms   79.27%
--     Req/Sec   455.04    237.50     0.98k    56.86%
--   16938 requests in 10.02s, 2.89MB read
-- Requests/sec:   1690.81
-- Transfer/sec:    295.84KB