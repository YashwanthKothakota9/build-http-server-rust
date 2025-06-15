#[allow(unused_imports)]
use std::net::TcpListener;
use std::{
    collections::HashMap,
    io::{Read, Write},
};

#[derive(Debug)]
struct Request {
    method: String,
    path: String,
    headers: HashMap<String, String>,
    body: String,
}

fn request_parser(request: &str) -> Request {
    let mut request_lines = request.lines();
    let request_line = request_lines.next().unwrap();
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    let method = parts[0];
    let path = parts[1];
    let mut headers = HashMap::new();
    let mut remaining_lines = Vec::new();
    let mut found_empty = false;

    for line in request_lines {
        if line.is_empty() {
            found_empty = true;
            continue;
        }
        if !found_empty {
            let (key, value) = line.split_once(": ").unwrap();
            headers.insert(key.to_string(), value.to_string());
        } else {
            remaining_lines.push(line);
        }
    }

    let body = remaining_lines.join("\n");
    Request {
        method: method.to_string(),
        path: path.to_string(),
        headers,
        body,
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                println!("accepted new connection");
                let mut buffer = [0; 1024];
                let bytes_read = _stream.read(&mut buffer).unwrap();
                let request = String::from_utf8_lossy(&buffer[..bytes_read]);
                let request_parsed = request_parser(&request);
                println!("Request: {:?}", request_parsed);
                let path = &request_parsed.path;
                let mut response: &str;
                if path == "/" {
                    response = "HTTP/1.1 200 OK\r\n\r\n";
                } else {
                    response = "HTTP/1.1 404 Not Found\r\n\r\n";
                }

                _stream.write_all(response.as_bytes()).unwrap();
                _stream.flush().unwrap();
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
