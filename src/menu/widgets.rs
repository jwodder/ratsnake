use crate::consts;
use ratatui::{
    buffer::Buffer,
    layout::{Offset, Rect},
    text::{Line, Span, Text},
    widgets::Widget,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Logo;

impl Logo {
    const RAT_WIDTH: u16 = 15;
    const SNAKE_WIDTH: u16 = 28;
    pub(super) const HEIGHT: u16 = 5;
    pub(super) const WIDTH: u16 = Self::RAT_WIDTH + Self::SNAKE_WIDTH;

    #[rustfmt::skip]
    const RAT: [&'static str; Self::HEIGHT as usize] = [
         " ____       _  ",
        r"|  _ \ __ _| |_",
         "| |_) / _` | __",
         "|  _ < (_| | |_",
        r"|_| \_\__,_|\__",
    ];

    #[rustfmt::skip]
    const SNAKE: [&'static str; Self::HEIGHT as usize] = [
         " ____              _        ",
         "/ ___| _ __   __ _| | _____ ",
        r"\___ \| '_ \ / _` | |/ / _ \",
         " ___) | | | | (_| |   <  __/",
        r"|____/|_| |_|\__,_|_|\_\___|",
    ];
}

impl Widget for Logo {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let rat_text = Text::from_iter(Self::RAT).style(consts::FRUIT_STYLE);
        rat_text.render(area, buf);
        let snake_text = Text::from_iter(Self::SNAKE).style(consts::SNAKE_STYLE);
        let snake_area = area
            .offset(Offset {
                x: Self::RAT_WIDTH.into(),
                y: 0,
            })
            .intersection(area);
        snake_text.render(snake_area, buf);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Instructions;

impl Instructions {
    pub(super) const HEIGHT: u16 = 7;
    pub(super) const WIDTH: u16 = 20;
}

impl Widget for Instructions {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let text = Text::from_iter([
            Line::from("Move the snake with:"),
            Line::from_iter([
                Span::raw("       "),
                Span::styled("←", consts::KEY_STYLE),
                Span::raw(" "),
                Span::styled("↓", consts::KEY_STYLE),
                Span::raw(" "),
                Span::styled("↑", consts::KEY_STYLE),
                Span::raw(" "),
                Span::styled("→", consts::KEY_STYLE),
            ]),
            Line::from_iter([
                Span::raw("   or: "),
                Span::styled("h", consts::KEY_STYLE),
                Span::raw(" "),
                Span::styled("j", consts::KEY_STYLE),
                Span::raw(" "),
                Span::styled("k", consts::KEY_STYLE),
                Span::raw(" "),
                Span::styled("l", consts::KEY_STYLE),
            ]),
            Line::from_iter([
                Span::raw("   or: "),
                Span::styled("a", consts::KEY_STYLE),
                Span::raw(" "),
                Span::styled("s", consts::KEY_STYLE),
                Span::raw(" "),
                Span::styled("w", consts::KEY_STYLE),
                Span::raw(" "),
                Span::styled("d", consts::KEY_STYLE),
            ]),
            Line::from_iter([
                Span::raw("   or: "),
                Span::styled("4", consts::KEY_STYLE),
                Span::raw(" "),
                Span::styled("2", consts::KEY_STYLE),
                Span::raw(" "),
                Span::styled("8", consts::KEY_STYLE),
                Span::raw(" "),
                Span::styled("6", consts::KEY_STYLE),
            ]),
            Line::from("Eat the fruit, but"),
            Line::from("don't hit yourself!"),
        ]);
        debug_assert_eq!(
            text.height(),
            usize::from(Self::HEIGHT),
            "Instructions::HEIGHT is wrong"
        );
        debug_assert_eq!(
            text.width(),
            usize::from(Self::WIDTH),
            "Instructions::WIDTH is wrong"
        );
        text.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod logo {
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
            pretty_assertions::assert_eq!(buffer, expected);
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
            pretty_assertions::assert_eq!(buffer, expected);
        }

        #[test]
        fn rat_width() {
            assert!(Logo::RAT
                .iter()
                .all(|ln| ln.len() == usize::from(Logo::RAT_WIDTH)));
        }

        #[test]
        fn snake_width() {
            assert!(Logo::SNAKE
                .iter()
                .all(|ln| ln.len() == usize::from(Logo::SNAKE_WIDTH)));
        }
    }
}
