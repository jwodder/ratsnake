use crate::app::Screen;
use crate::command::Command;
use crate::consts;
use crate::menu::MainMenu;
use crate::options::Options;
use crate::util::{get_display_area, Globals};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget},
    Frame,
};
use std::fmt::Write;
use std::num::NonZeroU32;

/// The high score table screen
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HSTable {
    /// List of high score entries in descending order
    scores: Vec<HSEntry>,

    scroll_offset: usize,

    max_scroll: usize,

    /// Global data (options & high scores)
    globals: Globals,
}

impl HSTable {
    /// The maximum number of rows to display at once
    const MAX_ROWS: u16 = 18;

    const SCORE_COLUMN_WIDTH: u16 = 5;
    const WRAPAROUND_COLUMN_WIDTH: u16 = 10;
    const OBSTACLES_COLUMN_WIDTH: u16 = 9;
    const FRUITS_COLUMN_WIDTH: u16 = 6;
    const LEVEL_SIZE_COLUMN_WIDTH: u16 = 10;

    /// The width of the widget when scrolling is not in effect (When scrolling
    /// is in effect, add 2 to this value for the scrollbar and the margin
    /// between it & the table)
    const WIDTH: u16 = 1
        + HSTable::SCORE_COLUMN_WIDTH
        + 3
        + HSTable::WRAPAROUND_COLUMN_WIDTH
        + 3
        + HSTable::OBSTACLES_COLUMN_WIDTH
        + 3
        + HSTable::FRUITS_COLUMN_WIDTH
        + 3
        + HSTable::LEVEL_SIZE_COLUMN_WIDTH
        + 1;

    /// Create a new high score table screen from the given globals
    pub(crate) fn new(globals: Globals) -> Self {
        let scores = globals.high_scores.to_hsentries();
        let max_scroll = scores.len().saturating_sub(usize::from(Self::MAX_ROWS) - 1);
        HSTable {
            scores,
            scroll_offset: 0,
            max_scroll,
            globals,
        }
    }

    /// Draw the main menu on the given frame
    pub(crate) fn draw(&self, frame: &mut Frame<'_>) {
        frame.render_widget(self, frame.area());
    }

    /// Receive & handle the next input event.
    ///
    /// Returns `Some(screen)` if the application should switch to a different
    /// screen or quit.
    pub(crate) fn process_input(&mut self) -> std::io::Result<Option<Screen>> {
        Ok(crossterm::event::read()?
            .as_key_press_event()
            .and_then(Command::from_key_event)
            .and_then(|cmd| self.handle_command(cmd)))
    }

    /// Process an input command.
    ///
    /// Returns `Some(screen)` if the application should switch to a different
    /// screen or quit.
    fn handle_command(&mut self, cmd: Command) -> Option<Screen> {
        match (cmd, self.scrolling()) {
            (Command::Quit, _) => return Some(Screen::Quit),
            (Command::Esc, _) => return Some(Screen::Main(MainMenu::new(self.globals.clone()))),
            (Command::Up, true) => {
                if self.scroll_offset > 0 {
                    self.scroll_offset -= 1;
                }
            }
            (Command::Down, true) => {
                if self.scroll_offset < self.max_scroll.saturating_sub(1) {
                    self.scroll_offset += 1;
                }
            }
            _ => (),
        }
        None
    }

    /// Does the text not fit in [`MAX_ROWS`][Self::MAX_ROWS], necessitating
    /// scrolling and a scrollbar?
    fn scrolling(&self) -> bool {
        self.scores.len() > usize::from(Self::MAX_ROWS)
    }
}

