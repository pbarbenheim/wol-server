use std::convert::Infallible;
use std::net::SocketAddr;
use std::time::Duration;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, header, Method, StatusCode};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use wol_server::{send_wol, MacAddr, MacAddrError};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    const PORT: u16 = 4441;

    let addr = SocketAddr::from(([127, 0, 0, 1], PORT));

    let listener = TcpListener::bind(addr).await?;

    println!("Server listening on port {}", PORT);

    loop {
        let (stream, _) = listener.accept().await?;
        
        let io = TokioIo::new(stream);
        
        tokio::task::spawn(async move {
            let service = service_fn(move |req| handle_connection(req));
            if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                println!("Failed to serve connection: {:?}", err);
            }
        });
    }
}

async fn handle_connection(req: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    let addr = &req.uri().path()[1..];

    println!("{}", addr);

    let mac_addr_result: Result<MacAddr, MacAddrError> = addr.parse::<MacAddr>();
    
    match (mac_addr_result, req.method()) {
        (Ok(mac_addr), &Method::POST) => handle_wol(mac_addr).await,
        _ => Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Full::new(Bytes::from("Bad Request")))
            .unwrap()),
    }
}

async fn handle_wol(mac_addr: MacAddr) -> Result<Response<Full<Bytes>>, Infallible> {
    let s = retry(1000, 100, || {send_wol(mac_addr, None, None)}, Some(Duration::from_millis(5)));

    println!("s: {}", s);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/plain")
        .body(Full::new(Bytes::from(format!("Send magic packet to {}", mac_addr))))
        .unwrap())

}

fn retry<F, G, H>(max: usize, min: usize, f: F, delay: Option<Duration>) -> usize where F: Fn() -> Result<G, H> {
    let mut successful: usize = 0;
    for _ in 0..max {
        if successful >= min {
            break;
        }
        match delay {
            Some(d) => std::thread::sleep(d),
            None => (),
        };
        let r = f();
        match r {
            Ok(_) => successful += 1,
            Err(_) => (),
        };
    }
    successful
}
