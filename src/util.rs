use crate::consts;
use ratatui::layout::{Flex, Layout, Rect};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Side {
    Top,
    Bottom,
    Left,
    Right,
}

pub(crate) trait RectExt: Sized {
    fn shave(self, side: Side) -> Self;
}

impl RectExt for Rect {
    fn shave(mut self, side: Side) -> Self {
        match side {
            Side::Top => {
                self.y = self.y.saturating_add(1);
                self.height = self.height.saturating_sub(1);
            }
            Side::Bottom => {
                self.height = self.height.saturating_sub(1);
            }
            Side::Left => {
                self.x = self.x.saturating_add(1);
                self.width = self.width.saturating_sub(1);
            }
            Side::Right => {
                self.width = self.width.saturating_sub(1);
            }
        }
        if self.is_empty() {
            Rect::ZERO
        } else {
            self
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(Rect::new(5, 7, 20, 10), Side::Top, Rect::new(5, 8, 20, 9))]
    #[case(Rect::new(5, 7, 20, 10), Side::Bottom, Rect::new(5, 7, 20, 9))]
    #[case(Rect::new(5, 7, 20, 10), Side::Left, Rect::new(6, 7, 19, 10))]
    #[case(Rect::new(5, 7, 20, 10), Side::Right, Rect::new(5, 7, 19, 10))]
    #[case(Rect::new(5, 7, 1, 1), Side::Top, Rect::ZERO)]
    #[case(Rect::new(5, 7, 1, 1), Side::Bottom, Rect::ZERO)]
    #[case(Rect::new(5, 7, 1, 1), Side::Left, Rect::ZERO)]
    #[case(Rect::new(5, 7, 1, 1), Side::Right, Rect::ZERO)]
    #[case(Rect::ZERO, Side::Top, Rect::ZERO)]
    #[case(Rect::ZERO, Side::Bottom, Rect::ZERO)]
    #[case(Rect::ZERO, Side::Left, Rect::ZERO)]
    #[case(Rect::ZERO, Side::Right, Rect::ZERO)]
    fn test_shave(#[case] before: Rect, #[case] side: Side, #[case] after: Rect) {
        assert_eq!(before.shave(side), after);
    }
}
