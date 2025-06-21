use crate::app::AppState;
use crate::consts;
use crate::game::Game;
use crate::logo::Logo;
use crate::options::{LevelSize, Options};
use crate::util::get_display_area;
use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::{Flex, Layout, Rect},
    style::Style,
    text::{Line, Span, Text},
    widgets::{
        block::{Block, Padding},
        Widget,
    },
    Frame,
};
use std::fmt;
use std::io;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StartupScreen {
    selection: Selection,
    options: OptionsMenu,
}

impl StartupScreen {
    pub(crate) fn new(options: Options) -> Self {
        StartupScreen {
            selection: Selection::default(),
            options: OptionsMenu::new(options),
        }
    }

    pub(crate) fn draw(&self, frame: &mut Frame<'_>) {
        frame.render_widget(self, frame.area());
    }

    pub(crate) fn process_input(&mut self) -> io::Result<Option<AppState>> {
        Ok(self.handle_event(read()?))
    }

    fn handle_event(&mut self, event: Event) -> Option<AppState> {
        let normal_modifiers = KeyModifiers::NONE | KeyModifiers::SHIFT;
        if let Some(KeyEvent {
            code, modifiers, ..
        }) = event.as_key_press_event()
        {
            if (modifiers, code) == (KeyModifiers::CONTROL, KeyCode::Char('c')) {
                return Some(AppState::Quit);
            } else if normal_modifiers.contains(modifiers) {
                match (self.selection, code) {
                    (Selection::NewGameButton, KeyCode::Enter) | (_, KeyCode::Char('n')) => {
                        return Some(AppState::Game(self.new_game()));
                    }
                    (Selection::NewGameButton, KeyCode::Char('s' | 'j') | KeyCode::Down) => {
                        self.selection = Selection::Options;
                        self.options.active = true;
                    }
                    (Selection::Options, KeyCode::Char('w' | 'k') | KeyCode::Up) => {
                        self.selection = self.options.move_up();
                    }
                    (Selection::Options, KeyCode::Char('s' | 'j') | KeyCode::Down) => {
                        self.selection = self.options.move_down();
                    }
                    (Selection::Options, KeyCode::Char('a' | 'h') | KeyCode::Left) => {
                        self.options.move_left();
                    }
                    (Selection::Options, KeyCode::Char('d' | 'l') | KeyCode::Right) => {
                        self.options.move_right();
                    }
                    (Selection::Options, KeyCode::Char(' ') | KeyCode::Enter) => {
                        self.options.toggle();
                    }
                    (Selection::QuitButton, KeyCode::Enter) | (_, KeyCode::Char('q')) => {
                        return Some(AppState::Quit);
                    }
                    (Selection::QuitButton, KeyCode::Char('w' | 'k') | KeyCode::Up) => {
                        self.selection = Selection::Options;
                        self.options.active = true;
                    }
                    _ => (),
                }
            }
        }
        None
    }

    fn new_game(&self) -> Game {
        Game::new(self.options.to_options(), rand::rng())
    }
}

static INSTRUCTIONS: &[&str] = &[
    "Move the snake with:",
    "       ← ↓ ↑ →",
    "   or: h j k l",
    "   or: a s w d",
    "Eat the fruit, but",
    "don't hit yourself!",
];

const INSTRUCTIONS_WIDTH: u16 = 20;
const INSTRUCTIONS_HEIGHT: u16 = 6;

impl Widget for &StartupScreen {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let display = get_display_area(area);
        let [mut logo_area] = Layout::horizontal([Logo::WIDTH])
            .flex(Flex::Center)
            .areas(display);
        logo_area.height = Logo::HEIGHT;
        Logo.render(logo_area, buf);
        let mut y = Logo::HEIGHT + 1;
        let [instructions_area] = Layout::horizontal([INSTRUCTIONS_WIDTH])
            .flex(Flex::Center)
            .areas(Rect {
                y,
                height: INSTRUCTIONS_HEIGHT,
                ..display
            });
        Text::from_iter(INSTRUCTIONS.iter().copied()).render(instructions_area, buf);
        y += INSTRUCTIONS_HEIGHT + 1;

