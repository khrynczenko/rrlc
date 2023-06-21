# rrlc

`cargo run -- --help`

```
rrlc - rust rate limit checker
Spams HTTP requests on a given URL until time limit is reached or 429 (Too Many Requests) status code is returned.

Usage: rrlc [OPTIONS] <URL> <DURATION> <METHOD>

Arguments:
  <URL>


  <DURATION>
          For how long to spam requests in seconds

  <METHOD>
          [possible values: get, post]

Options:
  -m <MAX_REQUESTS>
          After what number of requests program will stop

          [default: 1000]

  -c <CONCURRENT_REQUESTS_COUNT>
          Size of buffer for requestes that are made at the same time

          [default: 15]

  -q, --quiet
          Disable logging to stdout/stderr

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```
