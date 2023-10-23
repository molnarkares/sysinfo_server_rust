use std::sync::{Arc, Mutex};

use hyper::{Body, Response};
use hyper::http::StatusCode;

use serde_json::json;
use sysinfo::{NetworksExt, System, SystemExt , NetworkExt};
pub(crate) async fn handle_networks(system: Arc<Mutex<System>>) -> Result<Response<Body>, hyper::Error> {
    let mut system = system.lock().unwrap();
    system.refresh_networks_list();
    let networks: Vec<_> = system.networks().iter()
        .map(|(interface_name, network)| {
            json!({
                "interface_name": interface_name,
                "data_received": network.received(),
                "data_transmitted": network.transmitted(),
            })
        })
        .collect();

    let response = match Response::builder()
        .header("Content-Type", "application/json")
        .body(Body::from(json!(networks).to_string())) {
        Ok(res) => res,
        Err(_) => Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Body::empty()).unwrap(),
    };
    Ok(response)
}

