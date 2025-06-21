use crate::consts;
use ratatui::{
    buffer::Buffer,
    layout::{Offset, Rect},
    text::Text,
    widgets::Widget,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Logo;

impl Logo {
    const RAT_WIDTH: u16 = 15;
    const SNAKE_WIDTH: u16 = 28;
    pub(crate) const HEIGHT: u16 = 5;
    pub(crate) const WIDTH: u16 = Self::RAT_WIDTH + Self::SNAKE_WIDTH;
}

#[rustfmt::skip]
static RAT: &[&str] = &[
     " ____       _  ",
    r"|  _ \ __ _| |_",
     "| |_) / _` | __",
     "|  _ < (_| | |_",
    r"|_| \_\__,_|\__",
];

#[rustfmt::skip]
static SNAKE: &[&str] = &[
     " ____              _        ",
     "/ ___| _ __   __ _| | _____ ",
    r"\___ \| '_ \ / _` | |/ / _ \",
     " ___) | | | | (_| |   <  __/",
    r"|____/|_| |_|\__,_|_|\_\___|",
];

impl Widget for Logo {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let rat_text = Text::from_iter(RAT.iter().copied()).style(consts::FRUIT_STYLE);
        rat_text.render(area, buf);
        let snake_text = Text::from_iter(SNAKE.iter().copied()).style(consts::SNAKE_STYLE);
        let snake_area = area
            .offset(Offset {
                x: Self::RAT_WIDTH.into(),
                y: 0,
            })
            .intersection(area);
        snake_text.render(snake_area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 50, 10));
        Logo.render(Rect::new(3, 2, 50, 10), &mut buffer);
        let mut expected = Buffer::with_lines([
            "",
            "",
            "    ____       _   ____              _            ",
            "   |  _ \\ __ _| |_/ ___| _ __   __ _| | _____     ",
            "   | |_) / _` | __\\___ \\| '_ \\ / _` | |/ / _ \\    ",
            "   |  _ < (_| | |_ ___) | | | | (_| |   <  __/    ",
            "   |_| \\_\\__,_|\\__|____/|_| |_|\\__,_|_|\\_\\___|    ",
            "",
            "",
            "",
        ]);
        expected.set_style(Rect::new(3, 2, 15, 8), consts::FRUIT_STYLE);
        expected.set_style(Rect::new(18, 2, 32, 8), consts::SNAKE_STYLE);
        assert_eq!(buffer, expected);
    }

    #[test]
    fn test_render_too_small() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 50, 10));
        Logo.render(Rect::new(3, 2, 40, 3), &mut buffer);
        let mut expected = Buffer::with_lines([
            "",
            "",
            "    ____       _   ____              _            ",
            "   |  _ \\ __ _| |_/ ___| _ __   __ _| | ___       ",
            "   | |_) / _` | __\\___ \\| '_ \\ / _` | |/ /        ",
            "",
            "",
            "",
            "",
            "",
        ]);
        expected.set_style(Rect::new(3, 2, 15, 3), consts::FRUIT_STYLE);
        expected.set_style(Rect::new(18, 2, 25, 3), consts::SNAKE_STYLE);
        assert_eq!(buffer, expected);
    }

    #[test]
    fn rat_width() {
        assert!(RAT
            .iter()
            .all(|ln| ln.len() == usize::from(Logo::RAT_WIDTH)));
    }

    #[test]
    fn snake_width() {
        assert!(SNAKE
            .iter()
            .all(|ln| ln.len() == usize::from(Logo::SNAKE_WIDTH)));
    }

    #[test]
    fn height() {
        assert_eq!(RAT.len(), usize::from(Logo::HEIGHT));
        assert_eq!(SNAKE.len(), usize::from(Logo::HEIGHT));
    }
}
