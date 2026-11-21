//! Server entry point: wires adapters and starts the HTTP endpoint.

use axum::{Router, http::HeaderValue};
use std::{error::Error, path::PathBuf, sync::Arc};
use tower_http::cors::CorsLayer;
use veltrunox_core::RecordingTitle;
use veltrunox_server_adapters::{AppState, SqliteStore, SystemClock, router};
use veltrunox_server_app::RecordingRepository;

/// Origin allowed to call the API from a browser (for example the Flutter dev server).
const ALLOWED_ORIGIN_VARIABLE: &str = "VELTRUNOX_ALLOWED_ORIGIN";
const DEFAULT_ADDRESS: &str = "127.0.0.1:8080";
const DEFAULT_DATA_DIRECTORY: &str = "data";
const DATABASE_FILE: &str = "veltrunox.db";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let address = std::env::var("VELTRUNOX_ADDRESS").unwrap_or_else(|_| DEFAULT_ADDRESS.to_owned());
    let data_directory = PathBuf::from(
        std::env::var("VELTRUNOX_DATA_DIR").unwrap_or_else(|_| DEFAULT_DATA_DIRECTORY.to_owned()),
    );
    std::fs::create_dir_all(&data_directory)?;
    let store = Arc::new(SqliteStore::open(&data_directory.join(DATABASE_FILE))?);
    add_sample_recordings_if_empty(store.as_ref())?;

    let app = AppState::new(store, Arc::new(SystemClock::new()));
    let listener = tokio::net::TcpListener::bind(&address).await?;
    println!("listening on http://{address}");
    axum::serve(listener, with_cors(router(app))).await?;
    Ok(())
}

/// Placeholder until upload exists: an empty library would leave nothing to play.
fn add_sample_recordings_if_empty(store: &dyn RecordingRepository) -> Result<(), Box<dyn Error>> {
    if !store.list()?.is_empty() {
        return Ok(());
    }
    for title in ["Soundcheck", "Verse and chorus run-through"] {
        store.add(RecordingTitle::new(title)?)?;
    }
    Ok(())
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
