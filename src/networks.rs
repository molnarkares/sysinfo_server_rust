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

#[cfg(test)]
mod tests {
    use std::convert::Infallible;
    use super::*;
    use std::net::SocketAddr;
    use hyper::Server;
    use hyper::service::{make_service_fn, service_fn};
    use reqwest;
    use tokio;
    use crate::handle_request;

    const TEST_SERVER_ADDR: &str = "127.0.0.1:8087"; // Use a different port for testing

    #[tokio::test]
    async fn test_networks_endpoint() {
        // Start the server in a background task
        tokio::spawn(async {
            let addr: SocketAddr = TEST_SERVER_ADDR.parse().expect("Invalid socket address");
            networks_test_server(addr).await;
        });

        // Give the server a moment to start
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let response: serde_json::Value = reqwest::get(&format!("http://{}/networks", TEST_SERVER_ADDR))
            .await
            .expect("Failed to send request")
            .json()
            .await
            .expect("Failed to parse response as JSON");

        // Check if the received JSON contains expected keys
        assert!(response.is_array());
        let networks_array = response.as_array().expect("Response is not an array");

        for network in networks_array {
            assert!(network.is_object());
            let network_obj = network.as_object().unwrap();

            assert!(network_obj.contains_key("interface_name"));
            assert!(network_obj.contains_key("data_received"));
            assert!(network_obj.contains_key("data_transmitted"));

            assert!(network_obj["data_received"].is_number());
            assert!(network_obj["data_transmitted"].is_number());
            assert!(network_obj["interface_name"].is_string());
        }
    }
    async fn networks_test_server(addr: SocketAddr) {
        let system = Arc::new(Mutex::new(System::new_all()));

        let test_service_load_avg = make_service_fn(move |_| {
            let system = system.clone();
            async {
                Ok::<_, Infallible>(service_fn(move |req| handle_request(req, system.clone())))
            }
        });

        let server = Server::bind(&addr).serve(test_service_load_avg);

        if let Err(e) = server.await {
            eprintln!("server error: {}", e);
        }
    }
}
