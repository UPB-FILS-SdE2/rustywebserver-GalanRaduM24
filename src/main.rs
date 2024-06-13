use std::env;
use std::fs;
use std::io::Read;
use std::io::{self, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn main() -> io::Result<()> {
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
        handle_connection(stream, root_folder.clone())?;
    }

    Ok(())
}

fn handle_connection(mut stream: TcpStream, root_folder: PathBuf) -> io::Result<()> {
    // Read HTTP request
    let mut buffer = String::new();
    stream.read_to_string(&mut buffer)?;

    // Parse HTTP request
    let mut lines = buffer.lines();
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
                handle_get_request(&mut stream, &full_path)?;
            }
            "POST" => {
                handle_post_request(&mut stream, &full_path)?;
            }
            _ => {
                // Unsupported method
                let response = format!("HTTP/1.1 405 Method Not Allowed\r\n\r\n");
                stream.write_all(response.as_bytes())?;
            }
        }
    }

    Ok(())
}

fn handle_get_request(stream: &mut TcpStream, full_path: &Path) -> io::Result<()> {
    // Check if file exists
    if full_path.is_file() {
        // Determine content type based on file extension
        let content_type = determine_content_type(&full_path);
        let file_content = fs::read_to_string(&full_path)?;

        // Respond with file contents
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-type: {}\r\nConnection: closed\r\n\r\n{}",
            content_type, file_content
        );
        stream.write_all(response.as_bytes())?;
    } else {
        // File not found
        let response = "HTTP/1.1 404 Not Found\r\nConnection: closed\r\n\r\n";
        stream.write_all(response.as_bytes())?;
    }

    Ok(())
}

fn handle_post_request(stream: &mut TcpStream, script_path: &Path) -> io::Result<()> {
    // Check if script file exists
    if script_path.is_file() {
        // Execute the script
        let output = Command::new(script_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?
            .wait_with_output()?;

        // Handle script execution result
        if output.status.success() {
            // Script executed successfully
            let stdout = String::from_utf8_lossy(&output.stdout);
            let response = format!(
                "HTTP/1.1 200 OK\r\nConnection: closed\r\n\r\n{}",
                stdout
            );
            stream.write_all(response.as_bytes())?;
        } else {
            // Script execution failed
            let stderr = String::from_utf8_lossy(&output.stderr);
            let response = format!(
                "HTTP/1.1 500 Internal Server Error\r\nConnection: closed\r\n\r\n{}",
                stderr
            );
            stream.write_all(response.as_bytes())?;
        }
    } else {
        // Script not found
        let response = "HTTP/1.1 404 Not Found\r\nConnection: closed\r\n\r\n";
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
