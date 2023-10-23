use std::convert::Infallible;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use hyper::{Body, Request, Response, Server};
use hyper::http::StatusCode;
use hyper::service::{make_service_fn, service_fn};
use serde_json::json;
use sysinfo::{ComponentExt, System, SystemExt};

#[tokio::main]
async fn main() {
    // Default address
    let default_addr = "127.0.0.1:5000";

    // Check command line arguments to override the default address if provided
    let arg_addr = std::env::args().nth(1).unwrap_or_else(|| default_addr.to_string());

    let addr: SocketAddr = match SocketAddr::from_str(&arg_addr) {
        Ok(a) => a,
        Err(_) => {
            eprintln!("Invalid address format. Falling back to default: {}", default_addr);
            SocketAddr::new(IpAddr::V4([127, 0, 0, 1].into()), 5000)
        }
    };

    let system = Arc::new(Mutex::new(System::new_all()));

    let make_service = make_service_fn(move |_| {
        let system = system.clone();
        async {
            Ok::<_, Infallible>(service_fn(move |req| handle_request(req, system.clone())))
        }
    });

    let server = Server::bind(&addr).serve(make_service);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

async fn handle_request(req: Request<Body>, system: Arc<Mutex<System>>) -> Result<Response<Body>, hyper::Error> {
    match req.uri().path() {
        "/cpus" => {
            let mut system = system.lock().unwrap();
            system.refresh_cpu();
            let cpu_info: Vec<_> = system.cpus().iter().enumerate().map(|(i, proc)| {
                json!({
                    "cpu_num": format!("cpu{}", i),
                    "percent": proc.cpu_usage(),
                    "frequency": proc.frequency() as u32
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
        "/disks" => {
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
        "/memory" => {
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
        "/temperatures" => {
            let mut system = system.lock().unwrap();
            system.refresh_all();

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

        _ => {
            let response = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Not Found"))
                .unwrap();
            Ok(response)
        }
    }
}
