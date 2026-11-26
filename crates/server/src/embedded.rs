//! The server inside the desktop app. One core runs for as long as the app. An
//! endpoint on the loopback interface always serves the app's own interface; a
//! second endpoint, switched on and off in the settings, serves the network.

mod settings;
mod sharing;

use crate::{Seed, config::Config, serve::listen, start_core};
use settings::SettingsFile;
use sharing::Sharing;
use std::{error::Error, path::PathBuf};
use tokio::runtime::{Builder, Runtime};
use veltrunox_server_adapters::{AppState, RandomTokens, router};
use veltrunox_server_app::SessionTokens;

pub use settings::SharingSettings;

/// Where the app keeps its files.
#[derive(Debug, Clone)]
pub struct Places {
    pub data_dir: PathBuf,
    /// The built web app offered to browsers on the network, if there is one.
    pub web_dir: Option<PathBuf>,
}

/// Why the network endpoint is not on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SharingError {
    /// Without an account whoever connects first would become the admin.
    NoAccount,
    PortInUse,
    Failed(String),
}

/// What the settings show.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharingStatus {
    pub settings: SharingSettings,
    /// Where browsers reach the server; empty while it is off.
    pub urls: Vec<String>,
    /// Why it is off although the settings want it on.
    pub failure: Option<SharingError>,
    pub has_web_app: bool,
}

pub struct Embedded {
    // Dropped last: the endpoints and the core's tasks run on it.
    runtime: Option<Runtime>,
    app: AppState,
    places: Places,
    settings: SettingsFile,
    local_url: String,
    owner_token: String,
    sharing: Option<Sharing>,
    failure: Option<SharingError>,
}

impl Embedded {
    /// Starts the core and the local endpoint, and the network endpoint if the
    /// settings say so.
    pub fn start(places: Places) -> Result<Self, Box<dyn Error>> {
        let runtime = Builder::new_multi_thread().enable_all().build()?;
        let config = Config {
            data_dir: places.data_dir.clone(),
            ..Config::default()
        };
        let guard = runtime.enter();
        let app = start_core(&config, Seed::Nothing)?;
        let owner_token = RandomTokens.issue();
        let local = router(app.clone().with_owner_token(&owner_token));
        let (address, serving) = listen("127.0.0.1:0", local, None)?;
        runtime.spawn(serving);
        drop(guard);
        let settings = SettingsFile::in_directory(&places.data_dir);
        let mut embedded = Self {
            runtime: Some(runtime),
            app,
            places,
            settings,
            local_url: format!("http://{address}"),
            owner_token,
            sharing: None,
            failure: None,
        };
        let wanted = embedded.settings.read();
        if wanted.enabled {
            embedded.failure = embedded.open(wanted).err();
        }
        Ok(embedded)
    }

    pub fn local_url(&self) -> &str {
        &self.local_url
    }

    /// Stands for the owner of this machine at the local endpoint only.
    pub fn owner_token(&self) -> &str {
        &self.owner_token
    }

    pub fn status(&self) -> SharingStatus {
        SharingStatus {
            settings: self.settings.read(),
            urls: self.sharing.as_ref().map(Sharing::urls).unwrap_or_default(),
            failure: self.failure.clone(),
            has_web_app: self.places.web_dir.is_some(),
        }
    }

    /// Switches the network endpoint to what is asked for and remembers it. A
    /// failure leaves it off and is not remembered as wanted.
    pub fn apply(&mut self, wanted: SharingSettings) -> Result<SharingStatus, SharingError> {
        // Closed first: the same port may be asked for again.
        self.sharing = None;
        self.failure = None;
        let opened = if wanted.enabled {
            self.open(wanted)
        } else {
            Ok(())
        };
        let remembered = SharingSettings {
            enabled: wanted.enabled && opened.is_ok(),
            ..wanted
        };
        self.settings
            .write(remembered)
            .map_err(|error| SharingError::Failed(error.to_string()))?;
        opened.map(|()| self.status())
    }

    fn open(&mut self, wanted: SharingSettings) -> Result<(), SharingError> {
        self.require_account()?;
        self.sharing = Some(Sharing::open(&self.app, &self.places, wanted)?);
        Ok(())
    }

    fn require_account(&self) -> Result<(), SharingError> {
        let failed = |error: String| SharingError::Failed(error);
        let accounts = self
            .app
            .accounts
            .as_ref()
            .ok_or_else(|| failed("no accounts".to_owned()))?;
        let status = accounts
            .status()
            .map_err(|error| failed(error.to_string()))?;
        if status.setup_needed {
            return Err(SharingError::NoAccount);
        }
        Ok(())
    }
}

impl Drop for Embedded {
    fn drop(&mut self) {
        self.sharing = None;
        if let Some(runtime) = self.runtime.take() {
            // Does not wait, so it is allowed from within another runtime.
            runtime.shutdown_background();
        }
    }
}

#[cfg(test)]
mod tests;
