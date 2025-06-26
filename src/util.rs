use crate::consts;
use enum_map::Enum;
use ratatui::layout::{Flex, Layout, Position, Positions, Rect, Size};
use std::path::PathBuf;
use thiserror::Error;

/// Values that would be global state if it weren't so evil.
///
/// Each screen instance stores a `Globals` instance, and it passes it to the
/// next screen when transitioning.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct Globals {
    /// Program configuration
    pub(crate) config: crate::config::Config,

    /// Gameplay options
    pub(crate) options: crate::options::Options,

    /// High score records
    pub(crate) high_scores: crate::highscores::HighScores,
}

/// The bounds of a game level: size and wraparound
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Bounds {
    /// The width of the level in cells
    pub(crate) width: u16,

    /// The height of the level in cells
    pub(crate) height: u16,

    /// If `true`, the level's boundaries wrap around toroidally.
    pub(crate) wrap: bool,
}

impl Bounds {
    /// Create a new `Bounds` with the given size and wraparound
    pub(crate) fn new(size: Size, wrap: bool) -> Bounds {
        Bounds {
            width: size.width,
            height: size.height,
            wrap,
        }
    }

    /// Retrieve the size of the level
    pub(crate) fn size(self) -> Size {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    /// Returns an iterator over all [`Position`]s in the level
    pub(crate) fn positions(self) -> Positions {
        Rect::from((Position::ORIGIN, self.size())).positions()
    }
}

/// An extension trait on [`enum_map::Enum`]
pub(crate) trait EnumExt: Enum {
    /// Returns an iterator over all values of the enum
    fn iter() -> EnumExtIter<Self>;

    /// Returns the next value of the enum, or `None` if this is the maximum
    /// value
    fn next(self) -> Option<Self>;

    /// Returns the previous value of the enum, or `None` if this is the
    /// minimum value
    fn prev(self) -> Option<Self>;

    /// Returns the minimum value of the enum
    fn min() -> Self;

    /// Returns the maximum value of the enum
    fn max() -> Self;
}

impl<T: Enum> EnumExt for T {
    fn iter() -> EnumExtIter<T> {
        EnumExtIter::new()
    }

    fn next(self) -> Option<Self> {
        self.into_usize()
            .checked_add(1)
            .filter(|&j| j < Self::LENGTH)
            .map(Self::from_usize)
    }

    fn prev(self) -> Option<Self> {
        self.into_usize().checked_sub(1).map(Self::from_usize)
    }

    fn min() -> Self {
        Self::from_usize(0)
    }

    fn max() -> Self {
        Self::from_usize(Self::LENGTH - 1)
    }
}

/// An iterator over all the values of an enum implementing
/// [`enum_map::Enum`]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EnumExtIter<T> {
    inner: std::ops::Range<usize>,
    _marker: std::marker::PhantomData<T>,
}

