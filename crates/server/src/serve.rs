//! Binds an address and serves routes on it, over HTTPS or HTTP.

use crate::tls::Tls;
use axum::{Router, http::HeaderValue};
use std::{
    future::Future,
    io,
    net::{SocketAddr, TcpListener},
};
use tower_http::{compression::CompressionLayer, cors::CorsLayer};

/// Binds at once, so a busy port is reported to the caller, and returns the
/// bound address with the future that serves until it is dropped. HTTPS if a
/// certificate is prepared, else HTTP. The future needs a Tokio runtime.
pub fn listen(
    address: &str,
    routes: Router,
    tls: Option<Tls>,
) -> io::Result<(
    SocketAddr,
    impl Future<Output = io::Result<()>> + Send + use<>,
)> {
    let listener = TcpListener::bind(address)?;
    listener.set_nonblocking(true)?;
    let local = listener.local_addr()?;
    let serving = async move {
        let Some(tls) = tls else {
            let listener = tokio::net::TcpListener::from_std(listener)?;
            return axum::serve(listener, routes.layer(CompressionLayer::new())).await;
        };
        let routes = tls.with_ca_download(routes).layer(CompressionLayer::new());
        let server = axum_server::from_tcp_rustls(listener, tls.config.clone())?;
        tls.keep_current();
        server.serve(routes.into_make_service()).await
    };
    Ok((local, serving))
}

/// Allows exactly one browser origin when configured; otherwise no cross-origin access.
pub fn with_cors(router: Router, allowed_origin: Option<&str>) -> Router {
    match allowed_origin.and_then(|origin| origin.parse::<HeaderValue>().ok()) {
        Some(origin) => router.layer(CorsLayer::new().allow_origin(origin)),
        None => router,
    }
}
