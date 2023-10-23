use std::sync::{Arc, Mutex};
use hyper::{Body, Response};
use hyper::http::StatusCode;
use serde_json::json;
use sysinfo::{System, SystemExt, CpuExt};

pub(crate) async fn handle_cpus(system: Arc<Mutex<System>>) -> Result<Response<Body>, hyper::Error> {
    let mut system = system.lock().unwrap();
    system.refresh_cpu();
    let cpu_info: Vec<_> = system.cpus().iter().enumerate().map(|(i, proc)| {
        json!({
            "cpu_num": format!("cpu{}", i),
            "percent": proc.cpu_usage(),
            "frequency": proc.frequency() as u32
        })
    }).collect();
    let body = json!({ "cpu_info": cpu_info });
    let response = match Response::builder()
        .header("Content-Type", "application/json")
        .body(Body::from(body.to_string())) {
        Ok(res) => res,
        Err(_) => Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Body::empty()).unwrap(),
    };
    Ok(response)
}