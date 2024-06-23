[![Review Assignment Due Date](https://classroom.github.com/assets/deadline-readme-button-24ddc0f5d75046c5622901739e7c5dd533143b0c8e959d652212380cedb1ea36.svg)](https://classroom.github.com/a/TXciPqtn)
# Rustwebserver

Detail the homework implementation.


# Overview
This project demonstrates a basic HTTP server written in Rust using the Tokio asynchronous runtime. It handles GET and POST requests, serves static files, and executes scripts located in a specified root folder.

## Features
- GET Requests: Handles requests for static files and executes scripts under /scripts.
- POST Requests: Executes scripts located in the requested path with input from the request body.
- Static File Serving: Determines and serves appropriate content types based on file extensions.
- Error Handling: Responds with appropriate HTTP status codes (404, 403, 405, 500) and error messages for various scenarios.

## Structure
- main.rs: Entry point of the server, handles command-line arguments, starts the TCP listener, and delegates incoming connections to handle_connection.
- handle_connection: Asynchronously handles each incoming TCP connection, parsing HTTP requests, and dispatching to appropriate request handlers (handle_get_request or handle_post_request).
- Request Handlers: Functions (handle_get_request, handle_post_request) process GET and POST requests respectively, serving static files or executing scripts.

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







