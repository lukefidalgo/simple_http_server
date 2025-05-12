# Simple Rust HTTP File Server

A basic HTTP/1.1 file server written in Rust. Supports:
- Serving static files from a directory
- Gzip compression (if requested via `Accept-Encoding`)
- Uploading files via `POST`
- Echo and User-Agent endpoints

## Usage

1. Clone the repository:

```bash
git clone https://github.com/lukefidalgo/simple_http_server.git
cd simple_http_server
```
2. Build and run:

```bash
cargo run -- --directory /path/to/serve
```

## Special Thanks
This project was written following the HTTP Server challange at [codecrafters.io](https://codecrafters.io/). It's the only programming learning platform I know of which is worth subscribing to.
