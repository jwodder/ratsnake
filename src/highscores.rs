use crate::options::Options;
use crate::util::{data_dir, LoadError, SaveError};
use serde::{de::Deserializer, ser::Serializer, Deserialize, Serialize};
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};

/// A collection of the highest score achieved for various `Options` values
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct HighScores(HashMap<Options, NonZeroU32>);

impl HighScores {
    /// The name of the file within the high scores directory in which "arcade"
    /// (non-level) high scores are saved.
    pub(crate) const ARCADE_FILE_NAME: &str = "arcade.json";

    /// Return the default filepath used for storing high score options
    pub(crate) fn default_path() -> Option<PathBuf> {
        // Use a directory within `data_dir()` in anticipation of eventually having
        // to store level high scores next to the "arcade" high scores
        data_dir().map(|p| p.join("highscores").join(Self::ARCADE_FILE_NAME))
    }

    /// Save the high scores to a file on disk
    ///
    /// # Errors
    ///
    /// Returns `Err` if creating the file's parent directories failed, if
    /// serializing the high scores failed, or if writing the serialized high
    /// scores failed.
    pub(crate) fn save(&self, path: &Path) -> Result<(), SaveError> {
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            fs_err::create_dir_all(parent).map_err(|e| SaveError::mkdir("high scores", e))?;
        }
        let mut src =
            serde_json::to_string(self).map_err(|e| SaveError::serialize("high scores", e))?;
        src.push('\n');
        fs_err::write(path, &src).map_err(|e| SaveError::write("high scores", e))?;
        Ok(())
    }

    /// Read high scores from a file on disk.  If the file does not exist, an
    /// empty `HighScores` value is returned.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the file could not be read or if the file's contents
    /// could not be deserialized.
    pub(crate) fn load(path: &Path) -> Result<HighScores, LoadError> {
        match fs_err::read(path) {
            Ok(src) => {
                serde_json::from_slice(&src).map_err(|e| LoadError::deserialize("high scores", e))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(HighScores::default()),
            Err(e) => Err(LoadError::read("high scores", e)),
        }
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
