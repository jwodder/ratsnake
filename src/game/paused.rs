use crate::command::Command;
use crate::consts;
use crate::util::EnumExt;
use crossterm::event::Event;
use enum_map::Enum;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{
        block::{Block, Padding},
        Widget,
    },
};

/// A widget for displaying a pause menu pop-up
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Paused {
    /// The currently-selected item in the pause menu
    selection: PauseOpt,
}

impl Paused {
    /// The height that should be used for the `Rect` passed to
    /// `Paused::render()`
    pub(super) const HEIGHT: u16 = 6;

    /// The width that should be used for the `Rect` passed to
    /// `Paused::render()`
    pub(super) const WIDTH: u16 = 19;

    /// Create a new `Paused`
    pub(super) fn new() -> Paused {
        Paused {
            selection: PauseOpt::min(),
        }
    }

    /// Handle an input event.  Returns `Some` if the user made a choice.
    pub(super) fn handle_event(&mut self, event: Event) -> Option<PauseOpt> {
        match Command::from_key_event(event.as_key_press_event()?)? {
            Command::Esc => return Some(PauseOpt::Resume),
            Command::R => return Some(PauseOpt::Restart),
            Command::M => return Some(PauseOpt::MainMenu),
            Command::Q | Command::Quit => return Some(PauseOpt::Quit),
            Command::Enter => return Some(self.selection),
            Command::Up => {
                if let Some(opt) = self.selection.prev() {
                    self.selection = opt;
                }
            }
            Command::Down => {
                if let Some(opt) = self.selection.next() {
                    self.selection = opt;
                }
            }
            Command::Next => self.selection = self.selection.next().unwrap_or_else(PauseOpt::min),
            Command::Prev => self.selection = self.selection.prev().unwrap_or_else(PauseOpt::max),
            Command::Home => self.selection = PauseOpt::min(),
            Command::End => self.selection = PauseOpt::max(),
            _ => (),
        }
        None
    }
}

/// The choices in the pause menu
#[derive(Clone, Copy, Debug, Enum, Eq, PartialEq)]
pub(super) enum PauseOpt {
    /// Unpause/resume the game
    Resume,

    /// Start the game over
    Restart,

    /// Return to the main menu
    MainMenu,

    /// Quit the application
    Quit,
}

impl PauseOpt {
    /// Render the option as a `Line` for display in the pause menu.  If
    /// `selected` is `true`, this option is the currently-selected/active one.
    fn to_line(self, selected: bool) -> Line<'static> {
        let mut line = Line::default();
        if selected {
            line.push_span("» ");
        } else {
            line.push_span("  ");
        }
        match self {
            PauseOpt::Resume => {
                line.push_span("Resume (");
                line.push_span(Span::styled("Esc", consts::KEY_STYLE));
                line.push_span(")");
            }
            PauseOpt::Restart => {
                line.push_span("Restart (");
                line.push_span(Span::styled("r", consts::KEY_STYLE));
                line.push_span(")");
            }
            PauseOpt::MainMenu => {
                line.push_span("Main Menu (");
                line.push_span(Span::styled("m", consts::KEY_STYLE));
                line.push_span(")");
            }
            PauseOpt::Quit => {
                line.push_span("Quit (");
                line.push_span(Span::styled("q", consts::KEY_STYLE));
                line.push_span(")");
            }
        }
        if selected {
            line = line.style(consts::MENU_SELECTION_STYLE);
        }
        line
    }
}

impl Widget for Paused {
    /*
     * ┌──── PAUSED ─────┐
     * │ » Resume (Esc)  │
     * │   Restart (r)   │
     * │   Main Menu (m) │
     * │   Quit (q)      │
     * └─────────────────┘
     */

    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title(" PAUSED ")
            .title_alignment(Alignment::Center)
            .padding(Padding::horizontal(1))
            .style(Style::reset());
        let inner = block.inner(area);
        block.render(area, buf);
        for (opt, row) in PauseOpt::iter().zip(inner.rows()) {
            opt.to_line(self.selection == opt).render(row, buf);
        }
    }
}