impl Widget for &HSTable {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let display = get_display_area(area);
        let [title_area, table_area, footer_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .spacing(1)
        .areas(display);

        Line::styled(" High Scores", consts::SCORE_BAR_STYLE).render(title_area, buf);

        if self.scores.is_empty() {
            Line::from("No high scores yet — Go play the game!")
                .centered()
                .render(table_area, buf);
        } else {
            let (table_area, scrollbar_area) = if self.scrolling() {
                let [table_area, scrollbar_area] = Layout::horizontal([HSTable::WIDTH, 1])
                    .flex(Flex::Center)
                    .spacing(1)
                    .areas(table_area);
                let [_, scrollbar_area] =
                    Layout::vertical([Constraint::Length(2), Constraint::Fill(1)])
                        .areas(scrollbar_area);
                (table_area, Some(scrollbar_area))
            } else {
                let [table_area] = Layout::horizontal([HSTable::WIDTH])
                    .flex(Flex::Center)
                    .areas(table_area);
                (table_area, None)
            };

            let mut rows = table_area.rows();
            if let Some(header_area) = rows.next() {
                Line::from_iter([
                    Span::raw(" "),
                    Span::styled("Score", consts::HSTABLE_HEADER_STYLE),
                    Span::raw(" │ "),
                    Span::styled("Wraparound", consts::HSTABLE_HEADER_STYLE),
                    Span::raw(" │ "),
                    Span::styled("Obstacles", consts::HSTABLE_HEADER_STYLE),
                    Span::raw(" │ "),
                    Span::styled("Fruits", consts::HSTABLE_HEADER_STYLE),
                    Span::raw(" │ "),
                    Span::styled("Level Size", consts::HSTABLE_HEADER_STYLE),
                    Span::raw(" "),
                ])
                .render(header_area, buf);
            }
            if let Some(hrule_area) = rows.next() {
                let mut s = String::new();
                for _ in 0..(HSTable::SCORE_COLUMN_WIDTH + 2) {
                    s.push('─');
                }
                s.push('┼');
                for _ in 0..(HSTable::WRAPAROUND_COLUMN_WIDTH + 2) {
                    s.push('─');
                }
                s.push('┼');
                for _ in 0..(HSTable::OBSTACLES_COLUMN_WIDTH + 2) {
                    s.push('─');
                }
                s.push('┼');
                for _ in 0..(HSTable::FRUITS_COLUMN_WIDTH + 2) {
                    s.push('─');
                }
                s.push('┼');
                for _ in 0..(HSTable::LEVEL_SIZE_COLUMN_WIDTH + 2) {
                    s.push('─');
                }
                buf.set_string(hrule_area.x, hrule_area.y, s, Style::new());
            }
            for (&entry, row_area) in std::iter::zip(&self.scores[self.scroll_offset..], rows) {
                let mut s = String::new();
                let _ = write!(
                    &mut s,
                    " {val:>width$}",
                    val = entry.score,
                    width = HSTable::SCORE_COLUMN_WIDTH as usize,
                );
                let _ = write!(
                    &mut s,
                    " │ {val:^width$}",
                    val = if entry.options.wraparound {
                        '✓'
                    } else {
                        '✗'
                    },
                    width = HSTable::WRAPAROUND_COLUMN_WIDTH as usize,
                );
                let _ = write!(
                    &mut s,
                    " │ {val:^width$}",
                    val = if entry.options.obstacles {
                        '✓'
                    } else {
                        '✗'
                    },
                    width = HSTable::OBSTACLES_COLUMN_WIDTH as usize,
                );
                let _ = write!(
                    &mut s,
                    " │ {val:>width$}",
                    val = entry.options.fruits,
                    width = HSTable::FRUITS_COLUMN_WIDTH as usize,
                );
                let _ = write!(
                    &mut s,
                    " │ {val:width$}",
                    val = entry.options.level_size,
                    width = HSTable::LEVEL_SIZE_COLUMN_WIDTH as usize,
                );
                let _ = write!(&mut s, " ");
                buf.set_string(row_area.x, row_area.y, s, Style::new());
            }
            if let Some(scrollbar_area) = scrollbar_area {
                let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .track_symbol(Some(ratatui::symbols::shade::MEDIUM));
                let mut scroll_state =
                    ScrollbarState::new(self.max_scroll).position(self.scroll_offset);
                scrollbar.render(scrollbar_area, buf, &mut scroll_state);
            }
        }

        Line::from_iter([
            Span::raw(" Press "),
            Span::styled("Esc", consts::KEY_STYLE),
            Span::raw(" to return to main menu"),
        ])
        .render(footer_area, buf);
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct HSEntry {
    pub(crate) score: NonZeroU32,
    pub(crate) options: Options,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::{FruitQty, LevelSize};
    use ratatui::layout::Rect;

    #[test]
    fn no_scores() {
        let hstable = HSTable::new(Globals::default());
        let area = Rect::new(0, 0, 80, 24);
        let mut buffer = Buffer::empty(area);
        hstable.render(area, &mut buffer);
        let mut expected = Buffer::with_lines([
            " High Scores                                                                    ",
            "                                                                                ",
            "                     No high scores yet — Go play the game!                     ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            " Press Esc to return to main menu                                               ",
        ]);
        expected.set_style(Rect::new(0, 0, 80, 1), consts::SCORE_BAR_STYLE);
        expected.set_style(Rect::new(7, 23, 3, 1), consts::KEY_STYLE); // "Esc"
        pretty_assertions::assert_eq!(buffer, expected);
    }

    #[test]
    fn one_score() {
        let mut globals = Globals::default();
        globals
            .high_scores
            .set(globals.options, NonZeroU32::new(42).unwrap());
        let hstable = HSTable::new(globals);
        let area = Rect::new(0, 0, 80, 24);
        let mut buffer = Buffer::empty(area);
        hstable.render(area, &mut buffer);
        let mut expected = Buffer::with_lines([
            " High Scores                                                                    ",
            "                                                                                ",
            "              Score │ Wraparound │ Obstacles │ Fruits │ Level Size              ",
            "             ───────┼────────────┼───────────┼────────┼────────────             ",
            "                 42 │     ✗      │     ✗     │      1 │ Large                   ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            "                                                                                ",
            " Press Esc to return to main menu                                               ",
        ]);
        expected.set_style(Rect::new(0, 0, 80, 1), consts::SCORE_BAR_STYLE);
        expected.set_style(Rect::new(14, 2, 5, 1), consts::HSTABLE_HEADER_STYLE); // "Score"
        expected.set_style(Rect::new(22, 2, 10, 1), consts::HSTABLE_HEADER_STYLE); // "Wraparound"
        expected.set_style(Rect::new(35, 2, 9, 1), consts::HSTABLE_HEADER_STYLE); // "Obstacles"
        expected.set_style(Rect::new(47, 2, 6, 1), consts::HSTABLE_HEADER_STYLE); // "Fruits"
        expected.set_style(Rect::new(56, 2, 10, 1), consts::HSTABLE_HEADER_STYLE); // "Level Size"
        expected.set_style(Rect::new(7, 23, 3, 1), consts::KEY_STYLE); // "Esc"
        pretty_assertions::assert_eq!(buffer, expected);
    }

    #[test]
    fn full_screen_of_scores() {
        let mut globals = Globals::default();
        let mut i = 1;
        for obstacles in [false, true] {
            for fruits in 1..=9 {
                let fruits = FruitQty::new(fruits).unwrap();
                globals.high_scores.set(
                    Options {
                        fruits,
                        obstacles,
                        ..Options::default()
                    },
                    NonZeroU32::new(i).unwrap(),
                );
                i += 1;
            }
        }
        let hstable = HSTable::new(globals);
        let area = Rect::new(0, 0, 80, 24);
        let mut buffer = Buffer::empty(area);
        hstable.render(area, &mut buffer);
        let mut expected = Buffer::with_lines([
            " High Scores                                                                    ",
            "                                                                                ",
            "              Score │ Wraparound │ Obstacles │ Fruits │ Level Size              ",
            "             ───────┼────────────┼───────────┼────────┼────────────             ",
            "                 18 │     ✗      │     ✓     │      9 │ Large                   ",
            "                 17 │     ✗      │     ✓     │      8 │ Large                   ",
            "                 16 │     ✗      │     ✓     │      7 │ Large                   ",
            "                 15 │     ✗      │     ✓     │      6 │ Large                   ",
            "                 14 │     ✗      │     ✓     │      5 │ Large                   ",
            "                 13 │     ✗      │     ✓     │      4 │ Large                   ",
            "                 12 │     ✗      │     ✓     │      3 │ Large                   ",
            "                 11 │     ✗      │     ✓     │      2 │ Large                   ",
            "                 10 │     ✗      │     ✓     │      1 │ Large                   ",
            "                  9 │     ✗      │     ✗     │      9 │ Large                   ",
            "                  8 │     ✗      │     ✗     │      8 │ Large                   ",
            "                  7 │     ✗      │     ✗     │      7 │ Large                   ",
            "                  6 │     ✗      │     ✗     │      6 │ Large                   ",
            "                  5 │     ✗      │     ✗     │      5 │ Large                   ",
            "                  4 │     ✗      │     ✗     │      4 │ Large                   ",
            "                  3 │     ✗      │     ✗     │      3 │ Large                   ",
            "                  2 │     ✗      │     ✗     │      2 │ Large                   ",
            "                  1 │     ✗      │     ✗     │      1 │ Large                   ",
            "                                                                                ",
            " Press Esc to return to main menu                                               ",
        ]);
        expected.set_style(Rect::new(0, 0, 80, 1), consts::SCORE_BAR_STYLE);
        expected.set_style(Rect::new(14, 2, 5, 1), consts::HSTABLE_HEADER_STYLE); // "Score"
        expected.set_style(Rect::new(22, 2, 10, 1), consts::HSTABLE_HEADER_STYLE); // "Wraparound"
        expected.set_style(Rect::new(35, 2, 9, 1), consts::HSTABLE_HEADER_STYLE); // "Obstacles"
        expected.set_style(Rect::new(47, 2, 6, 1), consts::HSTABLE_HEADER_STYLE); // "Fruits"
        expected.set_style(Rect::new(56, 2, 10, 1), consts::HSTABLE_HEADER_STYLE); // "Level Size"
        expected.set_style(Rect::new(7, 23, 3, 1), consts::KEY_STYLE); // "Esc"
        pretty_assertions::assert_eq!(buffer, expected);
    }

    #[test]
    fn scrolling() {
        let mut globals = Globals::default();
        let mut i = 1;
        for obstacles in [false, true] {
            for fruits in 1..=9 {
                let fruits = FruitQty::new(fruits).unwrap();
                for level_size in [LevelSize::Small, LevelSize::Medium, LevelSize::Large] {
                    globals.high_scores.set(
                        Options {
                            fruits,
                            obstacles,
                            level_size,
                            ..Options::default()
                        },
                        NonZeroU32::new(i).unwrap(),
                    );
                    i += 1;
                }
            }
        }
        let mut hstable = HSTable::new(globals);
        let area = Rect::new(0, 0, 80, 24);
        let mut buffer = Buffer::empty(area);
        hstable.render(area, &mut buffer);
        let mut expected = Buffer::with_lines([
            " High Scores                                                                    ",
            "                                                                                ",
            "             Score │ Wraparound │ Obstacles │ Fruits │ Level Size               ",
            "            ───────┼────────────┼───────────┼────────┼────────────              ",
            "                54 │     ✗      │     ✓     │      9 │ Large       ▲            ",
            "                53 │     ✗      │     ✓     │      9 │ Medium      █            ",
            "                52 │     ✗      │     ✓     │      9 │ Small       █            ",
            "                51 │     ✗      │     ✓     │      8 │ Large       █            ",
            "                50 │     ✗      │     ✓     │      8 │ Medium      █            ",
            "                49 │     ✗      │     ✓     │      8 │ Small       █            ",
            "                48 │     ✗      │     ✓     │      7 │ Large       ▒            ",
            "                47 │     ✗      │     ✓     │      7 │ Medium      ▒            ",
            "                46 │     ✗      │     ✓     │      7 │ Small       ▒            ",
            "                45 │     ✗      │     ✓     │      6 │ Large       ▒            ",
            "                44 │     ✗      │     ✓     │      6 │ Medium      ▒            ",
            "                43 │     ✗      │     ✓     │      6 │ Small       ▒            ",
            "                42 │     ✗      │     ✓     │      5 │ Large       ▒            ",
            "                41 │     ✗      │     ✓     │      5 │ Medium      ▒            ",
            "                40 │     ✗      │     ✓     │      5 │ Small       ▒            ",
            "                39 │     ✗      │     ✓     │      4 │ Large       ▒            ",
            "                38 │     ✗      │     ✓     │      4 │ Medium      ▒            ",
            "                37 │     ✗      │     ✓     │      4 │ Small       ▼            ",
            "                                                                                ",
            " Press Esc to return to main menu                                               ",
        ]);
        expected.set_style(Rect::new(0, 0, 80, 1), consts::SCORE_BAR_STYLE);
        expected.set_style(Rect::new(13, 2, 5, 1), consts::HSTABLE_HEADER_STYLE); // "Score"
        expected.set_style(Rect::new(21, 2, 10, 1), consts::HSTABLE_HEADER_STYLE); // "Wraparound"
        expected.set_style(Rect::new(34, 2, 9, 1), consts::HSTABLE_HEADER_STYLE); // "Obstacles"
        expected.set_style(Rect::new(46, 2, 6, 1), consts::HSTABLE_HEADER_STYLE); // "Fruits"
        expected.set_style(Rect::new(55, 2, 10, 1), consts::HSTABLE_HEADER_STYLE); // "Level Size"
        expected.set_style(Rect::new(7, 23, 3, 1), consts::KEY_STYLE); // "Esc"
        pretty_assertions::assert_eq!(buffer, expected);

        assert!(hstable.handle_command(Command::Down).is_none());
        let mut buffer = Buffer::empty(area);
        hstable.render(area, &mut buffer);
        let mut expected = Buffer::with_lines([
            " High Scores                                                                    ",
            "                                                                                ",
            "             Score │ Wraparound │ Obstacles │ Fruits │ Level Size               ",
            "            ───────┼────────────┼───────────┼────────┼────────────              ",
            "                53 │     ✗      │     ✓     │      9 │ Medium      ▲            ",
            "                52 │     ✗      │     ✓     │      9 │ Small       █            ",
            "                51 │     ✗      │     ✓     │      8 │ Large       █            ",
            "                50 │     ✗      │     ✓     │      8 │ Medium      █            ",
            "                49 │     ✗      │     ✓     │      8 │ Small       █            ",
            "                48 │     ✗      │     ✓     │      7 │ Large       █            ",
            "                47 │     ✗      │     ✓     │      7 │ Medium      █            ",
            "                46 │     ✗      │     ✓     │      7 │ Small       ▒            ",
            "                45 │     ✗      │     ✓     │      6 │ Large       ▒            ",
            "                44 │     ✗      │     ✓     │      6 │ Medium      ▒            ",
            "                43 │     ✗      │     ✓     │      6 │ Small       ▒            ",
            "                42 │     ✗      │     ✓     │      5 │ Large       ▒            ",
            "                41 │     ✗      │     ✓     │      5 │ Medium      ▒            ",
            "                40 │     ✗      │     ✓     │      5 │ Small       ▒            ",
            "                39 │     ✗      │     ✓     │      4 │ Large       ▒            ",
            "                38 │     ✗      │     ✓     │      4 │ Medium      ▒            ",
            "                37 │     ✗      │     ✓     │      4 │ Small       ▒            ",
            "                36 │     ✗      │     ✓     │      3 │ Large       ▼            ",
            "                                                                                ",
            " Press Esc to return to main menu                                               ",
        ]);
        expected.set_style(Rect::new(0, 0, 80, 1), consts::SCORE_BAR_STYLE);
        expected.set_style(Rect::new(13, 2, 5, 1), consts::HSTABLE_HEADER_STYLE); // "Score"
        expected.set_style(Rect::new(21, 2, 10, 1), consts::HSTABLE_HEADER_STYLE); // "Wraparound"
        expected.set_style(Rect::new(34, 2, 9, 1), consts::HSTABLE_HEADER_STYLE); // "Obstacles"
        expected.set_style(Rect::new(46, 2, 6, 1), consts::HSTABLE_HEADER_STYLE); // "Fruits"
        expected.set_style(Rect::new(55, 2, 10, 1), consts::HSTABLE_HEADER_STYLE); // "Level Size"
        expected.set_style(Rect::new(7, 23, 3, 1), consts::KEY_STYLE); // "Esc"
        pretty_assertions::assert_eq!(buffer, expected);

        for _ in 0..18 {
            assert!(hstable.handle_command(Command::Down).is_none());
        }
        let mut buffer = Buffer::empty(area);
        hstable.render(area, &mut buffer);
        let mut expected = Buffer::with_lines([
            " High Scores                                                                    ",
            "                                                                                ",
            "             Score │ Wraparound │ Obstacles │ Fruits │ Level Size               ",
            "            ───────┼────────────┼───────────┼────────┼────────────              ",
            "                35 │     ✗      │     ✓     │      3 │ Medium      ▲            ",
            "                34 │     ✗      │     ✓     │      3 │ Small       ▒            ",
            "                33 │     ✗      │     ✓     │      2 │ Large       ▒            ",
            "                32 │     ✗      │     ✓     │      2 │ Medium      ▒            ",
            "                31 │     ✗      │     ✓     │      2 │ Small       ▒            ",
            "                30 │     ✗      │     ✓     │      1 │ Large       ▒            ",
            "                29 │     ✗      │     ✓     │      1 │ Medium      ▒            ",
            "                28 │     ✗      │     ✓     │      1 │ Small       █            ",
            "                27 │     ✗      │     ✗     │      9 │ Large       █            ",
            "                26 │     ✗      │     ✗     │      9 │ Medium      █            ",
            "                25 │     ✗      │     ✗     │      9 │ Small       █            ",
            "                24 │     ✗      │     ✗     │      8 │ Large       █            ",
            "                23 │     ✗      │     ✗     │      8 │ Medium      ▒            ",
            "                22 │     ✗      │     ✗     │      8 │ Small       ▒            ",
            "                21 │     ✗      │     ✗     │      7 │ Large       ▒            ",
            "                20 │     ✗      │     ✗     │      7 │ Medium      ▒            ",
            "                19 │     ✗      │     ✗     │      7 │ Small       ▒            ",
            "                18 │     ✗      │     ✗     │      6 │ Large       ▼            ",
            "                                                                                ",
            " Press Esc to return to main menu                                               ",
        ]);
        expected.set_style(Rect::new(0, 0, 80, 1), consts::SCORE_BAR_STYLE);
        expected.set_style(Rect::new(13, 2, 5, 1), consts::HSTABLE_HEADER_STYLE); // "Score"
        expected.set_style(Rect::new(21, 2, 10, 1), consts::HSTABLE_HEADER_STYLE); // "Wraparound"
        expected.set_style(Rect::new(34, 2, 9, 1), consts::HSTABLE_HEADER_STYLE); // "Obstacles"
        expected.set_style(Rect::new(46, 2, 6, 1), consts::HSTABLE_HEADER_STYLE); // "Fruits"
        expected.set_style(Rect::new(55, 2, 10, 1), consts::HSTABLE_HEADER_STYLE); // "Level Size"
        expected.set_style(Rect::new(7, 23, 3, 1), consts::KEY_STYLE); // "Esc"
        pretty_assertions::assert_eq!(buffer, expected);

        for _ in 0..17 {
            assert!(hstable.handle_command(Command::Down).is_none());
        }
        let mut buffer = Buffer::empty(area);
        hstable.render(area, &mut buffer);
        let mut expected = Buffer::with_lines([
            " High Scores                                                                    ",
            "                                                                                ",
            "             Score │ Wraparound │ Obstacles │ Fruits │ Level Size               ",
            "            ───────┼────────────┼───────────┼────────┼────────────              ",
            "                18 │     ✗      │     ✗     │      6 │ Large       ▲            ",
            "                17 │     ✗      │     ✗     │      6 │ Medium      ▒            ",
            "                16 │     ✗      │     ✗     │      6 │ Small       ▒            ",
            "                15 │     ✗      │     ✗     │      5 │ Large       ▒            ",
            "                14 │     ✗      │     ✗     │      5 │ Medium      ▒            ",
            "                13 │     ✗      │     ✗     │      5 │ Small       ▒            ",
            "                12 │     ✗      │     ✗     │      4 │ Large       ▒            ",
            "                11 │     ✗      │     ✗     │      4 │ Medium      ▒            ",
            "                10 │     ✗      │     ✗     │      4 │ Small       ▒            ",
            "                 9 │     ✗      │     ✗     │      3 │ Large       ▒            ",
            "                 8 │     ✗      │     ✗     │      3 │ Medium      ▒            ",
            "                 7 │     ✗      │     ✗     │      3 │ Small       ▒            ",
            "                 6 │     ✗      │     ✗     │      2 │ Large       █            ",
            "                 5 │     ✗      │     ✗     │      2 │ Medium      █            ",
            "                 4 │     ✗      │     ✗     │      2 │ Small       █            ",
            "                 3 │     ✗      │     ✗     │      1 │ Large       █            ",
            "                 2 │     ✗      │     ✗     │      1 │ Medium      █            ",
            "                 1 │     ✗      │     ✗     │      1 │ Small       ▼            ",
            "                                                                                ",
            " Press Esc to return to main menu                                               ",
        ]);
        expected.set_style(Rect::new(0, 0, 80, 1), consts::SCORE_BAR_STYLE);
        expected.set_style(Rect::new(13, 2, 5, 1), consts::HSTABLE_HEADER_STYLE); // "Score"
        expected.set_style(Rect::new(21, 2, 10, 1), consts::HSTABLE_HEADER_STYLE); // "Wraparound"
        expected.set_style(Rect::new(34, 2, 9, 1), consts::HSTABLE_HEADER_STYLE); // "Obstacles"
        expected.set_style(Rect::new(46, 2, 6, 1), consts::HSTABLE_HEADER_STYLE); // "Fruits"
        expected.set_style(Rect::new(55, 2, 10, 1), consts::HSTABLE_HEADER_STYLE); // "Level Size"
        expected.set_style(Rect::new(7, 23, 3, 1), consts::KEY_STYLE); // "Esc"
        pretty_assertions::assert_eq!(buffer, expected);

        assert!(hstable.handle_command(Command::Down).is_none());
        let mut buffer = Buffer::empty(area);
        hstable.render(area, &mut buffer);
        pretty_assertions::assert_eq!(buffer, expected);
    }
}
