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

----------------------Back to my PC-----------------------------------
-- Using local_thread! NOTE: THIS ONLY 4k records put into database
-- ╰─ ❯❯ wrk -t4 -c100 -d10s -s post.lua http://localhost:3000
-- Running 10s test @ http://localhost:3000
--   4 threads and 100 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency     6.11ms    5.91ms  47.17ms   73.51%
--     Req/Sec     5.04k   491.68     6.08k    63.25%
--   200934 requests in 10.01s, 31.96MB read
-- Requests/sec:  20076.37
-- Transfer/sec:      3.19MB

-- ╰─ ❯❯ wrk -t4 -c100 -d10s -s post.lua http://localhost:3000
-- Running 10s test @ http://localhost:3000
--   4 threads and 100 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency     6.34ms    6.40ms  53.48ms   79.20%
--     Req/Sec     4.97k   611.94     5.90k    67.00%
--   197803 requests in 10.01s, 31.08MB read
-- Requests/sec:  19768.99
-- Transfer/sec:      3.11MB

------------------------------ +50% throughput after using result instead of panic------------------
-- ~29k
-------------------------------Using r2d2_sqlite pool------------------------
---------- Insert existed users
-- Running 10s test @ http://localhost:3000
--   4 threads and 100 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency     3.68ms    3.88ms 133.50ms   91.54%
--     Req/Sec     8.15k     1.39k   10.29k    67.25%
--   324492 requests in 10.00s, 46.73MB read
--   Non-2xx or 3xx responses: 324157
-- Requests/sec:  32434.56
-- Transfer/sec:      4.67MB
---------- Insert unexisting users
-- Running 10s test @ http://localhost:3000
--   4 threads and 100 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency    29.18ms  105.47ms   1.84s    96.18%
--     Req/Sec     2.26k   199.59     3.17k    69.75%
--   90106 requests in 10.00s, 13.03MB read
--   Socket errors: connect 0, read 0, write 0, timeout 2
--   Non-2xx or 3xx responses: 67380
-- Requests/sec:   9007.38
-- Transfer/sec:      1.30MB

------------- Using 1 thread wrk ---------------
-- ╰─ ❯❯ wrk -t1 -c400 -d10s -s post.lua http://localhost:3000
-- Running 10s test @ http://localhost:3000
--   1 threads and 400 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency   187.27ms  293.63ms   1.97s    90.79%
--     Req/Sec     3.07k   763.87     9.61k    95.00%
--   30539 requests in 10.02s, 4.47MB read
--   Socket errors: connect 0, read 0, write 0, timeout 93
--   Non-2xx or 3xx responses: 4
-- Requests/sec:   3047.93
-- Transfer/sec:    457.29KB
-------------------------------Using r2d2_sqlite pool with only 1 connection------------------------
-- Running 10s test @ http://localhost:3000
--   1 threads and 400 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency    18.29ms    9.07ms  91.20ms   80.90%
--     Req/Sec    22.55k     1.06k   24.71k    81.00%
--   224466 requests in 10.03s, 33.07MB read
-- Requests/sec:  22379.48
-- Transfer/sec:      3.30MB
-------------------------------Using rusqlite connection directly with Arc<Mutex>------------------------
-- ╰─ ❯❯ wrk -t1 -c400 -d10s -s post.lua http://localhost:3000
-- Running 10s test @ http://localhost:3000
--   1 threads and 400 connections
--   Thread Stats   Avg      Stdev     Max   +/- Stdev
--     Latency    17.23ms    1.74ms  28.69ms   72.62%
--     Req/Sec    23.31k     1.42k   25.23k    80.00%
--   231958 requests in 10.04s, 34.18MB read
-- Requests/sec:  23113.09
-- Transfer/sec:      3.41MB