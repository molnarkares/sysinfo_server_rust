use std::sync::{Arc, Mutex};

use hyper::{Body, Response};
use hyper::http::StatusCode;

use serde_json::json;
use sysinfo::{System, SystemExt, UserExt};
pub(crate) async fn handle_users(system: Arc<Mutex<System>>) -> Result<Response<Body>, hyper::Error> {
    let mut system = system.lock().unwrap();
    system.refresh_users_list();
    let users: Vec<_> = system.users().iter().map(|user| {
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

