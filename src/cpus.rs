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
            "frequency": proc.frequency() as u64
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
    async fn test_cpus_endpoint() {
        // Start the server in a background task
        tokio::spawn(async {
            let addr: SocketAddr = TEST_SERVER_ADDR.parse().expect("Invalid socket address");
            cpus_test_server(addr).await;
        });

        // Give the server a moment to start
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let response: serde_json::Value = reqwest::get(&format!("http://{}/cpus", TEST_SERVER_ADDR))
            .await
            .expect("Failed to send request")
            .json()
            .await
            .expect("Failed to parse response as JSON");

        // Check if the received JSON contains expected keys
        assert!(response.is_object());
        let cpu_info = response.get("cpu_info").expect("No `cpu_info` in response").as_array().expect("`cpu_info` is not an array");

        for cpu in cpu_info {
            assert!(cpu.is_object());
            let cpu_obj = cpu.as_object().unwrap();

            assert!(cpu_obj.contains_key("cpu_num"));
            assert!(cpu_obj.contains_key("percent"));
            assert!(cpu_obj.contains_key("frequency"));

            assert!(cpu_obj["cpu_num"].is_string());

            let percent = cpu_obj.get("percent").and_then(|v| v.as_f64()).expect("`percent` is not a float");
            assert!(percent >= 0.0 && percent <= 100.0);

            let frequency = cpu_obj.get("frequency").and_then(|v| v.as_u64()).expect("`frequency` is not an integer");
            assert!(frequency > 0);
        }
    }

    async fn cpus_test_server(addr: SocketAddr) {
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
