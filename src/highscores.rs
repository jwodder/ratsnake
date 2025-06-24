use crate::options::Options;
use crate::util::high_scores_file_path;
use serde::{de::Deserializer, ser::Serializer, Deserialize, Serialize};
use std::collections::HashMap;
use std::num::NonZeroU32;
use thiserror::Error;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct HighScores(HashMap<Options, NonZeroU32>);

impl HighScores {
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

    pub(crate) fn load() -> Result<HighScores, LoadError> {
        let path = high_scores_file_path().ok_or_else(LoadError::no_path)?;
        let src = match fs_err::read(&path) {
            Ok(src) => src,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(HighScores::default()),
            Err(e) => return Err(LoadError::read(e)),
        };
        serde_json::from_slice(&src).map_err(LoadError::deserialize)
    }

    pub(crate) fn get(&self, opts: Options) -> Option<NonZeroU32> {
        self.0.get(&opts).copied()
    }

    pub(crate) fn set(&mut self, opts: Options, score: NonZeroU32) {
        self.0.insert(opts, score);
    }

    fn to_json_array(&self) -> Vec<HighScoreEntry> {
        self.0
            .iter()
            .map(|(&options, &score)| HighScoreEntry { options, score })
            .collect()
    }

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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct HighScoreEntry {
    options: Options,
    score: NonZeroU32,
}

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

#[derive(Debug, Error)]
#[error("Failed to read high scores from disk")]
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

#[derive(Debug, Error)]
enum LoadErrorSource {
    #[error("failed to determine path to local data directory")]
    NoPath,
    #[error("failed to read high scores file")]
    Read(#[source] std::io::Error),
    #[error("failed to deserialize high scores")]
    Deserialize(#[source] serde_json::Error),
}
