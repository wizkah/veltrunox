//! Server entry point: wires adapters and starts the HTTP endpoint.

use axum::{Router, http::HeaderValue};
use std::{error::Error, path::PathBuf, sync::Arc};
use tower_http::{cors::CorsLayer, services::ServeDir};
use veltrunox_core::RecordingTitle;
use veltrunox_server_adapters::{
    AppState, AudioPlayback, SqliteStore, SystemClock, import_directory, router, watch_end,
};
use veltrunox_server_app::RecordingRepository;

/// Origin allowed to call the API from a browser (for example the Flutter dev server).
const ALLOWED_ORIGIN_VARIABLE: &str = "VELTRUNOX_ALLOWED_ORIGIN";
/// Directory whose audio files become recordings (until upload exists).
const MEDIA_DIRECTORY_VARIABLE: &str = "VELTRUNOX_MEDIA_DIR";
/// Directory with the built web app (`app/build/web`); served by the server itself if set.
const WEB_DIRECTORY_VARIABLE: &str = "VELTRUNOX_WEB_DIR";
/// Output device by name; the default device if unset.
const AUDIO_DEVICE_VARIABLE: &str = "VELTRUNOX_AUDIO_DEVICE";
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
    if let Ok(directory) = std::env::var(MEDIA_DIRECTORY_VARIABLE) {
        let added = import_directory(&PathBuf::from(directory), store.as_ref())?;
        println!("imported {added} new audio file(s)");
    }
    add_sample_recordings_if_empty(store.as_ref())?;

    let clock = Arc::new(SystemClock::new());
    let catalog = store.clone();
    let audio_clock = clock.clone();
    let app = AppState::with_audio(store, clock, |hooks| {
        Arc::new(AudioPlayback::start(
            std::env::var(AUDIO_DEVICE_VARIABLE).ok(),
            catalog,
            audio_clock,
            hooks,
        ))
    });
    watch_end(app.playback.clone());
    let listener = tokio::net::TcpListener::bind(&address).await?;
    println!("listening on http://{address}");
    axum::serve(listener, with_web_app(with_cors(router(app)))).await?;
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

/// Serves the web app under `/` when a directory is configured; the API paths keep priority.
fn with_web_app(router: Router) -> Router {
    match std::env::var(WEB_DIRECTORY_VARIABLE) {
        Ok(directory) => router.fallback_service(ServeDir::new(directory)),
        Err(_) => router,
    }
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
