use std::sync::{Arc, Mutex};
use hyper::{Body, Response};
use hyper::http::StatusCode;
use serde_json::json;
use sysinfo::{ComponentExt, System, SystemExt};

pub(crate) async fn handle_temperatures(system: Arc<Mutex<System>>) -> Result<Response<Body>, hyper::Error> {
    let mut system = system.lock().unwrap();
    system.refresh_components();
    let temperatures: Vec<_> = system.components().iter().filter_map(|component| {
        let label = component.label();
        let temperature = component.temperature();

        if label.is_empty() {
            None
        } else {
            Some(json!({
                "name": label,
                "temperature": temperature,
            }))
        }
    }).collect();


    let body_data = json!({
        "temperature_info": temperatures
    });

    let response = match Response::builder()
        .header("Content-Type", "application/json")
        .body(Body::from(json!(body_data).to_string())) {
        Ok(res) => res,
        Err(_) => Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Body::empty()).unwrap(),
    };
    Ok(response)
}
