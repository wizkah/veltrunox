//! Server entry point: wires adapters and starts the HTTP endpoint.

use axum::{Router, http::HeaderValue};
use std::{error::Error, sync::Arc};
use tower_http::cors::CorsLayer;
use veltrunox_core::{Recording, RecordingId, RecordingTitle};
use veltrunox_server_adapters::{InMemoryRecordingRepository, router};

/// Origin allowed to call the API from a browser (for example the Flutter dev server).
const ALLOWED_ORIGIN_VARIABLE: &str = "VELTRUNOX_ALLOWED_ORIGIN";
const DEFAULT_ADDRESS: &str = "127.0.0.1:8080";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let address = std::env::var("VELTRUNOX_ADDRESS").unwrap_or_else(|_| DEFAULT_ADDRESS.to_owned());
    let repository = Arc::new(InMemoryRecordingRepository::new(sample_recordings()?));
    let listener = tokio::net::TcpListener::bind(&address).await?;
    println!("listening on http://{address}");
    axum::serve(listener, with_cors(router(repository))).await?;
    Ok(())
}

/// Placeholder data until persistence exists.
fn sample_recordings() -> Result<Vec<Recording>, Box<dyn Error>> {
    ["Soundcheck", "Verse and chorus run-through"]
        .iter()
        .zip(1u64..)
        .map(|(title, id)| {
            Ok(Recording {
                id: RecordingId::new(id),
                title: RecordingTitle::new(title)?,
            })
        })
        .collect()
}

/// Allows exactly one browser origin when configured; otherwise no cross-origin access.
fn with_cors(router: Router) -> Router {
    match std::env::var(ALLOWED_ORIGIN_VARIABLE)
        .ok()
        .and_then(|origin| origin.parse::<HeaderValue>().ok())
    {
        Some(origin) => router.layer(CorsLayer::new().allow_origin(origin)),
        None => router,
    }
}
