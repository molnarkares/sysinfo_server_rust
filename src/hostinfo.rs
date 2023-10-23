use std::sync::{Arc, Mutex};

use hyper::{Body, Response};
use hyper::http::StatusCode;

use serde_json::json;
use sysinfo::{System, SystemExt};

pub(crate) async fn handle_system_info(system: Arc<Mutex<System>>) -> Result<Response<Body>, hyper::Error> {
    let mut system = system.lock().unwrap();
    system.refresh_system();
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

    const TEST_SERVER_ADDR: &str = "127.0.0.1:8093"; // Use a different port for testing

    #[tokio::test]
    async fn test_sysinfo_endpoint() {
        // Start the server in a background task
        tokio::spawn(async {
            let addr: SocketAddr = TEST_SERVER_ADDR.parse().expect("Invalid socket address");
            sysinfo_test_server(addr).await;
        });

        // Give the server a moment to start
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let response: serde_json::Value = reqwest::get(&format!("http://{}/sysinfo", TEST_SERVER_ADDR))
            .await
            .expect("Failed to send request")
            .json()
            .await
            .expect("Failed to parse response as JSON");

        assert!(response.is_array());
        let sysinfo_array = response.as_array().expect("Response is not an array");

        for sysinfo in sysinfo_array {
            assert!(sysinfo.is_object());
            let sysinfo_obj = sysinfo.as_object().unwrap();

            assert!(sysinfo_obj.contains_key("kernel_version"));
            assert!(sysinfo_obj.contains_key("os_version"));
            assert!(sysinfo_obj.contains_key("long_os_version"));
            assert!(sysinfo_obj.contains_key("distribution_id"));
            assert!(sysinfo_obj.contains_key("host_name"));

            assert!(sysinfo_obj["kernel_version"].is_string());
            assert!(sysinfo_obj["os_version"].is_string());
            assert!(sysinfo_obj["long_os_version"].is_string());
            assert!(sysinfo_obj["distribution_id"].is_string());
            assert!(sysinfo_obj["host_name"].is_string());
        }
    }

    async fn sysinfo_test_server(addr: SocketAddr) {
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
