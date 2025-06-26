use crate::options::Options;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Clone, Deserialize, Debug, Default, Eq, PartialEq)]
pub(crate) struct Config {
    #[serde(default)]
    options: Options,
}

impl Config {
    pub(crate) fn default_path() -> Result<PathBuf, ConfigError> {
        dirs::config_local_dir()
            .map(|p| p.join("ratsnake").join("config.toml"))
            .ok_or(ConfigError::NoPath)
    }

    pub(crate) fn load<P: AsRef<Path>>(path: P) -> Result<Config, ConfigError> {
        let content = fs_err::read_to_string(path)?;
        toml::from_str(&content).map_err(Into::into)
    }
}

#[derive(Debug, Error)]
pub(crate) enum ConfigError {
    #[error("failed to determine path to local configuration directory")]
    NoPath,
    #[error("failed to read configuration file")]
    Read(#[from] std::io::Error),
    #[error("failed to parse configuration file")]
    Parse(#[from] toml::de::Error),
}

impl ConfigError {
    pub(crate) fn is_not_found(&self) -> bool {
        matches!(self, ConfigError::Read(e) if e.kind() == std::io::ErrorKind::NotFound)
    }
}
