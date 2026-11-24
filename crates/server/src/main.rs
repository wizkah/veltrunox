//! Server entry point: wires adapters and starts the HTTP endpoint.

mod cli;
mod config;
mod user_admin;
mod web;

use axum::{Router, http::HeaderValue};
use config::Config;
use std::{error::Error, sync::Arc};
use tower_http::{compression::CompressionLayer, cors::CorsLayer};
use veltrunox_core::RecordingTitle;
use veltrunox_server_adapters::{
    AppState, Argon2Hasher, AudioPlayback, RandomTokens, SqliteStore, SystemClock, SystemWallClock,
    UploadStore, import_directory, router, system_volume, watch_end,
};
use veltrunox_server_app::AccountService;
use veltrunox_server_app::RecordingRepository;
use web::with_web_app;

const DATABASE_FILE: &str = "veltrunox.db";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let invocation = cli::parse(std::env::args().skip(1)).unwrap_or_else(|usage| {
        eprintln!("{usage}");
        std::process::exit(2);
    });
    let config = Config::load(invocation.config.as_deref()).unwrap_or_else(|error| {
        eprintln!("{error}");
        std::process::exit(2);
    });
    std::fs::create_dir_all(&config.data_dir)?;
    let store = Arc::new(SqliteStore::open(&config.data_dir.join(DATABASE_FILE))?);
    if let cli::Command::Users(command) = invocation.command {
        return administer_users(store, &command);
    }
    if let Some(directory) = &config.media_dir {
        let added = import_directory(directory, store.as_ref())?;
        println!("imported {added} new audio file(s)");
    }
    add_sample_recordings_if_empty(store.as_ref())?;

    let clock = Arc::new(SystemClock::new());
    let catalog = store.clone();
    let store_for_uploads = store.clone();
    let analyses = store.clone();
    let accounts = Arc::new(AccountService::new(
        store.clone(),
        store.clone(),
        Arc::new(Argon2Hasher::new()),
        Arc::new(RandomTokens),
        Arc::new(SystemWallClock),
    )?);
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
    })
    .with_accounts(accounts)
    .with_uploads(UploadStore::open(
        &config.upload_directory(),
        config.max_upload_bytes,
        store_for_uploads.as_ref(),
    )?);
    watch_end(&app);
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

/// Runs one user command against the database and ends the process on failure.
fn administer_users(
    store: Arc<SqliteStore>,
    command: &cli::UserCommand,
) -> Result<(), Box<dyn Error>> {
    let accounts = AccountService::new(
        store.clone(),
        store,
        Arc::new(Argon2Hasher::new()),
        Arc::new(RandomTokens),
        Arc::new(SystemWallClock),
    )?;
    let result = user_admin::run(
        &accounts,
        command,
        &mut user_admin::TerminalPasswords,
        &mut std::io::stdout(),
    );
    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
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
fn with_cors(router: Router, allowed_origin: Option<&str>) -> Router {
    match allowed_origin.and_then(|origin| origin.parse::<HeaderValue>().ok()) {
        Some(origin) => router.layer(CorsLayer::new().allow_origin(origin)),
        None => router,
    }
}
