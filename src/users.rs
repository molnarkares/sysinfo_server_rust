use std::sync::{Arc, Mutex};

use hyper::{Body, Response};
use hyper::http::StatusCode;

use serde_json::json;
use sysinfo::{System, SystemExt, UserExt};
pub(crate) async fn handle_users(system: Arc<Mutex<System>>) -> Result<Response<Body>, hyper::Error> {
    let mut system = system.lock().unwrap();
    system.refresh_users_list();
    let users: Vec<_> = system.users().iter().map(|user| {
        json!({
            "name": user.name(),
            "group": user.groups()
        })
    }).collect();


    let response = match Response::builder()
        .header("Content-Type", "application/json")
        .body(Body::from(json!(users).to_string())) {
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

    const TEST_SERVER_ADDR: &str = "127.0.0.1:8089"; // Use a different port for testing

    #[tokio::test]
    async fn test_users_endpoint() {
        // Start the server in a background task
        tokio::spawn(async {
            let addr: SocketAddr = TEST_SERVER_ADDR.parse().expect("Invalid socket address");
            users_test_server(addr).await;
        });

        // Give the server a moment to start
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let response: serde_json::Value = reqwest::get(&format!("http://{}/users", TEST_SERVER_ADDR))
            .await
            .expect("Failed to send request")
            .json()
            .await
            .expect("Failed to parse response as JSON");

        assert!(response.is_array());

        let users = response.as_array().expect("Users data is not an array");

        for user in users {
            assert!(user.is_object());
            let user_obj = user.as_object().unwrap();

            assert!(user_obj.contains_key("name"));
            assert!(user_obj.contains_key("group"));

            let group = user_obj["group"].as_array().expect("Group data is not an array");
            for g in group {
                assert!(g.is_string());
            }
        }
    }

    async fn users_test_server(addr: SocketAddr) {
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
