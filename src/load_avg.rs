use std::sync::{Arc, Mutex};

use hyper::{Body, Response};
use hyper::http::StatusCode;

use serde_json::json;
use sysinfo::{System, SystemExt};

pub(crate) async fn handle_load_average(system: Arc<Mutex<System>>) -> Result<Response<Body>, hyper::Error> {
    let mut system = system.lock().unwrap();
    system.refresh_system();
        let load_average = system.load_average();

        let result = json!([{
        "one": load_average.one,
        "five": load_average.five,
        "fifteen": load_average.fifteen,
    }]);

    let response = match Response::builder()
        .header("Content-Type", "application/json")
        .body(Body::from(json!(result).to_string())) {
        Ok(res) => res,
        Err(_) => Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Body::empty()).unwrap(),
    };
    Ok(response)
}
