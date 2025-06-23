use crate::consts;
use enum_map::Enum;
use ratatui::layout::{Flex, Layout, Rect};

pub(crate) trait EnumExt: Enum {
    fn next(self) -> Option<Self>;
    fn prev(self) -> Option<Self>;
    fn min() -> Self;
    fn max() -> Self;
}

impl<T: Enum> EnumExt for T {
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

pub(crate) fn get_display_area(buffer_area: Rect) -> Rect {
    let [display] = Layout::horizontal([consts::DISPLAY_SIZE.width])
        .flex(Flex::Center)
        .areas(buffer_area);
    let [display] = Layout::vertical([consts::DISPLAY_SIZE.height])
        .flex(Flex::Center)
        .areas(display);
    display
}
