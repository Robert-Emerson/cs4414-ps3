zhtta performance

httperf --client=0/1 --server=localhost --port=4414 --uri=/ --send-buffer=4096 --recv-buffer=16384 --num-conns=10000 --num-calls=1
Maximum connect burst length: 1

Total: connections 10000 requests 10000 replies 10000 test-duration 1.683 s

Connection rate: 5940.1 conn/s (0.2 ms/conn, <=1 concurrent connections)
Connection time [ms]: min 0.1 avg 0.2 max 5.1 median 0.5 stddev 0.1
Connection time [ms]: connect 0.0
Connection length [replies/conn]: 1.000

Request rate: 5940.1 req/s (0.2 ms/req)
Request size [B]: 62.0

Reply rate [replies/s]: min 0.0 avg 0.0 max 0.0 stddev 0.0 (0 samples)
Reply time [ms]: response 0.1 transfer 0.0
Reply size [B]: header 59.0 content 451.0 footer 0.0 (total 510.0)
Reply status: 1xx=0 2xx=10000 3xx=0 4xx=0 5xx=0

CPU time [s]: user 0.15 system 1.53 (user 9.0% system 90.8% total 99.9%)
Net I/O: 3323.3 KB/s (27.2*10^6 bps)

Errors: total 0 client-timo 0 socket-timo 0 connrefused 0 connreset 0
Errors: fd-unavail 0 addrunavail 0 ftab-full 0 other 0



zhttpto performance

httperf --client=0/1 --server=localhost --port=4414 --uri=/ --send-buffer=4096 --recv-buffer=16384 --num-conns=10000 --num-calls=1
Maximum connect burst length: 1

Total: connections 10000 requests 10000 replies 10000 test-duration 3.363 s

Connection rate: 2973.8 conn/s (0.3 ms/conn, <=1 concurrent connections)
Connection time [ms]: min 0.2 avg 0.3 max 7.3 median 0.5 stddev 0.2
Connection time [ms]: connect 0.0
Connection length [replies/conn]: 1.000

Request rate: 2973.8 req/s (0.3 ms/req)
Request size [B]: 62.0

Reply rate [replies/s]: min 0.0 avg 0.0 max 0.0 stddev 0.0 (0 samples)
Reply time [ms]: response 0.2 transfer 0.0
Reply size [B]: header 59.0 content 397.0 footer 0.0 (total 456.0)
Reply status: 1xx=0 2xx=10000 3xx=0 4xx=0 5xx=0

CPU time [s]: user 0.34 system 3.02 (user 10.2% system 89.7% total 99.9%)
Net I/O: 1506.9 KB/s (12.3*10^6 bps)

Errors: total 0 client-timo 0 socket-timo 0 connrefused 0 connreset 0
Errors: fd-unavail 0 addrunavail 0 ftab-full 0 other 0
