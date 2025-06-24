use crate::consts;
use enum_map::Enum;
use ratatui::layout::{Flex, Layout, Rect, Size};
use std::path::PathBuf;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct Globals {
    pub(crate) options: crate::options::Options,
}

pub(crate) trait EnumExt: Enum {
    fn iter() -> EnumExtIter<Self>;
    fn next(self) -> Option<Self>;
    fn prev(self) -> Option<Self>;
    fn min() -> Self;
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

/// Produce a `Rect` of the given size that is centered both vertically &
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

pub(crate) fn get_display_area(buffer_area: Rect) -> Rect {
    center_rect(buffer_area, consts::DISPLAY_SIZE)
}

pub(crate) fn options_file_path() -> Option<PathBuf> {
    dirs::data_local_dir().map(|p| p.join("ratsnake").join("options.json"))
}
