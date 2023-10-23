use std::sync::{Arc, Mutex};

use hyper::{Body, Response};
use hyper::http::StatusCode;

use serde_json::json;
use sysinfo::{System, SystemExt};
pub(crate) async fn handle_memory(system: Arc<Mutex<System>>) -> Result<Response<Body>, hyper::Error> {
    let mut system = system.lock().unwrap();
    system.refresh_memory();
    let memory_info = json!([{
        "available_memory": system.available_memory(),
        "free_memory": system.free_memory(),
        "free_swap": system.free_swap(),
        "total_memory": system.total_memory(),
        "total_swap": system.total_swap(),
        "used_memory": system.used_memory(),
        "used_swap": system.used_swap(),
    }]);
    let response = match Response::builder()
        .header("Content-Type", "application/json")
        .body(Body::from(json!(memory_info).to_string())) {
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

    const TEST_SERVER_ADDR: &str = "127.0.0.1:8086"; // Use a different port for testing

    #[tokio::test]
    async fn test_memory_endpoint() {
        // Start the server in a background task
        tokio::spawn(async {
            let addr: SocketAddr = TEST_SERVER_ADDR.parse().expect("Invalid socket address");
            memory_test_server(addr).await;
        });

        // Give the server a moment to start
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let response: serde_json::Value = reqwest::get(&format!("http://{}/memory", TEST_SERVER_ADDR))
            .await
            .expect("Failed to send request")
            .json()
            .await
            .expect("Failed to parse response as JSON");

        let memory_info_array = response.as_array().expect("Response is not an array");

        for memory_info in memory_info_array {
            assert!(memory_info.is_object());
            let memory_obj = memory_info.as_object().unwrap();

            assert!(memory_obj.contains_key("available_memory"));
            assert!(memory_obj.contains_key("free_memory"));
            assert!(memory_obj.contains_key("free_swap"));
            assert!(memory_obj.contains_key("total_memory"));
            assert!(memory_obj.contains_key("total_swap"));
            assert!(memory_obj.contains_key("used_memory"));
            assert!(memory_obj.contains_key("used_swap"));

            let available_memory = memory_obj.get("available_memory").and_then(|v| v.as_u64()).expect("`available_memory` is not an integer");
            let free_memory = memory_obj.get("free_memory").and_then(|v| v.as_u64()).expect("`free_memory` is not an integer");
            let free_swap = memory_obj.get("free_swap").and_then(|v| v.as_u64()).expect("`free_swap` is not an integer");

            let total_memory = memory_obj.get("total_memory").and_then(|v| v.as_u64()).expect("`total_memory` is not an integer");
            let total_swap = memory_obj.get("total_swap").and_then(|v| v.as_u64()).expect("`total_swap` is not an integer");
            let used_memory = memory_obj.get("used_memory").and_then(|v| v.as_u64()).expect("`used_memory` is not an integer");
            let used_swap = memory_obj.get("used_swap").and_then(|v| v.as_u64()).expect("`used_swap` is not an integer");

            assert!(used_memory <= total_memory);
            assert!(available_memory <= total_memory);
            assert!(free_memory+used_memory <= total_memory);

            assert!(used_swap <= total_swap);
            assert!(free_swap <= total_swap);
            assert!(free_swap+used_swap <= total_swap);

        }
    }

    async fn memory_test_server(addr: SocketAddr) {
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
