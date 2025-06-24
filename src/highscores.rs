use crate::options::Options;
use crate::util::high_scores_file_path;
use serde::{de::Deserializer, ser::Serializer, Deserialize, Serialize};
use std::collections::HashMap;
use std::num::NonZeroU32;
use thiserror::Error;

/// A collection of the highest score achieved for various `Options` values
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct HighScores(HashMap<Options, NonZeroU32>);

impl HighScores {
    /// Save the high scores to a file on disk
    ///
    /// # Errors
    ///
    /// Returns `Err` if the data directory path could not be determined, if
    /// creating the file's parent directories failed, if serializing the high
    /// scores failed, or if writing the serialized high scores failed.
    pub(crate) fn save(&self) -> Result<(), SaveError> {
        let path = high_scores_file_path().ok_or_else(SaveError::no_path)?;
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            fs_err::create_dir_all(parent).map_err(SaveError::mkdir)?;
        }
        let mut src = serde_json::to_string(self).map_err(SaveError::serialize)?;
        src.push('\n');
        fs_err::write(&path, &src).map_err(SaveError::write)?;
        Ok(())
    }

    /// Read high scores from a file on disk.  If the file does not exist, an
    /// empty `HighScores` value is returned.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the data directory path could not be determined, if
    /// the file could not be read, or if the file's contents could not be
    /// deserialized.
    pub(crate) fn load() -> Result<HighScores, LoadError> {
        let path = high_scores_file_path().ok_or_else(LoadError::no_path)?;
        let src = match fs_err::read(&path) {
            Ok(src) => src,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(HighScores::default()),
            Err(e) => return Err(LoadError::read(e)),
        };
        serde_json::from_slice(&src).map_err(LoadError::deserialize)
    }

    /// Return the high score, if any, for the given [`Options`]
    pub(crate) fn get(&self, opts: Options) -> Option<NonZeroU32> {
        self.0.get(&opts).copied()
    }

    /// Set the high score for `opts` to `score`.  No attempt is made to verify
    /// that `score` is higher than the current high score.
    pub(crate) fn set(&mut self, opts: Options, score: NonZeroU32) {
        self.0.insert(opts, score);
    }

    /// Convert the high scores to a list of `{"options": ..., "score": ...}`
    /// objects that can then be serialized as JSON
    fn to_json_array(&self) -> Vec<HighScoreEntry> {
        self.0
            .iter()
            .map(|(&options, &score)| HighScoreEntry { options, score })
            .collect()
    }

    /// Convert a list of `{"options": ..., "score": ...}` objects to a
    /// `HighScores` instance
    fn from_json_array(array: Vec<HighScoreEntry>) -> HighScores {
        HighScores(
            array
                .into_iter()
                .map(|hse| (hse.options, hse.score))
                .collect(),
        )
    }
}

impl Serialize for HighScores {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_json_array().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for HighScores {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Vec::<HighScoreEntry>::deserialize(deserializer).map(HighScores::from_json_array)
    }
}

/// An intermediate type used for serializing & deserializing `HighScores` as
/// JSON
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct HighScoreEntry {
    options: Options,
    score: NonZeroU32,
}

/// Error returned by [`HighScores::save()`]
#[derive(Debug, Error)]
#[error("Failed to save high scores to disk")]
pub(crate) struct SaveError(#[source] SaveErrorSource);

impl SaveError {
    fn no_path() -> Self {
        SaveError(SaveErrorSource::NoPath)
    }

    fn mkdir(e: std::io::Error) -> Self {
        SaveError(SaveErrorSource::Mkdir(e))
    }

    fn serialize(e: serde_json::Error) -> Self {
        SaveError(SaveErrorSource::Serialize(e))
    }

    fn write(e: std::io::Error) -> Self {
        SaveError(SaveErrorSource::Write(e))
    }
}

/// Source error of [`SaveError`].
///
/// Implementing this as separate type allows for error displays like the
/// following, with a general message at the top level and a source message
/// describing which part of the operation failed:
///
/// ```text
/// Failed to save high scores to disk
///
/// Caused by:
///     0: failed to create parent directories
///     1: permission denied
/// ```
#[derive(Debug, Error)]
enum SaveErrorSource {
    #[error("failed to determine path to local data directory")]
    NoPath,
    #[error("failed to create parent directories")]
    Mkdir(#[source] std::io::Error),
    #[error("failed to serialize high scores")]
    Serialize(#[source] serde_json::Error),
    #[error("failed to write high scores to disk")]
    Write(#[source] std::io::Error),
}

/// Error returned by [`HighScores::load()`]
#[derive(Debug, Error)]
#[error("Failed to load high scores from disk")]
pub(crate) struct LoadError(#[source] LoadErrorSource);

impl LoadError {
    fn no_path() -> Self {
        LoadError(LoadErrorSource::NoPath)
    }

    fn read(e: std::io::Error) -> Self {
        LoadError(LoadErrorSource::Read(e))
    }

    fn deserialize(e: serde_json::Error) -> Self {
        LoadError(LoadErrorSource::Deserialize(e))
    }
}

/// Source error of [`LoadError`].
///
/// Implementing this as separate type allows for error displays like the
/// following, with a general message at the top level and a source message
/// describing which part of the operation failed:
///
/// ```text
/// Failed to load high scores from disk
///
/// Caused by:
///     0: failed to read high scores file
///     1: permission denied
/// ```
#[derive(Debug, Error)]
enum LoadErrorSource {
    #[error("failed to determine path to local data directory")]
    NoPath,
    #[error("failed to read high scores file")]
    Read(#[source] std::io::Error),
    #[error("failed to deserialize high scores")]
    Deserialize(#[source] serde_json::Error),
}
