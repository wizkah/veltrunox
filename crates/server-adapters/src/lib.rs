//! Server adapters: persistence and the HTTP and real-time endpoints.

mod access;
mod accounts_http;
mod analysis;
mod analysis_worker;
mod audio;
mod clock;
mod end_watch;
mod events;
mod hooks;
mod http;
mod hub;
mod import;
mod security;
mod sqlite;
mod state;
mod upload;
mod users;

pub use analysis::{AnalysisError, analyse, analyse_reporting};
pub use analysis_worker::{AnalysisQueue, FileAnalyzer};
pub use audio::{
    AudioPlayback, DecodeError, Engine, Pcm, decode_file, list_output_devices, system_volume,
};
pub use clock::SystemClock;
pub use end_watch::watch_end;
pub use hooks::AudioHooks;
pub use http::router;
pub use import::{ImportError, import_directory};
pub use security::{Argon2Hasher, RandomTokens, SystemWallClock};
pub use sqlite::SqliteStore;
pub use state::AppState;
pub use upload::UploadStore;
