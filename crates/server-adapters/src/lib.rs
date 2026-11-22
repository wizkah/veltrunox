//! Server adapters: persistence and the HTTP and real-time endpoints.

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
mod sqlite;
mod state;

pub use analysis::{AnalysisError, analyse};
pub use analysis_worker::{AnalysisQueue, FileAnalyzer};
pub use audio::{
    AudioPlayback, DecodeError, Engine, Pcm, decode_file, list_output_devices, system_volume,
};
pub use clock::SystemClock;
pub use end_watch::watch_end;
pub use hooks::AudioHooks;
pub use http::router;
pub use import::{ImportError, import_directory};
pub use sqlite::SqliteStore;
pub use state::AppState;
