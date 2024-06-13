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

    //     match method {
    //         "GET" => {
    //             handle_get_request(&mut stream, &full_path)?;
    //         }
    //         "POST" => {
    //             handle_post_request(&mut stream, &full_path)?;
    //         }
    //         _ => {
    //             // Unsupported method
    //             let response = format!("HTTP/1.1 405 Method Not Allowed\r\n\r\n");
    //             stream.write_all(response.as_bytes())?;
    //         }
    //     }
    // }

    Ok(())
}

fn handle_get_request(stream: &mut TcpStream, full_path: &Path) -> io::Result<()> {
    // Check if file exists

    Ok(())
}

fn handle_post_request(stream: &mut TcpStream, script_path: &Path) -> io::Result<()> {
    // Check if script file exists


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