        let ngstyle = if self.selection == Selection::NewGameButton {
            consts::MENU_SELECTION_STYLE
        } else {
            Style::new()
        };
        Line::from(Span::styled("[New Game (n)]", ngstyle))
            .centered()
            .render(
                Rect {
                    y,
                    height: 1,
                    ..display
                },
                buf,
            );
        y += 2;
        let [options_area] = Layout::horizontal([OptionsMenu::WIDTH])
            .flex(Flex::Center)
            .areas(Rect {
                y,
                height: OptionsMenu::HEIGHT,
                ..display
            });
        (&self.options).render(options_area, buf);
        y += OptionsMenu::HEIGHT + 1;
        let qstyle = if self.selection == Selection::QuitButton {
            consts::MENU_SELECTION_STYLE
        } else {
            Style::new()
        };
        Line::from(Span::styled("[Quit (q)]", qstyle))
            .centered()
            .render(
                Rect {
                    y,
                    height: 1,
                    ..display
                },
                buf,
            );
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum Selection {
    #[default]
    NewGameButton,
    Options,
    QuitButton,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct OptionsMenu {
    /// Is the currently-selected startup screen item an element of this menu?
    active: bool,
    /// Index of the currently-selected item in the menu; if the menu isn't
    /// active, this is the index of the most recently-selected item.
    selection: usize,
    settings: [Adjustable; Self::OPTION_QTY],
}

impl OptionsMenu {
    const OPTION_QTY: usize = 4;
    const HEIGHT: u16 = 6; // OPTION_QTY + 2 (for the border)
    const OPTION_LABELS: [&'static str; Self::OPTION_QTY] =
        ["Wraparound", "Obstacles", "Fruits", "Level Size"];
    const HORIZONTAL_PADDING: u16 = 1; // padding on each side
    const POINTER_WIDTH: u16 = 2;
    const LABEL_WIDTH: u16 = 10;
    const LABEL_VALUE_GUTTER: u16 = 2;
    const VALUE_WIDTH: u16 = 10;
    const WIDTH: u16 = 2 /* for border */ + 2 * Self::HORIZONTAL_PADDING + Self::POINTER_WIDTH + Self::LABEL_WIDTH + Self::LABEL_VALUE_GUTTER + Self::VALUE_WIDTH;

    fn new(options: Options) -> Self {
        let settings = [
            Adjustable::Bool(options.wraparound),
            Adjustable::Bool(options.obstacles),
            Adjustable::Fruits(options.fruits),
            Adjustable::LevelSize(options.level_size),
        ];
        OptionsMenu {
            active: false,
            selection: 0,
            settings,
        }
    }

    fn to_options(&self) -> Options {
        let Adjustable::Bool(wraparound) = self.settings[0] else {
            panic!(
                "OptionsMenu.settings[0] should be a Bool; got {:?}",
                self.settings[0]
            );
        };
        let Adjustable::Bool(obstacles) = self.settings[1] else {
            panic!(
                "OptionsMenu.settings[1] should be a Bool; got {:?}",
                self.settings[1]
            );
        };
        let Adjustable::Fruits(fruits) = self.settings[2] else {
            panic!(
                "OptionsMenu.settings[2] should be a Fruits; got {:?}",
                self.settings[2]
            );
        };
        let Adjustable::LevelSize(level_size) = self.settings[3] else {
            panic!(
                "OptionsMenu.settings[3] should be a LevelSize; got {:?}",
                self.settings[3]
            );
        };
        Options {
            wraparound,
            obstacles,
            fruits,
            level_size,
        }
    }

    fn move_up(&mut self) -> Selection {
        if let Some(sel) = self.selection.checked_sub(1) {
            self.selection = sel;
            Selection::Options
        } else {
            self.active = false;
            Selection::NewGameButton
        }
    }

    fn move_down(&mut self) -> Selection {
        let sel = self.selection + 1;
        if sel < self.settings.len() {
            self.selection = sel;
            Selection::Options
        } else {
            self.active = false;
            Selection::QuitButton
        }
    }

    fn move_left(&mut self) {
        self.settings[self.selection].decrease();
    }

    fn move_right(&mut self) {
        self.settings[self.selection].increase();
    }

    fn toggle(&mut self) {
        self.settings[self.selection].toggle();
    }
}

impl Widget for &OptionsMenu {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title(" Options: ")
            .padding(Padding::horizontal(OptionsMenu::HORIZONTAL_PADDING));
        let menu_area = block.inner(area);
        block.render(area, buf);
        for (i, ((label, value), row)) in
            std::iter::zip(OptionsMenu::OPTION_LABELS, self.settings.iter())
                .zip(menu_area.rows())
                .enumerate()
        {
            let selected = self.active && i == self.selection;
            let style = if selected {
                consts::MENU_SELECTION_STYLE
            } else {
                Style::new()
            };
            let s = format!(
                "{pointer:pwidth$}{label:lwidth$}{space:gutter$}{value}",
                pointer = if selected { "»" } else { "" },
                pwidth = usize::from(OptionsMenu::POINTER_WIDTH),
                lwidth = usize::from(OptionsMenu::LABEL_WIDTH),
                space = "",
                gutter = usize::from(OptionsMenu::LABEL_VALUE_GUTTER),
            );
            Span::styled(s, style).render(row, buf);
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Adjustable {
    Bool(bool),
    Fruits(usize),
    LevelSize(LevelSize),
}

impl Adjustable {
    fn increase(&mut self) {
        match self {
            Adjustable::Bool(ref mut b) => *b = true,
            Adjustable::Fruits(ref mut fr) => {
                let new_fruits = fr.saturating_add(1);
                if new_fruits <= consts::MAX_FRUITS {
                    *fr = new_fruits;
                }
            }
            Adjustable::LevelSize(ref mut size) => {
                if let Some(new_size) = size.increase() {
                    *size = new_size;
                }
            }
        }
    }

    fn decrease(&mut self) {
        match self {
            Adjustable::Bool(ref mut b) => *b = false,
            Adjustable::Fruits(ref mut fr) => {
                let new_fruits = fr.saturating_sub(1);
                if new_fruits > 0 {
                    *fr = new_fruits;
                }
            }
            Adjustable::LevelSize(ref mut size) => {
                if let Some(new_size) = size.decrease() {
                    *size = new_size;
                }
            }
        }
    }

    fn toggle(&mut self) {
        if let Adjustable::Bool(ref mut b) = self {
            *b = !*b;
        }
    }
}

impl fmt::Display for Adjustable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Adjustable::Bool(false) => write!(f, "   [ ]    "),
            Adjustable::Bool(true) => write!(f, "   [✓]    "),
            Adjustable::Fruits(fr) => {
                write!(
                    f,
                    "{left} {fr:^6} {right}",
                    left = if fr == 1 { '◁' } else { '◀' },
                    right = if fr == consts::MAX_FRUITS {
                        '▷'
                    } else {
                        '▶'
                    }
                )
            }
            Adjustable::LevelSize(sz) => {
                write!(
                    f,
                    "{left} {sz:6} {right}",
                    left = if sz == LevelSize::MINIMUM {
                        '◁'
                    } else {
                        '◀'
                    },
                    right = if sz == LevelSize::MAXIMUM {
                        '▷'
                    } else {
                        '▶'
                    }
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod startup {
        use super::*;
        use ratatui::{buffer::Buffer, layout::Rect};

        #[test]
        fn draw_initial() {
            let startup = StartupScreen::new(Options::default());
            let area = Rect::new(0, 0, 80, 24);
            let mut buffer = Buffer::empty(area);
            startup.render(area, &mut buffer);
            let mut expected = Buffer::with_lines([
                "                    ____       _   ____              _                          ",
                r"                   |  _ \ __ _| |_/ ___| _ __   __ _| | _____                   ",
                r"                   | |_) / _` | __\___ \| '_ \ / _` | |/ / _ \                  ",
                "                   |  _ < (_| | |_ ___) | | | | (_| |   <  __/                  ",
                r"                   |_| \_\__,_|\__|____/|_| |_|\__,_|_|\_\___|                  ",
                "                                                                                ",
                "                              Move the snake with:                              ",
                "                                     ← ↓ ↑ →                                    ",
                "                                 or: h j k l                                    ",
                "                                 or: a s w d                                    ",
                "                              Eat the fruit, but                                ",
                "                              don't hit yourself!                               ",
                "                                                                                ",
                "                                 [New Game (n)]                                 ",
                "                                                                                ",
                "                          ┌ Options: ────────────────┐                          ",
                "                          │   Wraparound     [ ]     │                          ",
                "                          │   Obstacles      [ ]     │                          ",
                "                          │   Fruits      ◁   1    ▶ │                          ",
                "                          │   Level Size  ◀ Large  ▷ │                          ",
                "                          └──────────────────────────┘                          ",
                "                                                                                ",
                "                                   [Quit (q)]                                   ",
                "                                                                                ",
            ]);
            expected.set_style(Rect::new(19, 0, 15, 5), consts::FRUIT_STYLE);
            expected.set_style(Rect::new(34, 0, 28, 5), consts::SNAKE_STYLE);
            expected.set_style(Rect::new(33, 13, 14, 1), consts::MENU_SELECTION_STYLE);
            assert_eq!(buffer, expected);
        }
    }

    mod options_menu {
        use super::*;

        #[test]
        fn label_width() {
            let actual_width = OptionsMenu::OPTION_LABELS
                .iter()
                .map(|lbl| lbl.len())
                .max()
                .unwrap();
            assert_eq!(actual_width, usize::from(OptionsMenu::LABEL_WIDTH));
        }

        #[test]
        fn roundtrip_defaults() {
            let opts = Options::default();
            let optmenu = OptionsMenu::new(opts);
            assert_eq!(optmenu.to_options(), opts);
        }

        #[test]
        fn roundtrip_custom() {
            let opts = Options {
                wraparound: true,
                obstacles: true,
                fruits: 4,
                level_size: LevelSize::Small,
            };
            let optmenu = OptionsMenu::new(opts);
            assert_eq!(optmenu.to_options(), opts);
        }
    }
}
