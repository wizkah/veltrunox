//! Server entry point: wires adapters and starts the HTTP endpoint.

mod config;
mod web;

use axum::{Router, http::HeaderValue};
use config::Config;
use std::{error::Error, path::PathBuf, sync::Arc};
use tower_http::{compression::CompressionLayer, cors::CorsLayer};
use veltrunox_core::RecordingTitle;
use veltrunox_server_adapters::{
    AppState, AudioPlayback, SqliteStore, SystemClock, import_directory, router, system_volume,
    watch_end,
};
use veltrunox_server_app::RecordingRepository;
use web::with_web_app;

const DATABASE_FILE: &str = "veltrunox.db";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::load(config_path_argument()?.as_deref()).unwrap_or_else(|error| {
        eprintln!("{error}");
        std::process::exit(2);
    });
    std::fs::create_dir_all(&config.data_dir)?;
    let store = Arc::new(SqliteStore::open(&config.data_dir.join(DATABASE_FILE))?);
    if let Some(directory) = &config.media_dir {
        let added = import_directory(directory, store.as_ref())?;
        println!("imported {added} new audio file(s)");
    }
    add_sample_recordings_if_empty(store.as_ref())?;

    let clock = Arc::new(SystemClock::new());
    let catalog = store.clone();
    let analyses = store.clone();
    let audio_clock = clock.clone();
    let volume = system_volume(config.audio_device.as_deref());
    let app = AppState::with_audio(store, clock, volume, |hooks| {
        Arc::new(AudioPlayback::start(
            config.audio_device.clone(),
            volume,
            catalog,
            analyses,
            audio_clock,
            hooks,
        ))
    });
    watch_end(app.playback.clone());
    queue_pending_analyses(&app);
    let listener = tokio::net::TcpListener::bind(&config.address).await?;
    println!("listening on http://{}", config.address);
    let routes = with_web_app(
        with_cors(router(app), config.allowed_origin.as_deref()),
        config.web_dir.as_deref(),
    );
    axum::serve(listener, routes.layer(CompressionLayer::new())).await?;
    Ok(())
}

/// Analyses what has no result yet; they run one after the other in the background.
fn queue_pending_analyses(app: &AppState) {
    match app.analysis.unanalysed() {
        Ok(pending) => pending
            .into_iter()
            .for_each(|id| app.analysis_queue.enqueue(id)),
        Err(error) => eprintln!("cannot list recordings to analyse: {error}"),
    }
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

/// Reads `--config <file>` from the command line.
fn config_path_argument() -> Result<Option<PathBuf>, Box<dyn Error>> {
    let mut arguments = std::env::args().skip(1);
    match arguments.next().as_deref() {
        None => Ok(None),
        Some("--config") => match (arguments.next(), arguments.next()) {
            (Some(path), None) => Ok(Some(PathBuf::from(path))),
            _ => Err("usage: veltrunox-server [--config <file>]".into()),
        },
        Some(_) => Err("usage: veltrunox-server [--config <file>]".into()),
    }
}

/// Allows exactly one browser origin when configured; otherwise no cross-origin access.
fn with_cors(router: Router, allowed_origin: Option<&str>) -> Router {
    match allowed_origin.and_then(|origin| origin.parse::<HeaderValue>().ok()) {
        Some(origin) => router.layer(CorsLayer::new().allow_origin(origin)),
        None => router,
    }
}
