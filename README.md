[![Review Assignment Due Date](https://classroom.github.com/assets/deadline-readme-button-24ddc0f5d75046c5622901739e7c5dd533143b0c8e959d652212380cedb1ea36.svg)](https://classroom.github.com/a/TXciPqtn)
# Rustwebserver

Detail the homework implementation.


# Overview
This Rust program implements a simple HTTP server capable of handling GET and POST requests. It serves static files from a specified root directory and can execute scripts located in a /scripts directory relative to the root. The server uses Tokio for asynchronous handling of connections and processes.

## Features
- GET Requests: Handles requests for static files and executes scripts under /scripts.
- POST Requests: Executes scripts located in the requested path with input from the request body.
- Error Handling: Responds with appropriate HTTP status codes (404, 403, 405, 500) and error messages for various scenarios.

## Dependencies
- tokio: Used for asynchronous programming and handling I/O operations concurrently.
- std::env: For accessing command-line arguments and environment variables.
- std::fs: Provides file system operations like reading files.
- std::io: Handles input-output operations, including reading from and writing to TCP streams.
- std::net: Used for TCP listener and stream operations.
- std::path: Manages file and directory paths.
- std::process: Invokes processes and manages their input/output streams.

### HTTP Methods Supported
- GET: Retrieves static files and executes scripts under /scripts.
- POST: Executes scripts located in the requested path with input from the request body.

### HTTP Responses
- 200 OK: Successful response with appropriate content and headers.
- 403 Forbidden: Access to requested resource is forbidden.
- 404 Not Found: Requested resource does not exist.
- 405 Method Not Allowed: Requested HTTP method is not supported.

### Security Considerations
- Path Traversal: Prevents access to files outside the specified root directory.
- Script Execution: Executes scripts only from the /scripts directory to mitigate security risks.

### Development Notes
- Concurrency: Uses Tokio's asynchronous runtime for handling multiple client connections efficiently.
- Error Handling: Implements robust error handling to manage unexpected situations gracefully.
- Content Type Detection: Determines the appropriate content type based on file extensions for HTTP responses.







