use std::sync::{Arc, Mutex};

use hyper::{Body, Response};
use hyper::http::StatusCode;

use serde_json::json;
use sysinfo::{System, SystemExt};

pub(crate) async fn handle_boot_time(system: Arc<Mutex<System>>) -> Result<Response<Body>, hyper::Error> {
    let mut system = system.lock().unwrap();
    system.refresh_system();
    let boot_time = json!({
        "boot_time": system.boot_time(),
    });

    let response = match Response::builder()
        .header("Content-Type", "application/json")
        .body(Body::from(json!(boot_time).to_string())) {
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

    const TEST_SERVER_ADDR: &str = "127.0.0.1:8082"; // Use a different port for testing

    #[tokio::test]
    async fn test_boot_time_endpoint() {
        // Start the server in a background task
        tokio::spawn(async {
            let addr: SocketAddr = TEST_SERVER_ADDR.parse().expect("Invalid socket address");
            boot_time_test_server(addr).await;
        });

        // Give the server a moment to start
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let response_json: serde_json::Value = reqwest::get(&format!("http://{}/boot_time", TEST_SERVER_ADDR))
            .await
            .expect("Failed to send request")
            .json()
            .await
            .expect("Failed to parse response as JSON");

        assert!(response_json["boot_time"].is_number());

        // Check if the received JSON contains expected keys
        let response = response_json["boot_time"].as_u64().unwrap();
        // Ensure the returned boot time is a valid timestamp and it's in the past
        assert!(response > 0);
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        assert!(response <= current_time);

    }

    async fn boot_time_test_server(addr: SocketAddr) {
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
//     // Check if the received JSON contains expected keys
//     let response = &response_json["boot_time"];
//     assert!(response.is_number());
//     // Ensure the returned boot time is a valid timestamp and it's in the past
//     // assert!(response > 0.0);
//     // let current_time = std::time::SystemTime::now()
//     //     .duration_since(std::time::SystemTime::UNIX_EPOCH)
//     //     .expect("Time went backwards")
//     //     .as_secs_f64();
//     // assert!(response <= current_time);
// }
//
