use hyper::{Server, Response, Body, Request};
use hyper::service::{make_service_fn, service_fn};
use sysinfo::{System, SystemExt, DiskExt, CpuExt};
use std::sync::{Arc, Mutex};
use std::convert::Infallible;
use serde_json::json;

#[tokio::main]
async fn main() {
    let addr = ([127, 0, 0, 1], 3000).into();
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
            Ok(Response::new(Body::from(body.to_string())))
        },
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
            Ok(Response::new(Body::from(json!(disks_info).to_string())))
        },
        _ => Ok(Response::new(Body::from("Not Found"))),
    }
}
