use std::sync::{Arc, Mutex};

use hyper::{Body, Response};
use hyper::http::StatusCode;

use serde_json::json;
use sysinfo::{System, SystemExt};

pub(crate) async fn handle_system_info(system: Arc<Mutex<System>>) -> Result<Response<Body>, hyper::Error> {
    let mut system = system.lock().unwrap();
    system.refresh_all();

    let system_data = json!([{
        "kernel_version": system.kernel_version().unwrap_or_else(|| "N/A".to_string()),
        "os_version": system.os_version().unwrap_or_else(|| "N/A".to_string()),
        "long_os_version": system.long_os_version().unwrap_or_else(|| "N/A".to_string()),
        "distribution_id": system.distribution_id(),
        "host_name": system.host_name().unwrap_or_else(|| "N/A".to_string()),
    }]);

    let response = match Response::builder()
        .header("Content-Type", "application/json")
        .body(Body::from(json!(system_data).to_string())) {
        Ok(res) => res,
        Err(_) => Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Body::empty()).unwrap(),
    };
    Ok(response)
}

