use crate::highscores::HighScores;
use crate::options::Options;
use crate::util::{expanduser, LoadError, NoHomeError, SaveError};
use serde::{de::Deserializer, Deserialize};
use std::borrow::Cow;
use std::fmt;
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
    /// options file path.  Return `None` if saving & loading of gameplay
    /// options is disabled.
    fn options_file(&self) -> Result<Option<Cow<'_, Path>>, NoHomeError> {
        match self.files.options_file {
            OptionsFile::Path(ref path) => Ok(Some(Cow::from(path))),
            OptionsFile::Default => match Options::default_path() {
                Some(path) => Ok(Some(Cow::from(path))),
                None => Err(NoHomeError),
            },
            OptionsFile::Off => Ok(None),
        }
    }

    /// Load gameplay options from a file, if enabled.  If the file does not
    /// exist, `self.options` is returned.
    pub(crate) fn load_options(&self) -> Result<Options, LoadError> {
        let r = match self.options_file() {
            Ok(Some(p)) => Options::load(&p),
            Ok(None) => Ok(None),
            Err(_) => Err(LoadError::no_path("options")),
        };
        match r {
            Ok(Some(opts)) => Ok(opts),
            Ok(None) => Ok(self.options),
            Err(e) => Err(e),
        }
    }

    /// Save the given gameplay options to a file, if enabled.
    pub(crate) fn save_options(&self, options: Options) -> Result<(), SaveError> {
        match self.options_file() {
            Ok(Some(p)) => options.save(&p),
            Ok(None) => Ok(()),
            Err(_) => Err(SaveError::no_path("options")),
        }
    }

    /// Return the filepath at which high scores should be stored: a file
    /// inside the directory file given in the configuration or, if that is not
    /// set, the default high scores file path.
    fn high_scores_file(&self) -> Result<PathBuf, NoHomeError> {
        match self.files.high_scores_dir {
            Some(ref path) => Ok(path.join(HighScores::ARCADE_FILE_NAME)),
            None => HighScores::default_path().ok_or(NoHomeError),
        }
    }

    /// Load high scores from a file.  If the file does not exist, an empty
    /// `HighScores` value is returned.
    pub(crate) fn load_high_scores(&self) -> Result<HighScores, LoadError> {
        match self.high_scores_file() {
            Ok(p) => HighScores::load(&p),
            Err(_) => Err(LoadError::no_path("high scores")),
        }
    }

    /// Save the given high scores to a file
    pub(crate) fn save_high_scores(&self, scores: &HighScores) -> Result<(), SaveError> {
        match self.high_scores_file() {
            Ok(p) => scores.save(&p),
            Err(_) => Err(SaveError::no_path("high scores")),
        }
    }
}

#[derive(Clone, Deserialize, Debug, Default, Eq, PartialEq)]
#[serde(try_from = "RawFileConfig")]
pub(crate) struct FileConfig {
    /// Path at which gameplay options should be stored
    options_file: OptionsFile<PathBuf>,

    /// Path to the directory in which high scores should be saved
    // This is a directory path in anticipation of eventually also storing
    // level high scores here.
    high_scores_dir: Option<PathBuf>,
}

#[derive(Clone, Deserialize, Debug, Default, Eq, PartialEq)]
#[serde(default, rename_all = "kebab-case")]
struct RawFileConfig {
    options_file: OptionsFile<String>,
    high_scores_dir: Option<String>,
}

impl TryFrom<RawFileConfig> for FileConfig {
    type Error = NoHomeError;

    fn try_from(value: RawFileConfig) -> Result<FileConfig, NoHomeError> {
        Ok(FileConfig {
            options_file: value.options_file.expanduser()?,
            high_scores_dir: value
                .high_scores_dir
                .as_deref()
                .map(expanduser)
                .transpose()?,
        })
    }
}

/// Possible settings for the `files.options-file` configuration
#[derive(Clone, Debug, Default, Eq, PartialEq)]
enum OptionsFile<T> {
    /// Save & load gameplay options to/from the given path
    Path(T),

    /// Save & load gameplay options to/from the default path
    #[default]
    Default,

    /// Do not save or load gameplay options to/from disk
    Off,
}

impl OptionsFile<String> {
    fn expanduser(self) -> Result<OptionsFile<PathBuf>, NoHomeError> {
        match self {
            OptionsFile::Path(p) => Ok(OptionsFile::Path(expanduser(&p)?)),
            OptionsFile::Default => Ok(OptionsFile::Default),
            OptionsFile::Off => Ok(OptionsFile::Off),
        }
    }
}

impl<'de> Deserialize<'de> for OptionsFile<String> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;

        impl serde::de::Visitor<'_> for Visitor {
            type Value = OptionsFile<String>;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a string or boolean")
            }

            fn visit_bool<E>(self, input: bool) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if input {
                    Ok(OptionsFile::Default)
                } else {
                    Ok(OptionsFile::Off)
                }
            }

            fn visit_str<E>(self, input: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(OptionsFile::Path(input.to_owned()))
            }

            fn visit_string<E>(self, input: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(OptionsFile::Path(input))
            }
        }

        deserializer.deserialize_str(Visitor)
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn options_file_path() {
        let tmp = NamedTempFile::new().unwrap();
        std::fs::write(
            tmp.path(),
            concat!(
                "[files]\n",
                "options-file = \"/home/luser/stuff/ratsnake/options.json\"\n"
            ),
        )
        .unwrap();
        let cfg = Config::load(tmp.path(), false).unwrap();
        assert_eq!(
            cfg,
            Config {
                files: FileConfig {
                    options_file: OptionsFile::Path(PathBuf::from(
                        "/home/luser/stuff/ratsnake/options.json"
                    )),
                    ..FileConfig::default()
                },
                ..Config::default()
            }
        );
        assert_eq!(
            cfg.options_file(),
            Ok(Some(Cow::from(PathBuf::from(
                "/home/luser/stuff/ratsnake/options.json"
            ))))
        );
    }

    #[test]
    fn options_file_missing() {
        let tmp = NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), "[files]\n").unwrap();
        let cfg = Config::load(tmp.path(), false).unwrap();
        assert_eq!(
            cfg,
            Config {
                files: FileConfig {
                    options_file: OptionsFile::Default,
                    ..FileConfig::default()
                },
                ..Config::default()
            }
        );
        assert!(cfg.options_file().unwrap().is_some());
    }

    #[test]
    fn options_file_true() {
        let tmp = NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), "[files]\noptions-file = true\n").unwrap();
        let cfg = Config::load(tmp.path(), false).unwrap();
        assert_eq!(
            cfg,
            Config {
                files: FileConfig {
                    options_file: OptionsFile::Default,
                    ..FileConfig::default()
                },
                ..Config::default()
            }
        );
        assert!(cfg.options_file().unwrap().is_some());
    }

    #[test]
    fn options_file_false() {
        let tmp = NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), "[files]\noptions-file = false\n").unwrap();
        let cfg = Config::load(tmp.path(), false).unwrap();
        assert_eq!(
            cfg,
            Config {
                files: FileConfig {
                    options_file: OptionsFile::Off,
                    ..FileConfig::default()
                },
                ..Config::default()
            }
        );
        assert_eq!(cfg.options_file(), Ok(None));
    }
}
