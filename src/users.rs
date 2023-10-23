use std::sync::{Arc, Mutex};

use hyper::{Body, Response};
use hyper::http::StatusCode;

use serde_json::json;
use sysinfo::{System, SystemExt, UserExt};
pub(crate) async fn handle_users(system: Arc<Mutex<System>>) -> Result<Response<Body>, hyper::Error> {
    let mut system = system.lock().unwrap();
    system.refresh_users_list();
    let logged_in_users = _get_logged_in_users();

    let users: Vec<_> = system.users().iter()
        .filter(|user| logged_in_users.contains(&user.name().to_string()))
        .map(|user| {
            json!({
                "name": user.name(),
                "group": user.groups()
            })
        }).collect();


    let response = match Response::builder()
        .header("Content-Type", "application/json")
        .body(Body::from(json!(users).to_string())) {
        Ok(res) => res,
        Err(_) => Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Body::empty()).unwrap(),
    };
    Ok(response)
}

fn _get_logged_in_users() -> Vec<String> {
    use std::process::Command;

    let output = Command::new("who")
        .arg("-q")
        .output()
        .expect("failed to execute `who` command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.lines().next().unwrap_or_default().split_whitespace().map(|s| s.to_string()).collect()
}
