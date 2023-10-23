use std::sync::{Arc, Mutex};

use hyper::{Body, Response};
use hyper::http::StatusCode;

use serde_json::json;
use sysinfo::{System, SystemExt};
pub(crate) async fn handle_memory(system: Arc<Mutex<System>>) -> Result<Response<Body>, hyper::Error> {
    let mut system = system.lock().unwrap();
    system.refresh_memory();

    let memory_info = json!([{
        "available_memory": system.available_memory(),
        "free_memory": system.free_memory(),
        "free_swap": system.free_swap(),
        "total_memory": system.total_memory(),
        "total_swap": system.total_swap(),
        "used_memory": system.used_memory(),
        "used_swap": system.used_swap(),
    }]);
    let response = match Response::builder()
        .header("Content-Type", "application/json")
        .body(Body::from(json!(memory_info).to_string())) {
        Ok(res) => res,
        Err(_) => Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Body::empty()).unwrap(),
    };
    Ok(response)
}

