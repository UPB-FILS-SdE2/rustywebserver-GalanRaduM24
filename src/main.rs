use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use std::process::Stdio;
use tokio::runtime::Runtime;


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
            handle_connection(stream, root_folder).await.unwrap();
        });
    }
    Ok(())
}


async fn handle_connection(mut stream: TcpStream, root_folder: PathBuf) -> io::Result<()> {
    // Read HTTP request
    let mut buffer = [0; 1024];
    stream.read(&mut buffer)?;
    let request = String::from_utf8_lossy(&buffer[..]);

    // Parse the HTTP request
    let (method, path, query) = {
        // Split request lines
        let lines: Vec<&str> = request.lines().collect();
        if lines.is_empty() {
            ("".to_string(), "".to_string(), None)
        } else {
            // Split the request line into method, path, and HTTP version
            let mut parts = lines[0].split_whitespace();
            let method = parts.next().unwrap_or("").to_string();
            let mut path = parts.next().unwrap_or("").to_string();
            let _http_version = parts.next().unwrap_or(""); // Not used

            // Check if the path contains a query string
            let query = if let Some(index) = path.find('?') {
                let query = path.split_off(index + 1);
                path.pop(); // Remove the '?' from the end of path
                Some(query)
            } else {
                None
            };

            (method, path, query)
        }
    };

    // Delegate to the appropriate handler based on the HTTP method
    match method.as_str() {
        "GET" => handle_get_request(&mut stream, &root_folder, &path, query).await,
        "POST" => handle_post_request(&mut stream, &root_folder, &path, &request).await,
        _ => {
            println!("{} 127.0.0.1 {} -> 405 (Method Not Allowed)", method, path);
            let response = b"HTTP/1.1 405 Method Not Allowed\r\nConnection: close\r\n\r\n<html>405 Method Not Allowed</html>";
            stream.write_all(response)?;
            Ok(())
        }
    }
}

// Asynchronous function to handle GET requests
async fn handle_get_request(
    stream: &mut TcpStream,
    root_folder: &Path,
    path: &str,
    query: Option<String>,
) -> io::Result<()> {
    // Construct the full path to the requested file
    let full_path = root_folder.join(&path[1..]); // Remove the leading '/' from the path

    // Check if the requested path is forbidden
    if path.starts_with("/..") || path.starts_with("/forbidden") {
        println!("GET 127.0.0.1 {} -> 403 (Forbidden)", path);
        let response = b"HTTP/1.1 403 Forbidden\r\nConnection: close\r\n\r\n<html>403 Forbidden</html>";
        stream.write_all(response)?;
        return Ok(());
    }

    // Check if the requested file exists and is a file
    if full_path.is_file() {
        // Read the file contents
        let contents = fs::read(&full_path)?;
        // Determine the MIME type of the file
        let mime_type = determine_content_type(&full_path);

        // Construct the HTTP response
        println!("GET 127.0.0.1 {} -> 200 (OK)", path);
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            mime_type,
            contents.len(),
        );
        stream.write_all(response.as_bytes())?;
        stream.write_all(&contents)?;
    } else {
        // File not found
        println!("GET 127.0.0.1 {} -> 404 (Not Found)", path);
        let response = b"HTTP/1.1 404 Not Found\r\nConnection: close\r\n\r\n<html>404 Not Found</html>";
        stream.write_all(response)?;
    }

    Ok(())
}


fn determine_content_type(file_path: &Path) -> &'static str {
    match file_path.extension().and_then(|ext| ext.to_str()) {
        Some("txt") => "text/plain; charset=utf-8",
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("jpg") => "image/jpeg",
        Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("zip") => "application/zip",
        _ => "application/octet-stream",
    }
}

