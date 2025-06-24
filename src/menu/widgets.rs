use crate::consts;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    text::{Line, Span, Text},
    widgets::Widget,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Logo;

impl Logo {
    const RAT_WIDTH: u16 = 15;
    const SNAKE_WIDTH: u16 = 28;
    const SNAKE_BODY_LENGTH: u16 = 12;
    const SNAKE_FRUIT_GUTTER: u16 = 2;
    const TEXT_HEIGHT: u16 = 5;
    pub(super) const HEIGHT: u16 = Self::TEXT_HEIGHT + 2;
    pub(super) const WIDTH: u16 = Self::RAT_WIDTH + Self::SNAKE_WIDTH;

    #[rustfmt::skip]
    const RAT: [&'static str; Self::TEXT_HEIGHT as usize] = [
         " ____       _  ",
        r"|  _ \ __ _| |_",
         "| |_) / _` | __",
         "|  _ < (_| | |_",
        r"|_| \_\__,_|\__",
    ];

    #[rustfmt::skip]
    const SNAKE: [&'static str; Self::TEXT_HEIGHT as usize] = [
         " ____              _        ",
         "/ ___| _ __   __ _| | _____ ",
        r"\___ \| '_ \ / _` | |/ / _ \",
         " ___) | | | | (_| |   <  __/",
        r"|____/|_| |_|\__,_|_|\_\___|",
    ];
}

impl Widget for Logo {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [area] = Layout::horizontal([Self::WIDTH])
            .flex(Flex::Start)
            .areas(area);
        let [words_area, diagram_area] = Layout::vertical([Self::TEXT_HEIGHT, 1])
            .flex(Flex::Start)
            .spacing(1)
            .areas(area);
        let [rat_area, snake_area] = Layout::horizontal([Self::RAT_WIDTH, Self::SNAKE_WIDTH])
            .flex(Flex::Start)
            .areas(words_area);
        Text::from_iter(Self::RAT)
            .style(consts::FRUIT_STYLE)
            .render(rat_area, buf);
        Text::from_iter(Self::SNAKE)
            .style(consts::SNAKE_STYLE)
            .render(snake_area, buf);
        let [body_area, head_area, _, fruit_area] = Layout::horizontal([
            Constraint::Length(Self::SNAKE_BODY_LENGTH),
            Constraint::Length(1),
            Constraint::Length(Self::SNAKE_FRUIT_GUTTER),
            Constraint::Length(1),
        ])
        .flex(Flex::Center)
        .areas(diagram_area);
        for p in body_area.positions() {
            if let Some(cell) = buf.cell_mut(p) {
                cell.set_char(consts::SNAKE_BODY_SYMBOL);
                cell.set_style(consts::SNAKE_STYLE);
            }
        }
        for p in head_area.positions() {
            if let Some(cell) = buf.cell_mut(p) {
                cell.set_char(consts::SNAKE_HEAD_EAST_SYMBOL);
                cell.set_style(consts::SNAKE_STYLE);
            }
        }
        for p in fruit_area.positions() {
            if let Some(cell) = buf.cell_mut(p) {
                cell.set_char(consts::FRUIT_SYMBOL);
                cell.set_style(consts::FRUIT_STYLE);
            }
        }
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
            Logo.render(Rect::new(3, 1, Logo::WIDTH, Logo::HEIGHT), &mut buffer);
            #[rustfmt::skip]
            let mut expected = Buffer::with_lines([
                 "",
                 "    ____       _   ____              _            ",
                r"   |  _ \ __ _| |_/ ___| _ __   __ _| | _____     ",
                r"   | |_) / _` | __\___ \| '_ \ / _` | |/ / _ \    ",
                 "   |  _ < (_| | |_ ___) | | | | (_| |   <  __/    ",
                r"   |_| \_\__,_|\__|____/|_| |_|\__,_|_|\_\___|    ",
                 "",
                 "                 ⚬⚬⚬⚬⚬⚬⚬⚬⚬⚬⚬⚬<  ●                 ",
                 "",
                 "",
            ]);
            expected.set_style(Rect::new(3, 1, 15, 5), consts::FRUIT_STYLE);
            expected.set_style(Rect::new(18, 1, 28, 5), consts::SNAKE_STYLE);
            expected.set_style(Rect::new(17, 7, 13, 1), consts::SNAKE_STYLE);
            expected.set_style(Rect::new(32, 7, 1, 1), consts::FRUIT_STYLE);
            pretty_assertions::assert_eq!(buffer, expected);
        }

        #[test]
        fn test_render_too_big() {
            let mut buffer = Buffer::empty(Rect::new(0, 0, 50, 10));
            Logo.render(Rect::new(3, 1, 50, 10), &mut buffer);
            #[rustfmt::skip]
            let mut expected = Buffer::with_lines([
                 "",
                 "    ____       _   ____              _            ",
                r"   |  _ \ __ _| |_/ ___| _ __   __ _| | _____     ",
                r"   | |_) / _` | __\___ \| '_ \ / _` | |/ / _ \    ",
                 "   |  _ < (_| | |_ ___) | | | | (_| |   <  __/    ",
                r"   |_| \_\__,_|\__|____/|_| |_|\__,_|_|\_\___|    ",
                 "",
                 "                 ⚬⚬⚬⚬⚬⚬⚬⚬⚬⚬⚬⚬<  ●                 ",
                 "",
                 "",
            ]);
            expected.set_style(Rect::new(3, 1, 15, 5), consts::FRUIT_STYLE);
            expected.set_style(Rect::new(18, 1, 28, 5), consts::SNAKE_STYLE);
            expected.set_style(Rect::new(17, 7, 13, 1), consts::SNAKE_STYLE);
            expected.set_style(Rect::new(32, 7, 1, 1), consts::FRUIT_STYLE);
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
