mod cpus;
mod disks;
mod memory;
mod temperatures;
mod hostinfo;
mod users;
mod networks;
mod load_avg;
mod boot_time;

use std::convert::Infallible;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use hyper::{Body, Method, Request, Response, Server};
use hyper::http::StatusCode;
use hyper::service::{make_service_fn, service_fn};
use sysinfo::{System, SystemExt};

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
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/memory") => memory::handle_memory(system).await,
        (&Method::GET, "/temperatures") => temperatures::handle_temperatures(system).await,
        (&Method::GET, "/sysinfo") => hostinfo::handle_system_info(system).await,
        (&Method::GET, "/disks") => disks::handle_disks(system).await,
        (&Method::GET, "/cpus") => cpus::handle_cpus(system).await,
        (&Method::GET, "/users") => users::handle_users(system).await,
        (&Method::GET, "/networks") => networks::handle_networks(system).await,
        (&Method::GET, "/load_average") => load_avg::handle_load_average(system).await,
        (&Method::GET, "/boot_time") => boot_time::handle_boot_time(system).await,
        _ => {
            let response = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Not Found"))
                .unwrap();
            Ok(response)
        }
    }
}
