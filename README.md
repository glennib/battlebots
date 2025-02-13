# battlebots

Binary that can be either a server or a benchmarking client.

The server serves both a gRPC and a HTTP/REST service with the same API.
The client runs benchmarking against these services with either a gRPC client or a HTTP client.

```shell
cargo run -r -- --help
```
