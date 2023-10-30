// Jackson Coxson

use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use log::{error, info};

static LOADING: &str = include_str!("loading.html");

pub async fn serve(timeout: u8) -> u16 {
    let timeout: Arc<u16> = Arc::new(timeout as u16 * 1000);
    let make_service = make_service_fn(move |_conn| {
        let timeout = Arc::clone(&timeout);
        async move { Ok::<_, Infallible>(service_fn(move |req| handle(req, timeout.clone()))) }
    });

    let mut port = 3000;
    let server;
    loop {
        // Construct our SocketAddr to listen on...
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        server = match Server::try_bind(&addr) {
            Ok(s) => s
                .http2_keep_alive_interval(None)
                .tcp_keepalive(None)
                .http1_keepalive(false)
                .serve(make_service),
            Err(_) => {
                port += 1;
                continue;
            }
        };
        break;
    }

    tokio::spawn(async {
        if let Err(e) = server.await {
            error!("Fallback HTTP server error: {}", e);
        }
    });

    port
}

async fn handle(_req: Request<Body>, timeout: Arc<u16>) -> Result<Response<Body>, Infallible> {
    info!("Sending loading HTML");
    let loading = LOADING.replace("MUKDUK_FILL_ME", timeout.to_string().as_str());
    Ok(Response::new(Body::from(loading)))
}
