//! Wires the adapters to one running core: library, playback, analysis, accounts.

use crate::config::Config;
use std::{error::Error, sync::Arc};
use veltrunox_core::RecordingTitle;
use veltrunox_server_adapters::{
    AppState, Argon2Hasher, AudioPlayback, RandomTokens, SqliteStore, SystemClock, SystemWallClock,
    UploadStore, import_directory, system_volume, watch_end,
};
use veltrunox_server_app::{AccountService, ApiTokenService, RecordingRepository};

const DATABASE_FILE: &str = "veltrunox.db";

/// What an empty library starts with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Seed {
    /// Two recordings without audio, so a fresh server shows something.
    Samples,
    Nothing,
}

/// Opens the database in the data directory, creating both as needed.
pub fn open_store(config: &Config) -> Result<Arc<SqliteStore>, Box<dyn Error>> {
    std::fs::create_dir_all(&config.data_dir)?;
    Ok(Arc::new(SqliteStore::open(
        &config.data_dir.join(DATABASE_FILE),
    )?))
}

/// Starts playback and analysis. Needs a Tokio runtime to be entered; the
/// background tasks live as long as that runtime.
pub fn start_core(config: &Config, seed: Seed) -> Result<AppState, Box<dyn Error>> {
    let store = open_store(config)?;
    if let Some(directory) = &config.media_dir {
        let added = import_directory(directory, store.as_ref())?;
        println!("imported {added} new audio file(s)");
    }
    if seed == Seed::Samples {
        add_sample_recordings_if_empty(store.as_ref())?;
    }

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
    let api_tokens = Arc::new(ApiTokenService::new(
        store.clone(),
        Arc::new(RandomTokens),
        Arc::new(SystemWallClock),
    ));
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
    .with_api_tokens(api_tokens)
    .with_uploads(UploadStore::open(
        &config.upload_directory(),
        config.max_upload_bytes,
        store_for_uploads.as_ref(),
    )?);
    watch_end(&app);
    queue_pending_analyses(&app);
    Ok(app)
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
