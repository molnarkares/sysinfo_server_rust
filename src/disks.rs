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

    const TEST_SERVER_ADDR: &str = "127.0.0.1:8083"; // Use a different port for testing

    #[tokio::test]
    async fn test_disks_endpoint() {
        // Start the server in a background task
        tokio::spawn(async {
            let addr: SocketAddr = TEST_SERVER_ADDR.parse().expect("Invalid socket address");
            disks_test_server(addr).await;
        });

        // Give the server a moment to start
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let response: serde_json::Value = reqwest::get(&format!("http://{}/disks", TEST_SERVER_ADDR))
            .await
            .expect("Failed to send request")
            .json()
            .await
            .expect("Failed to parse response as JSON");

        // Check if the received JSON contains expected type (array)
        let disks_info = response.as_array().expect("Response is not an array");

        for disk in disks_info {
            assert!(disk.is_object());
            let disk_obj = disk.as_object().unwrap();

            assert!(disk_obj.contains_key("device_name"));
            assert!(disk_obj.contains_key("file_system"));
            assert!(disk_obj.contains_key("total_space"));
            assert!(disk_obj.contains_key("available_space"));

            assert!(disk_obj["device_name"].is_string());
            assert!(disk_obj["file_system"].is_string());

            let total_space = disk_obj.get("total_space").and_then(|v| v.as_u64()).expect("`total_space` is not an integer");
            let available_space = disk_obj.get("available_space").and_then(|v| v.as_u64()).expect("`available_space` is not an integer");
            assert!(available_space <= total_space);
        }
    }

    async fn disks_test_server(addr: SocketAddr) {
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
