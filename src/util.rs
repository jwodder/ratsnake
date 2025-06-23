use crate::consts;
use enum_map::Enum;
use ratatui::layout::{Flex, Layout, Rect, Size};

pub(crate) trait EnumExt: Enum {
    fn iter() -> impl Iterator<Item = Self>;
    fn next(self) -> Option<Self>;
    fn prev(self) -> Option<Self>;
    fn min() -> Self;
    fn max() -> Self;
}

impl<T: Enum> EnumExt for T {
    fn iter() -> impl Iterator<Item = Self> {
        (0..Self::LENGTH).map(Self::from_usize)
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
