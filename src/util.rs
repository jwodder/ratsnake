use crate::consts;
use ratatui::layout::{Flex, Layout, Rect};

pub(crate) fn get_display_area(buffer_area: Rect) -> Rect {
    let [display] = Layout::horizontal([consts::DISPLAY_SIZE.width])
        .flex(Flex::Center)
        .areas(buffer_area);
    let [display] = Layout::vertical([consts::DISPLAY_SIZE.height])
        .flex(Flex::Center)
        .areas(display);
    display
}
