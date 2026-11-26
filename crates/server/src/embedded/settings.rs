//! What the app remembers about serving the network.

use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

const FILE: &str = "sharing.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SharingSettings {
    pub enabled: bool,
    pub port: u16,
    /// HTTPS with the server's own certificate authority.
    pub https: bool,
}

impl Default for SharingSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            port: 8080,
            https: false,
        }
    }
}

pub struct SettingsFile(PathBuf);

impl SettingsFile {
    pub fn in_directory(directory: &Path) -> Self {
        Self(directory.join(FILE))
    }

    /// The defaults, which keep the server off, if nothing readable is stored.
    pub fn read(&self) -> SharingSettings {
        fs::read(&self.0)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    pub fn write(&self, settings: SharingSettings) -> io::Result<()> {
        fs::write(&self.0, serde_json::to_vec_pretty(&settings)?)
    }
}