async fn handle_post_request(
    stream: &mut std::net::TcpStream,
    root_folder: &PathBuf,
    path: &str,
    request: &str,
) -> std::io::Result<()> {
    let full_path = root_folder.join(&path[1..]);

    if full_path.is_file() {
        let mut cmd = Command::new(&full_path);

        // Set environment variables from query parameters
        if let Some(query) = extract_query_string(request) {
            let query_pairs = query.split('&').map(|pair| {
                let mut split = pair.split('=');
                (
                    split.next().unwrap_or("").to_string(),
                    split.next().unwrap_or("").to_string(),
                )
            });

            for (key, value) in query_pairs {
                let env_var = format!("Query_{}", key);
                cmd.env(env_var, value);
            }
        }

        // Additional environment variables required by the script
        cmd.env("Method", "POST");
        cmd.env("Path", path);

        let output = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to execute script")
            .wait_with_output()
            .await
            .expect("Failed to read stdout");

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let (headers, body_start_index) = parse_headers(&output_str);
            let body = output_str.lines().skip(body_start_index).collect::<Vec<_>>().join("\n");
            let content_type = headers
            .iter()
            .find(|&&(ref k, _)| *k == "Content-Type")
            .map(|&(_, ref v)| v.clone())
            .unwrap_or_else(|| "text/plain".to_string());
        
        let content_length = headers
            .iter()
            .find(|&&(ref k, _)| *k == "Content-Length")
            .map(|&(_, ref v)| v.clone())
            .unwrap_or_else(|| body.len().to_string());

            println!("POST 127.0.0.1 {} -> 200 (OK)", path);

            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                content_type, content_length, body
            ).as_bytes().to_vec();
            
            stream.write_all(&response)?;
        } else {
            println!("POST 127.0.0.1 {} -> 500 (Internal Server Error)", path);
            let response = b"HTTP/1.1 500 Internal Server Error\r\nConnection: close\r\n\r\n<html>500 Internal Server Error</html>".to_vec();
            stream.write_all(&response)?;
        }
    } else {
        println!("POST 127.0.0.1 {} -> 404 (Not Found)", path);
        let response = b"HTTP/1.1 404 Not Found\r\nConnection: close\r\n\r\n<html>404 Not Found</html>".to_vec();
        stream.write_all(&response)?;
    }

    Ok(())
}

fn extract_query_string(request: &str) -> Option<&str> {
    // Find the start of the request line
    if let Some(start_index) = request.find("\r\n") {
        let request_line = &request[..start_index];

        // Find the start of the query string (after the method and path)
        if let Some(path_index) = request_line.find(' ') {
            if let Some(query_start) = request_line[path_index..].find('?') {
                let query_start = path_index + query_start + 1; // Skip '?'
                if let Some(query_end) = request_line[path_index + query_start..].find(' ') {
                    return Some(&request_line[path_index + query_start..path_index + query_start + query_end]);
                }
            }
        }
    }

    None
}

fn parse_headers(response: &str) -> (Vec<(String, String)>, usize) {
    let mut headers = Vec::new();
    let mut body_start_index = 0;

    // Split the response into lines
    let lines: Vec<&str> = response.lines().collect();

    // Iterate over lines to parse headers
    for (index, line) in lines.iter().enumerate() {
        if line.is_empty() {
            // Empty line indicates end of headers, body starts after this
            body_start_index = index + 1;
            break;
        }

        // Split each line into key-value pairs
        if let Some((key, value)) = parse_header_line(line) {
            headers.push((key, value));
        }
    }

    (headers, body_start_index)
}

fn parse_header_line(line: &str) -> Option<(String, String)> {
    if let Some(separator_index) = line.find(':') {
        let key = line[..separator_index].trim().to_string();
        let value = line[separator_index + 1..].trim().to_string();
        Some((key, value))
    } else {
        None
    }
}