impl<T: Enum> EnumExtIter<T> {
    fn new() -> Self {
        EnumExtIter {
            inner: 0..T::LENGTH,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: Enum> Iterator for EnumExtIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.inner.next().map(T::from_usize)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<T: Enum> DoubleEndedIterator for EnumExtIter<T> {
    fn next_back(&mut self) -> Option<T> {
        self.inner.next_back().map(T::from_usize)
    }
}

impl<T: Enum> ExactSizeIterator for EnumExtIter<T> {}

impl<T: Enum> std::iter::FusedIterator for EnumExtIter<T> {}

/// Error returned by [`Options::save()`][crate::options::Options::save] and
/// [`HighScores::save()`][crate::highscores::HighScores::save]
#[derive(Debug, Error)]
#[error("Failed to save {desc} to disk")]
pub(crate) struct SaveError {
    /// A description of what we were trying to save ("options" or "high
    /// scores")
    desc: &'static str,

    /// The actual error
    source: SaveErrorSource,
}

impl SaveError {
    pub(crate) fn no_path(desc: &'static str) -> Self {
        SaveError {
            desc,
            source: SaveErrorSource::NoPath,
        }
    }

    pub(crate) fn mkdir(desc: &'static str, e: std::io::Error) -> Self {
        SaveError {
            desc,
            source: SaveErrorSource::Mkdir(e),
        }
    }

    pub(crate) fn serialize(desc: &'static str, e: serde_json::Error) -> Self {
        SaveError {
            desc,
            source: SaveErrorSource::Serialize(e),
        }
    }

    pub(crate) fn write(desc: &'static str, e: std::io::Error) -> Self {
        SaveError {
            desc,
            source: SaveErrorSource::Write(e),
        }
    }
}

/// Source error of [`SaveError`].
///
/// Implementing this as separate type allows for error displays like the
/// following, with a general message at the top level and a source message
/// describing which part of the operation failed:
///
/// ```text
/// Failed to save options to disk
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
    #[error("failed to serialize value")]
    Serialize(#[source] serde_json::Error),
    #[error("failed to write data to disk")]
    Write(#[source] std::io::Error),
}

/// Error returned by [`Options::load()`][crate::options::Options::load] and
/// [`HighScores::load()`][crate::highscores::HighScores::load]
#[derive(Debug, Error)]
#[error("Failed to load {desc} from disk")]
pub(crate) struct LoadError {
    /// A description of what we were trying to save ("options" or "high
    /// scores")
    desc: &'static str,

    /// The actual error
    source: LoadErrorSource,
}

impl LoadError {
    pub(crate) fn no_path(desc: &'static str) -> Self {
        LoadError {
            desc,
            source: LoadErrorSource::NoPath,
        }
    }

    pub(crate) fn read(desc: &'static str, e: std::io::Error) -> Self {
        LoadError {
            desc,
            source: LoadErrorSource::Read(e),
        }
    }

    pub(crate) fn deserialize(desc: &'static str, e: serde_json::Error) -> Self {
        LoadError {
            desc,
            source: LoadErrorSource::Deserialize(e),
        }
    }
}

/// Source error of [`LoadError`].
///
/// Implementing this as separate type allows for error displays like the
/// following, with a general message at the top level and a source message
/// describing which part of the operation failed:
///
/// ```text
/// Failed to load options from disk
///
/// Caused by:
///     0: failed to read file
///     1: permission denied
/// ```
#[derive(Debug, Error)]
enum LoadErrorSource {
    #[error("failed to determine path to local data directory")]
    NoPath,
    #[error("failed to read file")]
    Read(#[source] std::io::Error),
    #[error("failed to deserialize file contents")]
    Deserialize(#[source] serde_json::Error),
}

/// Produce a [`Rect`] of the given size that is centered both vertically &
/// horizontally within `area`
pub(crate) fn center_rect(area: Rect, size: Size) -> Rect {
    let [inner] = Layout::horizontal([size.width])
        .flex(Flex::Center)
        .areas(area);
    let [inner] = Layout::vertical([size.height])
        .flex(Flex::Center)
        .areas(inner);
    inner
}

/// Calculate a [`Rect`] of size [`DISPLAY_SIZE`][consts::DISPLAY_SIZE]
/// centered vertically & horizontally within `buffer_area`.
///
/// This function is intended to be used to calculate the `Rect` in which
/// everything should be drawn, given the area of the terminal window.
pub(crate) fn get_display_area(buffer_area: Rect) -> Rect {
    center_rect(buffer_area, consts::DISPLAY_SIZE)
}

/// Return the path to the directory in which `ratsnake` should store data
/// files.  Returns `None` if no appropriate directory path is defined for this
/// OS.
pub(crate) fn data_dir() -> Option<PathBuf> {
    dirs::data_local_dir().map(|p| p.join("ratsnake"))
}

/// Return the path to the file in which high scores should be stored
pub(crate) fn high_scores_file_path() -> Option<PathBuf> {
    // Use a directory within `data_dir()` in anticipation of eventually having
    // to store level high scores next to the "main" high scores
    data_dir().map(|p| p.join("highscores").join("arcade.json"))
}
