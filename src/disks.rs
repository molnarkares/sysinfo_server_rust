use std::sync::{Arc, Mutex};

use hyper::{Body, Response};
use hyper::http::StatusCode;

use serde_json::json;
use sysinfo::{System, SystemExt, DiskExt};
pub(crate) async fn handle_disks(system: Arc<Mutex<System>>) -> Result<Response<Body>, hyper::Error> {
    let mut system = system.lock().unwrap();
    system.refresh_disks();
    let disks_info: Vec<_> = system.disks().iter().map(|disk| {
        json!({
            "device_name": disk.name().to_str().unwrap_or_default(),
            "file_system": std::str::from_utf8(disk.file_system()).unwrap_or_default(),
            "total_space": disk.total_space(),
            "available_space": disk.available_space()
        })
    }).collect();
    let response = match Response::builder()
        .header("Content-Type", "application/json")
        .body(Body::from(json!(disks_info).to_string())) {
        Ok(res) => res,
        Err(_) => Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Body::empty()).unwrap(),
    };
    Ok(response)
}
