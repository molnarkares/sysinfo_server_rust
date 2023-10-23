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

    const TEST_SERVER_ADDR: &str = "127.0.0.1:8088"; // Use a different port for testing

    #[tokio::test]
    async fn test_temperatures_endpoint() {
        // Start the server in a background task
        tokio::spawn(async {
            let addr: SocketAddr = TEST_SERVER_ADDR.parse().expect("Invalid socket address");
            temperatures_test_server(addr).await;
        });

        // Give the server a moment to start
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let response: serde_json::Value = reqwest::get(&format!("http://{}/temperatures", TEST_SERVER_ADDR))
            .await
            .expect("Failed to send request")
            .json()
            .await
            .expect("Failed to parse response as JSON");

        // Check if the received JSON contains expected keys
        assert!(response.is_object());

        let temperature_info = response["temperature_info"].as_array().expect("Temperature info is not an array");

        for temp in temperature_info {
            assert!(temp.is_object());
            let temp_obj = temp.as_object().unwrap();

            assert!(temp_obj.contains_key("name"));
            assert!(temp_obj.contains_key("temperature"));

            assert!(temp_obj["name"].is_string());
            assert!(temp_obj["temperature"].is_f64());
            let temperature_value = temp_obj["temperature"].as_f64().unwrap();
            assert!(temperature_value > -100.0 && temperature_value < 200.0);
        }
    }
    async fn temperatures_test_server(addr: SocketAddr) {
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
