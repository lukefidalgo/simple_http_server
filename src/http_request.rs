use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read},
    net::TcpStream,
};

#[allow(unused)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

#[allow(unused)]
impl HttpRequest {
    pub fn from_stream(reader: &mut BufReader<TcpStream>)-> std::io::Result<Self> {
        let mut request_line = String::new();    
        reader.read_line(&mut request_line)?;

        let mut parts = request_line.split_whitespace();
        let method = parts.next().unwrap_or("").to_string();
        let path = parts.next().unwrap_or("").to_string();
        let version = parts.next().unwrap_or("").to_string();

        let mut headers = HashMap::new();
        loop {
            let mut line = String::new();
            reader.read_line(&mut line)?;
            
            if line == "\r\n" || line == "\n" {
                break; // End of headers
            }
            
            if let Some((key, value)) = line.split_once(':') {
                headers.insert(
                    key.trim().to_string(),
                    value.trim().to_string(),
                );
            }
        }

        let body = if let Some(length) = headers.get("Content-Length") {
            if let Ok(length) = length.parse::<usize>() {
                let mut body = vec![0; length];
                reader.read_exact(&mut body)?;
                Some(String::from_utf8_lossy(&body).to_string())
            } else {
                None
            }
        } else {
            None
        };

        Ok(Self {
            method,
            path,
            version,
            headers,
            body,
        })
    }

    pub fn print_request(&self) {
        println!("method: {}\npath: {}", self.method, self.path);
        println!("Headers:");
        for (key, value) in &self.headers {
            println!("{}: {}", key, value);
        }
        match &self.body {
            Some(body) => println!("Body: {}\n", body),
            None => println!("No body present\n"),
        }
    }
}
