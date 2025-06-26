use crate::options::Options;
use crate::util::{LoadError, SaveError};
use serde::Deserialize;
use std::borrow::Cow;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Program configuration read from a configuration file
#[derive(Clone, Deserialize, Debug, Default, Eq, PartialEq)]
pub(crate) struct Config {
    /// Default options when no options file is present
    #[serde(default)]
    pub(crate) options: Options,

    /// Settings about data files
    #[serde(default)]
    pub(crate) files: FileConfig,
}

impl Config {
    /// Return the default configuration file path
    pub(crate) fn default_path() -> Result<PathBuf, ConfigError> {
        dirs::config_local_dir()
            .map(|p| p.join("ratsnake").join("config.toml"))
            .ok_or(ConfigError::NoPath)
    }

    /// Read configuration from a file on disk.  If the file does not exist and
    /// `allow_missing` is true, a default `Config` value is returned.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the file could not be read or if the file's contents
    /// could not be deserialized.
    pub(crate) fn load(path: &Path, allow_missing: bool) -> Result<Config, ConfigError> {
        let content = match fs_err::read_to_string(path) {
            Ok(content) => content,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound && allow_missing => {
                return Ok(Config::default())
            }
            Err(e) => return Err(ConfigError::Read(e)),
        };
        toml::from_str(&content).map_err(Into::into)
    }

    /// Return the filepath at which gameplay options should be stored: the
    /// file given in the configuration or, if that is not set, the default
    /// options file path.  Return `None` if no path is present in the
    /// configuration and the default path could not be computed.
    fn options_file(&self) -> Option<Cow<'_, Path>> {
        self.files
            .options_file
            .as_deref()
            .map(Cow::from)
            .or_else(|| Options::default_path().map(Cow::from))
    }

    /// Load gameplay options from a file.  If the file does not exist, `self.options`
    /// is returned.
    ///
    /// If `self.files.save_options` is `false`, `self.options` is returned
    /// without reading anything from disk.
    pub(crate) fn load_options(&self) -> Result<Options, LoadError> {
        let r = if !self.files.save_options {
            Ok(None)
        } else if let Some(p) = self.options_file() {
            Options::load(&p)
        } else {
            Err(LoadError::no_path("options"))
        };
        match r {
            Ok(Some(opts)) => Ok(opts),
            Ok(None) => Ok(self.options),
            Err(e) => Err(e),
        }
    }

    /// Save the given gameplay options to a file.
    ///
    /// If `self.files.save_options` is `false`, nothing is saved.
    pub(crate) fn save_options(&self, options: Options) -> Result<(), SaveError> {
        if !self.files.save_options {
            return Ok(());
        }
        if let Some(p) = self.options_file() {
            options.save(&p)
        } else {
            Err(SaveError::no_path("options"))
        }
    }
}

#[derive(Clone, Deserialize, Debug, Eq, PartialEq)]
#[serde(try_from = "RawFileConfig")]
pub(crate) struct FileConfig {
    /// Path at which gameplay options should be stored
    options_file: Option<PathBuf>,

    /// Whether to load & save gameplay options in a file
    save_options: bool,
}

impl Default for FileConfig {
    fn default() -> FileConfig {
        FileConfig {
            options_file: None,
            save_options: true,
        }
    }
}

#[derive(Clone, Deserialize, Debug, Eq, PartialEq)]
#[serde(default, rename_all = "kebab-case")]
struct RawFileConfig {
    options_file: Option<String>,
    save_options: bool,
}

impl Default for RawFileConfig {
    fn default() -> RawFileConfig {
        RawFileConfig {
            options_file: None,
            save_options: true,
        }
    }
}

impl TryFrom<RawFileConfig> for FileConfig {
    type Error = std::io::Error;

    fn try_from(value: RawFileConfig) -> Result<FileConfig, std::io::Error> {
        Ok(FileConfig {
            options_file: value.options_file.map(expanduser::expanduser).transpose()?,
            save_options: value.save_options,
        })
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
