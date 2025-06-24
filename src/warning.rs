use crate::command::Command;
use crate::util::center_rect;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Flex, Layout, Rect, Size},
    text::{Line, Text},
    widgets::{
        block::{Block, Padding},
        Clear, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget,
    },
};
use std::borrow::Cow;

/// A widget for displaying a warning about an error (including its source
/// traceback messages) in a pop-up window
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Warning {
    lines: Vec<String>,
    scroll_offset: usize,
    max_scroll: usize,
}

impl Warning {
    /// The maximum number of lines to display at once
    const MAX_LINES: u16 = 16;

    /// The width of the text area
    const TEXT_WIDTH: u16 = 48;

    /// The width of the widget when scrolling is not in effect (When scrolling
    /// is in effect, add 2 to this value for the scrollbar and the margin
    /// between it & the text)
    const WIDTH: u16 = Self::TEXT_WIDTH + 4;

    /// Process an input command.  Returns `Some` if the user dismisses the
    /// pop-up or quits the application.
    pub(crate) fn handle_command(&mut self, cmd: Command) -> Option<WarningOutcome> {
        match (cmd, self.scrolling()) {
            (Command::Enter, _) => return Some(WarningOutcome::Dismissed),
            (Command::Quit, _) => return Some(WarningOutcome::Quit),
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

    /// Does the text not fit in [`MAX_LINES`][Self::MAX_LINES], necessitating
    /// scrolling and a scrollbar?
    fn scrolling(&self) -> bool {
        self.lines.len() > usize::from(Self::MAX_LINES)
    }

    /// Create a new `Warning` from a list of error messages and their source
    /// error messages.
    ///
    /// `msgs[0]` is the `Display` of the top-level error being reported.
    /// `msgs[1]` is the `Display` of the top-level error's source, if any.
    /// `msgs[2]` is the `Display` of the source's source, and so on.
    ///
    /// # Panics
    ///
    /// Panics if `msgs` is empty.
    fn from_error_messages(msgs: Vec<String>) -> Self {
        assert!(
            !msgs.is_empty(),
            "Empty Vec passed to Warning::from_error_messages()"
        );
        let mut lines = Vec::new();
        let opts = textwrap::Options::new(usize::from(Warning::TEXT_WIDTH)).break_words(true);
        lines.extend(
            textwrap::wrap(msgs[0].as_str(), opts)
                .into_iter()
                .map(Cow::into_owned),
        );
        if msgs.len() > 1 {
            lines.push(String::new());
            lines.push(String::from("Caused by:"));
            if msgs.len() > 2 {
                for (i, m) in msgs.into_iter().skip(1).enumerate() {
                    let init_indent = format!("{i:>5}: ");
                    let opts = textwrap::Options::new(usize::from(Warning::TEXT_WIDTH))
                        .break_words(true)
                        .initial_indent(&init_indent)
                        .subsequent_indent("       ");
                    lines.extend(textwrap::wrap(&m, opts).into_iter().map(Cow::into_owned));
                }
            } else {
                let opts = textwrap::Options::new(usize::from(Warning::TEXT_WIDTH))
                    .break_words(true)
                    .initial_indent("    ")
                    .subsequent_indent("    ");
                lines.extend(
                    textwrap::wrap(msgs[1].as_str(), opts)
                        .into_iter()
                        .map(Cow::into_owned),
                );
            }
        }
        let max_scroll = lines
            .len()
            .saturating_sub(usize::from(Warning::MAX_LINES) - 1);
        Warning {
            lines,
            scroll_offset: 0,
            max_scroll,
        }
    }
}

/// An enum of the ways that the user can finish working with a `Warning`
/// pop-up
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WarningOutcome {
    /// The user dismissed the pop-up
    Dismissed,

    /// The user quit the application
    Quit,
}

impl<E: std::error::Error> From<E> for Warning {
    fn from(e: E) -> Warning {
        let mut msgs = vec![e.to_string()];
        let mut source = e.source();
        while let Some(src) = source {
            msgs.push(src.to_string());
            source = e.source();
        }
        Warning::from_error_messages(msgs)
    }
}

impl Widget for &Warning {
    /// Render a `Warning` in the given area of `buf`.
    ///
    /// Note that `area` should be the area of the entire display within which
    /// the pop-up will be rendered.  The `render()` method will calculate a
    /// `Rect` for the actual area inside `area` on which the drawing will
    /// occur.
    fn render(self, area: Rect, buf: &mut Buffer) {
        let height = u16::try_from(self.lines.len())
            .unwrap_or(u16::MAX)
            .min(Warning::MAX_LINES)
            .saturating_add(4);
        let block_area = center_rect(
            area,
            Size {
                width: Warning::WIDTH.saturating_add(u16::from(self.scrolling()) * 2),
                height,
            },
        );
        let block = Block::bordered()
            .title(" WARNING ")
            .title_alignment(Alignment::Center)
            .padding(Padding::horizontal(1));
        let [text_area, ok_area] = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)])
            .flex(Flex::Start)
            .spacing(1)
            .areas(block.inner(block_area));
        Clear.render(block_area, buf);
        block.render(block_area, buf);
        if self.scrolling() {
            let [text_area, scrollbar_area] =
                Layout::horizontal([Constraint::Fill(1), Constraint::Length(1)])
                    .flex(Flex::Start)
                    .spacing(1)
                    .areas(text_area);
            Text::from_iter(
                self.lines
                    .iter()
                    .skip(self.scroll_offset)
                    .take(usize::from(Warning::MAX_LINES))
                    .map(String::as_str),
            )
            .render(text_area, buf);
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .track_symbol(Some(ratatui::symbols::shade::MEDIUM));
            let mut scroll_state =
                ScrollbarState::new(self.max_scroll).position(self.scroll_offset);
            scrollbar.render(scrollbar_area, buf, &mut scroll_state);
        } else {
            Text::from_iter(self.lines.iter().map(String::as_str)).render(text_area, buf);
        }
        Line::from("[OK]").centered().render(ok_area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{buffer::Buffer, layout::Rect};

    #[test]
    fn render_no_cause() {
        let warning = Warning::from_error_messages(vec![String::from("Terminal broke")]);
        let area = Rect::new(0, 0, 80, 24);
        let mut buffer = Buffer::empty(area);
        warning.render(area, &mut buffer);
        let expected = Buffer::with_lines([
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "              ┌──────────────────── WARNING ─────────────────────┐              ",
            "              │ Terminal broke                                   │              ",
            "              │                                                  │              ",
            "              │                       [OK]                       │              ",
            "              └──────────────────────────────────────────────────┘              ",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
        ]);
        pretty_assertions::assert_eq!(buffer, expected);
    }

    #[test]
    fn render_one_cause() {
        let warning = Warning::from_error_messages(vec![
            String::from("Terminal broke"),
            String::from("not a tty"),
        ]);
        let area = Rect::new(0, 0, 80, 24);
        let mut buffer = Buffer::empty(area);
        warning.render(area, &mut buffer);
        let expected = Buffer::with_lines([
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "              ┌──────────────────── WARNING ─────────────────────┐              ",
            "              │ Terminal broke                                   │              ",
            "              │                                                  │              ",
            "              │ Caused by:                                       │              ",
            "              │     not a tty                                    │              ",
            "              │                                                  │              ",
            "              │                       [OK]                       │              ",
            "              └──────────────────────────────────────────────────┘              ",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
        ]);
        pretty_assertions::assert_eq!(buffer, expected);
    }

    #[test]
    fn render_two_causes() {
        let warning = Warning::from_error_messages(vec![
            String::from("Failed to draw snake"),
            String::from("terminal broke"),
            String::from("not a tty"),
        ]);
        let area = Rect::new(0, 0, 80, 24);
        let mut buffer = Buffer::empty(area);
        warning.render(area, &mut buffer);
        let expected = Buffer::with_lines([
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "              ┌──────────────────── WARNING ─────────────────────┐              ",
            "              │ Failed to draw snake                             │              ",
            "              │                                                  │              ",
            "              │ Caused by:                                       │              ",
            "              │     0: terminal broke                            │              ",
            "              │     1: not a tty                                 │              ",
            "              │                                                  │              ",
            "              │                       [OK]                       │              ",
            "              └──────────────────────────────────────────────────┘              ",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
        ]);
        pretty_assertions::assert_eq!(buffer, expected);
    }

    #[test]
    fn render_wrapped_one_cause() {
        let warning = Warning::from_error_messages(vec![
            String::from("'Twas brillig, and the slithy toves did gyre and gimble in the wabe"),
            String::from("All mimsy were the borogoves, and the mome raths outgrabe"),
        ]);
        let area = Rect::new(0, 0, 80, 24);
        let mut buffer = Buffer::empty(area);
        warning.render(area, &mut buffer);
        let expected = Buffer::with_lines([
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "              ┌──────────────────── WARNING ─────────────────────┐              ",
            "              │ 'Twas brillig, and the slithy toves did gyre and │              ",
            "              │ gimble in the wabe                               │              ",
            "              │                                                  │              ",
            "              │ Caused by:                                       │              ",
            "              │     All mimsy were the borogoves, and the mome   │              ",
            "              │     raths outgrabe                               │              ",
            "              │                                                  │              ",
            "              │                       [OK]                       │              ",
            "              └──────────────────────────────────────────────────┘              ",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
        ]);
        pretty_assertions::assert_eq!(buffer, expected);
    }

    #[test]
    fn render_wrapped_two_causes() {
        let warning = Warning::from_error_messages(vec![
            String::from("'Twas brillig, and the slithy toves did gyre and gimble in the wabe"),
            String::from("All mimsy were the borogoves, and the mome raths outgrabe"),
            String::from(
                "Beware the Jabberwock, my son!  The jaws that bite, the claws that catch!",
            ),
        ]);
        let area = Rect::new(0, 0, 80, 24);
        let mut buffer = Buffer::empty(area);
        warning.render(area, &mut buffer);
        let expected = Buffer::with_lines([
            "",
            "",
            "",
            "",
            "",
            "",
            "              ┌──────────────────── WARNING ─────────────────────┐              ",
            "              │ 'Twas brillig, and the slithy toves did gyre and │              ",
            "              │ gimble in the wabe                               │              ",
            "              │                                                  │              ",
            "              │ Caused by:                                       │              ",
            "              │     0: All mimsy were the borogoves, and the     │              ",
            "              │        mome raths outgrabe                       │              ",
            "              │     1: Beware the Jabberwock, my son!  The jaws  │              ",
            "              │        that bite, the claws that catch!          │              ",
            "              │                                                  │              ",
            "              │                       [OK]                       │              ",
            "              └──────────────────────────────────────────────────┘              ",
            "",
            "",
            "",
            "",
            "",
            "",
        ]);
        pretty_assertions::assert_eq!(buffer, expected);
    }

    #[test]
    fn render_scrolling() {
        let mut warning = Warning::from_error_messages(vec![
            String::from("'Twas brillig, and the slithy toves"),
            String::from("Did gyre and gimble in the wabe;"),
            String::from("All mimsy were the borogoves,"),
            String::from("And the mome raths outgrabe."),
            String::from("Beware the Jabberwock, my son!"),
            String::from("The jaws that bite, the claws that catch!"),
            String::from("Beware the Jubjub bird, and shun"),
            String::from("The frumious Bandersnatch!"),
            String::from("He took his vorpal sword in hand:"),
            String::from("Long time the manxome foe he sought--"),
            String::from("So rested he by the Tumtum tree,"),
            String::from("And stood awhile in thought."),
            String::from("And as in uffish thought he stood,"),
            String::from("The Jabberwock, with eyes of flame,"),
            String::from("Came whiffling through the tulgey wood,"),
            String::from("And burbled as it came!"),
            String::from("One, two!  One, two!  And through and through"),
            String::from("The vorpal blade went snicker-snack!"),
            String::from("He left it dead, and with its head"),
            String::from("He went galumping back."),
        ]);
        let area = Rect::new(0, 0, 80, 24);

        let mut buffer = Buffer::empty(area);
        warning.render(area, &mut buffer);
        let expected = Buffer::with_lines([
            "",
            "",
            "             ┌───────────────────── WARNING ──────────────────────┐             ",
            "             │ 'Twas brillig, and the slithy toves              ▲ │             ",
            "             │                                                  █ │             ",
            "             │ Caused by:                                       █ │             ",
            "             │     0: Did gyre and gimble in the wabe;          █ │             ",
            "             │     1: All mimsy were the borogoves,             █ │             ",
            "             │     2: And the mome raths outgrabe.              █ │             ",
            "             │     3: Beware the Jabberwock, my son!            █ │             ",
            "             │     4: The jaws that bite, the claws that catch! █ │             ",
            "             │     5: Beware the Jubjub bird, and shun          █ │             ",
            "             │     6: The frumious Bandersnatch!                █ │             ",
            "             │     7: He took his vorpal sword in hand:         █ │             ",
            "             │     8: Long time the manxome foe he sought--     ▒ │             ",
            "             │     9: So rested he by the Tumtum tree,          ▒ │             ",
            "             │    10: And stood awhile in thought.              ▒ │             ",
            "             │    11: And as in uffish thought he stood,        ▒ │             ",
            "             │    12: The Jabberwock, with eyes of flame,       ▼ │             ",
            "             │                                                    │             ",
            "             │                        [OK]                        │             ",
            "             └────────────────────────────────────────────────────┘             ",
            "",
            "",
        ]);
        pretty_assertions::assert_eq!(buffer, expected);

        assert!(warning.handle_command(Command::Down).is_none());
        let mut buffer = Buffer::empty(area);
        warning.render(area, &mut buffer);
        let expected = Buffer::with_lines([
            "",
            "",
            "             ┌───────────────────── WARNING ──────────────────────┐             ",
            "             │                                                  ▲ │             ",
            "             │ Caused by:                                       ▒ │             ",
            "             │     0: Did gyre and gimble in the wabe;          █ │             ",
            "             │     1: All mimsy were the borogoves,             █ │             ",
            "             │     2: And the mome raths outgrabe.              █ │             ",
            "             │     3: Beware the Jabberwock, my son!            █ │             ",
            "             │     4: The jaws that bite, the claws that catch! █ │             ",
            "             │     5: Beware the Jubjub bird, and shun          █ │             ",
            "             │     6: The frumious Bandersnatch!                █ │             ",
            "             │     7: He took his vorpal sword in hand:         █ │             ",
            "             │     8: Long time the manxome foe he sought--     █ │             ",
            "             │     9: So rested he by the Tumtum tree,          ▒ │             ",
            "             │    10: And stood awhile in thought.              ▒ │             ",
            "             │    11: And as in uffish thought he stood,        ▒ │             ",
            "             │    12: The Jabberwock, with eyes of flame,       ▒ │             ",
            "             │    13: Came whiffling through the tulgey wood,   ▼ │             ",
            "             │                                                    │             ",
            "             │                        [OK]                        │             ",
            "             └────────────────────────────────────────────────────┘             ",
            "",
            "",
        ]);
        pretty_assertions::assert_eq!(buffer, expected);

        for _ in 0..8 {
            assert!(warning.handle_command(Command::Down).is_none());
        }
        let mut buffer = Buffer::empty(area);
        warning.render(area, &mut buffer);
        let expected = Buffer::with_lines([
            "",
            "",
            "             ┌───────────────────── WARNING ──────────────────────┐             ",
            "             │     4: The jaws that bite, the claws that catch! ▲ │             ",
            "             │     5: Beware the Jubjub bird, and shun          ▒ │             ",
            "             │     6: The frumious Bandersnatch!                ▒ │             ",
            "             │     7: He took his vorpal sword in hand:         ▒ │             ",
            "             │     8: Long time the manxome foe he sought--     ▒ │             ",
            "             │     9: So rested he by the Tumtum tree,          █ │             ",
            "             │    10: And stood awhile in thought.              █ │             ",
            "             │    11: And as in uffish thought he stood,        █ │             ",
            "             │    12: The Jabberwock, with eyes of flame,       █ │             ",
            "             │    13: Came whiffling through the tulgey wood,   █ │             ",
            "             │    14: And burbled as it came!                   █ │             ",
            "             │    15: One, two!  One, two!  And through and     █ │             ",
            "             │        through                                   █ │             ",
            "             │    16: The vorpal blade went snicker-snack!      █ │             ",
            "             │    17: He left it dead, and with its head        █ │             ",
            "             │    18: He went galumping back.                   ▼ │             ",
            "             │                                                    │             ",
            "             │                        [OK]                        │             ",
            "             └────────────────────────────────────────────────────┘             ",
            "",
            "",
        ]);
        pretty_assertions::assert_eq!(buffer, expected);

        assert!(warning.handle_command(Command::Down).is_none());
        let mut buffer = Buffer::empty(area);
        warning.render(area, &mut buffer);
        pretty_assertions::assert_eq!(buffer, expected);
    }
}
