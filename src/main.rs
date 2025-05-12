use std::{
    collections::HashMap,
    fs,
    io::{BufReader, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
};

use clap::Parser;
use flate2::{write::GzEncoder, Compression};

mod http_request;
use http_request::HttpRequest;

#[derive(Parser)]
#[command(version, about = "A simple file server")]
struct Args {
    #[arg(short, long, default_value = ".")]
    directory: PathBuf,
}

fn supports_gzip(headers: &HashMap<String, String>) -> bool {
    headers
        .get("Accept-Encoding")
        .map(|encodings| encodings.contains("gzip"))
        .unwrap_or(false)
}

fn build_response(
    headers: &HashMap<String, String>,
    status: &str,
    body: Option<&[u8]>,
    content_type: Option<&str>,
    should_close: bool,
) -> Vec<u8> {
    let mut response = Vec::new();
    let use_gzip = supports_gzip(headers);

    response.extend_from_slice(format!("HTTP/1.1 {}\r\n", status).as_bytes());

    if should_close {
        response.extend_from_slice(b"Connection: close\r\n");
    }

    let (compressed_body, content_encoding) = if let Some(body) = body {
        if use_gzip {
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(body).unwrap();
            (encoder.finish().unwrap(), Some("gzip"))
        } else {
            (body.to_vec(), None)
        }
    } else {
        (Vec::new(), None)
    };

    if content_encoding.is_some() {
        response.extend_from_slice(b"Content-Encoding: gzip\r\n");
    }

    if let Some(ct) = content_type {
        response.extend_from_slice(format!("Content-Type: {}\r\n", ct).as_bytes());
    }

    if !compressed_body.is_empty() {
        response.extend_from_slice(format!("Content-Length: {}\r\n", compressed_body.len()).as_bytes());
    }

    response.extend_from_slice(b"\r\n");

    if !compressed_body.is_empty() {
        response.extend_from_slice(&compressed_body);
    }

    response
}

fn handle_connection(mut stream: TcpStream, directory: &PathBuf) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut should_close = false;

    while !should_close {
        let request = match HttpRequest::from_stream(&mut reader) {
            Ok(req) => req,
            Err(e) => {
                eprintln!("Failed to parse request: {}", e);
                break;
            }
        };

        should_close = request.headers
            .get("Connection")
            .map(|c| c.eq_ignore_ascii_case("close"))
            .unwrap_or(false);

        let response = match (request.method.as_str(), request.path.as_str()) {
                ("GET", "/") => build_response(&request.headers, "200 OK", None, None, should_close),
                ("GET", path) if path.starts_with("/echo/") => {
                    let content = path.trim_start_matches("/echo/");
                    build_response(
                        &request.headers,
                        "200 OK",
                        Some(content.as_bytes()),
                        Some("text/plain"),
                        should_close
                    )
                }
                ("GET", path) if path.starts_with("/user-agent") => {
                    let content = request.headers
                        .get("User-Agent")
                        .map(|s| s.as_str())
                        .unwrap_or("Unknown");
                    build_response(
                        &request.headers,
                        "200 OK",
                        Some(content.as_bytes()),
                        Some("text/plain"),
                        should_close
                    )
                }
                ("GET", path) if path.starts_with("/files") => {
                    let filename = path.trim_start_matches("/files/");
                    let filepath = directory.join(filename);

                    match fs::read(&filepath) {
                        Ok(content) => build_response(
                            &request.headers,
                            "200 OK",
                            Some(&content),
                            Some("application/octet-stream"),
                            should_close
                        ),
                        Err(_) => build_response(&request.headers, "404 Not Found", None, None, should_close),
                    }
                }
                ("POST", path) if path.starts_with("/files/") => {
                    let filename = path.trim_start_matches("/files/");
                    let filepath = directory.join(filename);

                    if let Some(body) = request.body {
                        match fs::write(&filepath, body) {
                            Ok(_) => build_response(&request.headers, "201 Created", None, None, should_close),

                            Err(e) => {
                                eprintln!("Failed to write file: {}", e);
                                build_response(&request.headers, "500 Internal Server Error", None, None, should_close)
                            }
                        }
                    } else {
                        build_response(&request.headers, "400 Bad Request", None, None, should_close)
                    }
                }
                (_, _) => build_response(&request.headers, "404 Not Found", None, None, should_close),
            };

            if let Err(e) = stream.write_all(&response) {
                eprintln!("Failed to write response: {}", e);
                break;
            }

            stream.flush().unwrap();
    }
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let listener = TcpListener::bind("127.0.0.1:4221")?;
    
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let directory = args.directory.clone();
                std::thread::spawn(move || handle_connection(stream, &directory));
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
    
    Ok(())
}
