//! The server inside the desktop app. In a browser there is none, and every
//! function says so; the interface does not call them there.
//!
//! `async` for the same reason as in `api`.

use crate::host;

/// Where the app's own interface reaches the server inside it.
pub struct LocalHost {
    pub url: String,
    /// Signs this app in as the owner of the machine; works at `url` only.
    pub owner_token: String,
}

/// What the settings ask of the network endpoint.
pub struct SharingChoice {
    pub enabled: bool,
    pub port: u16,
    pub https: bool,
}

/// Why the network endpoint is not on.
pub enum SharingProblem {
    NoAccount,
    PortInUse,
    Failed { detail: String },
}

pub struct SharingInfo {
    pub choice: SharingChoice,
    /// Where browsers reach the server; empty while it is off.
    pub urls: Vec<String>,
    /// Why it is off although it was left on.
    pub problem: Option<SharingProblem>,
    pub has_web_app: bool,
}

/// Starts the server inside the app, or tells where it already runs.
pub async fn start_local_host() -> Result<LocalHost, String> {
    host::start()
}

pub async fn sharing_info() -> Result<SharingInfo, SharingProblem> {
    host::info()
}

/// Switches the network endpoint on or off and remembers the choice.
pub async fn apply_sharing(choice: SharingChoice) -> Result<SharingInfo, SharingProblem> {
    host::apply(choice)
}
