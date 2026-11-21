//! Server configuration: one TOML file replaces all start-up settings.

use serde::Deserialize;
use std::{
    fmt, fs,
    path::{Path, PathBuf},
};

/// File looked up in the working directory when no path is given.
pub const DEFAULT_CONFIG_FILE: &str = "veltrunox.toml";

/// Settings of the server; every key is optional and has a default.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    /// Address and port the server listens on.
    pub address: String,
    /// Directory for the database.
    pub data_dir: PathBuf,
    /// Directory whose audio files are imported as recordings at start-up.
    pub media_dir: Option<PathBuf>,
    /// Audio output device by name; the default device if unset.
    pub audio_device: Option<String>,
    /// Browser origin allowed to call the API (for example a Flutter dev server).
    pub allowed_origin: Option<String>,
    /// Directory with a built web app; replaces the one baked into the binary.
    pub web_dir: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            address: "127.0.0.1:8080".to_owned(),
            data_dir: PathBuf::from("data"),
            media_dir: None,
            audio_device: None,
            allowed_origin: None,
            web_dir: None,
        }
    }
}

#[derive(Debug)]
pub enum ConfigError {
    Read(PathBuf, std::io::Error),
    Parse(PathBuf, toml::de::Error),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read(path, error) => write!(formatter, "cannot read {}: {error}", path.display()),
            Self::Parse(path, error) => write!(formatter, "invalid {}: {error}", path.display()),
        }
    }
}

impl std::error::Error for ConfigError {}

impl Config {
    /// Parses TOML text; unknown keys are rejected so typos do not go unnoticed.
    pub fn parse(text: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(text)
    }

    /// Loads the named file. Without a name, `veltrunox.toml` in the working directory is
    /// used if it exists; otherwise the defaults apply. A named file must exist.
    /// Relative paths inside the file are relative to the working directory.
    pub fn load(explicit_path: Option<&Path>) -> Result<Self, ConfigError> {
        let (path, required) = match explicit_path {
            Some(path) => (path, true),
            None => (Path::new(DEFAULT_CONFIG_FILE), false),
        };
        match fs::read_to_string(path) {
            Ok(text) => Self::parse(&text).map_err(|error| ConfigError::Parse(path.into(), error)),
            Err(error) if !required && error.kind() == std::io::ErrorKind::NotFound => {
                Ok(Self::default())
            }
            Err(error) => Err(ConfigError::Read(path.into(), error)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_file_gives_defaults() {
        assert_eq!(Config::parse("").unwrap(), Config::default());
    }

    #[test]
    fn keys_override_defaults() {
        let config =
            Config::parse("address = \"0.0.0.0:9000\"\naudio_device = \"alsa:x\"").unwrap();
        assert_eq!(config.address, "0.0.0.0:9000");
        assert_eq!(config.audio_device.as_deref(), Some("alsa:x"));
        assert_eq!(config.data_dir, PathBuf::from("data"));
    }

    #[test]
    fn unknown_key_is_rejected() {
        assert!(Config::parse("adress = \"x\"").is_err());
    }

    #[test]
    fn named_file_must_exist() {
        let directory = tempfile::tempdir().unwrap();
        let missing = directory.path().join("missing.toml");
        assert!(matches!(
            Config::load(Some(&missing)),
            Err(ConfigError::Read(..))
        ));
    }

    #[test]
    fn named_file_is_loaded() {
        let directory = tempfile::tempdir().unwrap();
        let file = directory.path().join("server.toml");
        fs::write(&file, "data_dir = \"/var/lib/veltrunox\"").unwrap();
        let config = Config::load(Some(&file)).unwrap();
        assert_eq!(config.data_dir, PathBuf::from("/var/lib/veltrunox"));
    }
}
