use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;

#[tokio::main]
async fn main() -> io::Result<()> {
    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <port> <root_folder>", args[0]);
        std::process::exit(1);
    }

    let port = args[1].parse::<u16>().expect("Invalid port number");
    let root_folder = PathBuf::from(&args[2]);

    // Print startup information
    println!("Root folder: {}", root_folder.display());
    println!("Server listening on 0.0.0.0:{}", port);

    // Start server
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))?;
    for stream in listener.incoming() {
        let stream = stream?;
        let root_folder = root_folder.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, root_folder).await {
                eprintln!("Error handling connection: {}", e);
            }
        });
    }
    Ok(())
}

async fn handle_connection(mut stream: TcpStream, root_folder: PathBuf) -> io::Result<()> {
    // Read the request
    let mut buffer = [0; 1024];
    stream.read(&mut buffer)?;

    // Convert the request buffer to a string
    let request = String::from_utf8_lossy(&buffer[..]);
    
    // Parse the request to get method and path
    let (method, path) = parse_request(&request);

    // Prepare full file path
    let full_path = root_folder.join(&path[1..]);

    // Generate response based on method and path
    let response = if path.starts_with("/..") || path.starts_with("/forbidden") {
        // Forbidden paths
        format!("HTTP/1.1 403 Forbidden\r\nConnection: close\r\n\r\n<html>403 Forbidden</html>")
    } else if path.starts_with("/scripts/") {
        // Handle scripts execution
        match method.as_str() {
            "GET" | "POST" => {
                if full_path.is_file() {
                    execute_script(&full_path, &method, &request).await.unwrap_or_else(|_| {
                        format!("HTTP/1.1 500 Internal Server Error\r\nConnection: close\r\n\r\n<html>500 Internal Server Error</html>")
                    })
                } else {
                    format!("HTTP/1.1 404 Not Found\r\nConnection: close\r\n\r\n<html>404 Not Found</html>")
                }
            }
            _ => format!("HTTP/1.1 405 Method Not Allowed\r\nConnection: close\r\n\r\n<html>405 Method Not Allowed</html>")
        }
    } else {
        // Serve static files
        match method.as_str() {
            "GET" => {
                if full_path.is_file() {
                    serve_file(&full_path).await.unwrap_or_else(|_| {
                        format!("HTTP/1.1 500 Internal Server Error\r\nConnection: close\r\n\r\n<html>500 Internal Server Error</html>")
                    })
                } else {
                    format!("HTTP/1.1 404 Not Found\r\nConnection: close\r\n\r\n<html>404 Not Found</html>")
                }
            }
            _ => format!("HTTP/1.1 405 Method Not Allowed\r\nConnection: close\r\n\r\n<html>405 Method Not Allowed</html>")
        }
    };

    // Write the response to the client
    stream.write_all(response.as_bytes())?;
    stream.flush()?;

    Ok(())
}

fn parse_request(request: &str) -> (String, String) {
    let mut method = String::new();
    let mut path = String::new();

    if let Some(first_line_end) = request.find("\r\n") {
        if let Some(space_idx) = request.find(' ') {
            method = request[0..space_idx].to_string();
            if let Some(path_end_idx) = request[space_idx + 1..].find(' ') {
                path = request[space_idx + 1..space_idx + 1 + path_end_idx].to_string();
            }
        }
    }

    (method, path)
}

async fn execute_script(script_path: &Path, method: &str, request: &str) -> io::Result<String> {
    let output = if method == "GET" {
        Command::new(script_path)
            .output()?
    } else {
        let body = request.split("\r\n\r\n").last().unwrap_or("");
        Command::new(script_path)
            .arg(body)
            .output()?
    };

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

async fn serve_file(file_path: &Path) -> io::Result<String> {
    let content = fs::read(file_path)?;
    let content_type = mime_guess::from_path(file_path).first_or_octet_stream().to_string();

    Ok(format!("HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", content_type, content.len()) + &String::from_utf8_lossy(&content))
}