/* 
use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use tokio::process::Command;
use std::process::Stdio;
use tokio::runtime::Runtime;


#[tokio::main]
async fn main() {
    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <port> <root_folder>", args[0]);
        std::process::exit(1);
    }

    let port = args[1].parse::<u16>().expect("Invalid port number");
    let root_folder = Path::new(&args[2]).to_path_buf();

    // Print startup information
    println!("Root folder: {}", root_folder.display());
    println!("Server listening on 0.0.0.0:{}", port);

    // Start server
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))?;
    for stream in listener.incoming() {
        let stream = stream?;
        let root_folder = root_folder.clone();
        tokio::spawn(async move {
            handle_connection(stream, root_folder).await.unwrap();
        });
    }

    Ok(())
}

async fn handle_connection(mut stream: TcpStream, root_folder: PathBuf) -> io::Result<()> {
    // Read HTTP request
    let mut buffer = [0; 1024];
    stream.read(&mut buffer)?;

    // Parse HTTP request
    let request = String::from_utf8_lossy(&buffer[..]);
    let mut lines = request.lines();
    if let Some(request_line) = lines.next() {
        let mut parts = request_line.split_whitespace();
        let method = parts.next().unwrap_or("");
        let path = parts.next().unwrap_or("");
        let _version = parts.next().unwrap_or(""); // HTTP version (not used)

        // Normalize path to avoid directory traversal
        let requested_path = Path::new(&path);
        let full_path = root_folder.join(requested_path.strip_prefix("/").unwrap_or(requested_path));

        // Log the request
        let request_source = stream.peer_addr().unwrap().ip();
        println!("{} {} ->", method, path);

        match method {
            "GET" => {
                handle_get_request(&mut stream, &full_path).await?;
            }
            "POST" => {
                handle_post_request(&mut stream, &full_path).await?;
            }
            _ => {
                // Unsupported method
                let response = "HTTP/1.1 405 Method Not Allowed\r\nConnection: close\r\n\r\n";
                stream.write_all(response.as_bytes())?;
            }
        }
    }

    Ok(())
}

async fn handle_get_request(stream: &mut TcpStream, full_path: &Path) -> io::Result<()> {
    // Check if file exists
    if full_path.is_file() {
        // Determine content type based on file extension
        let content_type = determine_content_type(&full_path);
        let file_content = fs::read(&full_path)?;

        // Respond with file contents
        let response = format!(
            "HTTP/1.1 127.0.0.1 {} 200 (OK)", content_type
            // "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            // content_type,
            // file_content.len()
        );
        stream.write_all(response.as_bytes())?;
        stream.write_all(&file_content)?;
    } else {
        // File not found
        let response = "HTTP/1.1 404 Not Found\r\nConnection: close\r\n\r\n";
        stream.write_all(response.as_bytes())?;
    }

    Ok(())
}

async fn handle_post_request(stream: &mut TcpStream, script_path: &Path) -> io::Result<()> {
    // Check if script file exists
    if script_path.is_file() {
        // Execute the script
        let output = Command::new(script_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        // Handle script execution result
        if output.status.success() {
            // Script executed successfully
            let stdout = String::from_utf8_lossy(&output.stdout);
            let response = format!(
                "HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n{}",
                stdout
            );
            stream.write_all(response.as_bytes())?;
        } else {
            // Script execution failed
            let stderr = String::from_utf8_lossy(&output.stderr);
            let response = format!(
                "HTTP/1.1 500 Internal Server Error\r\nConnection: close\r\n\r\n{}",
                stderr
            );
            stream.write_all(response.as_bytes())?;
        }
    } else {
        // Script not found
        let response = "HTTP/1.1 404 Not Found\r\nConnection: close\r\n\r\n";
        stream.write_all(response.as_bytes())?;
    }

    Ok(())
}

fn determine_content_type(file_path: &Path) -> &'static str {
    match file_path.extension().and_then(|ext| ext.to_str()) {
        Some("txt") => "text/plain; charset=utf-8",
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("jpg") => "image/jpeg",
        Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("zip") => "application/zip",
        _ => "application/octet-stream",
    }
}
*/