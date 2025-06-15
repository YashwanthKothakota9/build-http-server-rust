#[allow(unused_imports)]
use std::net::TcpListener;
use std::{
    collections::HashMap,
    env, fs,
    io::{Read, Write},
    net::TcpStream,
    thread,
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

fn response_with_body(body: &str, file: bool) -> String {
    if file {
        return format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );
    }
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
        body.len(),
        body
    )
}

fn handle_connection(stream: &mut TcpStream, dir: Option<String>) {
    println!("accepted new connection");
    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer).unwrap();
    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    let request_parsed = request_parser(&request);
    let path = &request_parsed.path;
    let method = &request_parsed.method;
    let response = if path == "/" {
        "HTTP/1.1 200 OK\r\n\r\n".to_string()
    } else if path == "/user-agent" {
        response_with_body(request_parsed.headers.get("User-Agent").unwrap(), false)
    } else if path.starts_with("/echo") {
        response_with_body(&path[6..], false)
    } else if path.starts_with("/files") {
        let file_name = &path[7..];
        let file_path = format!("{}/{}", dir.unwrap(), file_name);
        if method == "GET" {
            if let Ok(content) = fs::read_to_string(file_path) {
                response_with_body(&content, true)
            } else {
                "HTTP/1.1 404 Not Found\r\n\r\n".to_string()
            }
        } else if method == "POST" {
            let content_length = request_parsed.headers.get("Content-Length").unwrap();
            let content_length_int = content_length.parse::<usize>().unwrap();
            let body = request_parsed.body;
            if body.len() != content_length_int {
                "HTTP/1.1 400 Bad Request\r\n\r\n".to_string()
            } else if fs::write(file_path, body).is_ok() {
                "HTTP/1.1 201 Created\r\n\r\n".to_string()
            } else {
                "HTTP/1.1 500 Internal Server Error\r\n\r\n".to_string()
            }
        } else {
            "HTTP/1.1 405 Method Not Allowed\r\n\r\n".to_string()
        }
    } else {
        "HTTP/1.1 404 Not Found\r\n\r\n".to_string()
    };
    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let dir = if args.len() > 2 {
        args[2].clone()
    } else {
        "".to_string()
    };

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                let dir_clone = dir.clone();
                thread::spawn(move || {
                    handle_connection(
                        &mut _stream,
                        if !dir_clone.is_empty() {
                            Some(dir_clone)
                        } else {
                            None
                        },
                    );
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
