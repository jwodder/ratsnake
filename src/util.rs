use crate::consts;
use enum_map::Enum;
use ratatui::layout::{Flex, Layout, Position, Positions, Rect, Size};
use std::path::PathBuf;

/// Values that would be global state if it weren't so evil.
///
/// Each screen instance stores a `Globals` instance, and it passes it to the
/// next screen when transitioning.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct Globals {
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
fn data_dir() -> Option<PathBuf> {
    dirs::data_local_dir().map(|p| p.join("ratsnake"))
}

/// Return the path to the file in which gameplay options should be stored
pub(crate) fn options_file_path() -> Option<PathBuf> {
    data_dir().map(|p| p.join("options.json"))
}

/// Return the path to the file in which high scores should be stored
pub(crate) fn high_scores_file_path() -> Option<PathBuf> {
    // Use a directory within `data_dir()` in anticipation of eventually having
    // to store level high scores next to the "main" high scores
    data_dir().map(|p| p.join("highscores").join("arcade.json"))
}
