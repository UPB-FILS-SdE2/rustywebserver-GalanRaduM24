// tests/web_server_tests.rs

use std::process::{Command, Child};
use std::thread;
use std::time::Duration;
use reqwest::blocking::Client;
use std::fs::{self, File};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;

fn start_server(port: &str, root_folder: &str) -> Child {
    Command::new("./target/debug/rustywebserver")
        .arg(port)
        .arg(root_folder)
        .spawn()
        .expect("Failed to start server")
}

fn stop_server(server: &mut Child) {
    server.kill().expect("Failed to stop server");
    server.wait().expect("Failed to wait on server");
}

fn setup_test_files(root_folder: &str) {
    fs::create_dir_all(root_folder).expect("Failed to create root folder");
    let mut file = File::create(format!("{}/index.html", root_folder)).expect("Failed to create test file");
    writeln!(file, "<html><body>Hello, world!</body></html>").expect("Failed to write to test file");
}

#[test]
fn test_get_request_success() {
    let port = "8000";
    let root_folder = "./test_root";
    setup_test_files(root_folder);

    let mut server = start_server(port, root_folder);
    thread::sleep(Duration::from_secs(1)); // Give the server time to start

    let client = Client::new();
    let res = client.get(&format!("http://localhost:{}/index.html", port))
        .send()
        .expect("Failed to send request");

    assert_eq!(res.status(), 200);
    assert_eq!(res.text().expect("Failed to read response text"), "<html><body>Hello, world!</body></html>");

    stop_server(&mut server);
}

#[test]
fn test_get_request_not_found() {
    let port = "8000";
    let root_folder = "./test_root";
    setup_test_files(root_folder);

    let mut server = start_server(port, root_folder);
    thread::sleep(Duration::from_secs(1)); // Give the server time to start

    let client = Client::new();
    let res = client.get(&format!("http://localhost:{}/nonexistent.html", port))
        .send()
        .expect("Failed to send request");

    assert_eq!(res.status(), 404);

    stop_server(&mut server);
}

#[test]
fn test_get_request_forbidden() {
    let port = "8000";
    let root_folder = "./test_root";
    setup_test_files(root_folder);

    // Make a file unreadable
    let forbidden_file = format!("{}/forbidden.html", root_folder);
    File::create(&forbidden_file).expect("Failed to create forbidden file");
    fs::set_permissions(&forbidden_file, fs::Permissions::from_mode(0o000)).expect("Failed to set permissions");

    let mut server = start_server(port, root_folder);
    thread::sleep(Duration::from_secs(1)); // Give the server time to start

    let client = Client::new();
    let res = client.get(&format!("http://localhost:{}/forbidden.html", port))
        .send()
        .expect("Failed to send request");

    assert_eq!(res.status(), 403);

    stop_server(&mut server);
    fs::remove_file(&forbidden_file).expect("Failed to remove forbidden file");
}

#[test]
fn test_post_request_success() {
    let port = "8000";
    let root_folder = "./test_root";
    setup_test_files(root_folder);

    let mut server = start_server(port, root_folder);
    thread::sleep(Duration::from_secs(1)); // Give the server time to start

    let client = Client::new();
    let res = client.post(&format!("http://localhost:{}/index.html", port))
        .body("This is a test")
        .send()
        .expect("Failed to send request");

    assert_eq!(res.status(), 200);

    stop_server(&mut server);
}

#[test]
fn test_script_execution_success() {
    let port = "8000";
    let root_folder = "./test_root";
    let scripts_folder = format!("{}/scripts", root_folder);
    fs::create_dir_all(&scripts_folder).expect("Failed to create scripts folder");
    
    let script_path = format!("{}/test_script.sh", scripts_folder);
    let mut script = File::create(&script_path).expect("Failed to create script file");
    writeln!(script, "#!/bin/sh\necho 'Script executed'").expect("Failed to write to script file");
    fs::set_permissions(&script_path, fs::Permissions::from_mode(0o755)).expect("Failed to set script permissions");

    let mut server = start_server(port, root_folder);
    thread::sleep(Duration::from_secs(1)); // Give the server time to start

    let client = Client::new();
    let res = client.get(&format!("http://localhost:{}/scripts/test_script.sh", port))
        .send()
        .expect("Failed to send request");

    assert_eq!(res.status(), 200);
    assert_eq!(res.text().expect("Failed to read response text"), "Script executed");

    stop_server(&mut server);
    fs::remove_file(&script_path).expect("Failed to remove script file");
}
