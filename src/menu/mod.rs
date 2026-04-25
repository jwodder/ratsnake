mod widgets;
use self::widgets::{Instructions, Logo};
use crate::app::Screen;
use crate::command::Command;
use crate::consts;
use crate::game::Game;
use crate::hstable::HSTable;
use crate::options::{Adjustable, OptKey, OptValue, Options};
use crate::util::{EnumExt, Globals, get_display_area};
use crate::warning::{Warning, WarningOutcome};
use crossterm::event::{Event, read};
use enum_map::{Enum, EnumMap};
use rand::{Rng, RngExt};
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Padding, StatefulWidget, Widget},
};

/// The main menu/startup screen
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MainMenu {
    /// The currently-selected form element
    selection: Selection,

    /// The state of the options sub-menu
    opts_menu: OptionsMenu,

    /// The state that the menu is currently in
    state: MenuState,

    /// Global data (options & high scores)
    globals: Globals,
}

impl MainMenu {
    /// Create a new main menu from the given globals
    pub(crate) fn new(globals: Globals) -> Self {
        MainMenu {
            selection: Selection::default(),
            opts_menu: OptionsMenu::new(globals.options),
            state: MenuState::Normal,
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
        Ok(self.handle_event(read()?))
    }

    /// Handle the given input event.
    ///
    /// Returns `Some(screen)` if the application should switch to a different
    /// screen or quit.
    fn handle_event(&mut self, event: Event) -> Option<Screen> {
        let cmd = Command::from_key_event(event.as_key_press_event()?)?;
        if cmd == Command::Quit {
            return Some(Screen::Quit);
        }
        match self.state {
            MenuState::Normal => match (self.selection, cmd) {
                (_, Command::Home) => self.selection = Selection::min(),
                (_, Command::End) => self.selection = Selection::max(),
                (Selection::PlayButton, Command::Enter) | (_, Command::P) => {
                    let options = self.opts_menu.to_options();
                    self.globals.options = options;
                    match self.globals.config.save_options(options) {
                        Ok(()) => return Some(Screen::Game(self.play())),
                        Err(e) => self.state = MenuState::SaveWarning(Warning::from(e)),
                    }
                }
                (s, Command::Prev) => {
                    self.selection = s.prev().unwrap_or_else(Selection::max);
                }
                (s, Command::Up) => {
                    if let Some(s2) = s.prev() {
                        self.selection = s2;
                    }
                }
                (s, Command::Down) => {
                    if let Some(s2) = s.next() {
                        self.selection = s2;
                    }
                }
                (s, Command::Next) => {
                    self.selection = s.next().unwrap_or_else(Selection::min);
                }
                (Selection::Options(opt), Command::Left) => self.opts_menu.move_left(opt),
                (Selection::Options(opt), Command::Right) => self.opts_menu.move_right(opt),
                (Selection::Options(opt), Command::Space | Command::Enter) => {
                    self.opts_menu.toggle(opt);
                }
                (Selection::Randomize, Command::Enter) | (_, Command::At) => {
                    self.opts_menu.randomize(rand::rng());
                }
                (Selection::HighScores, Command::Enter) | (_, Command::HighScores) => {
                    return Some(Screen::HSTable(HSTable::new(self.globals.clone())));
                }
                (Selection::QuitButton, Command::Enter) | (_, Command::Q) => {
                    return Some(Screen::Quit);
                }
                _ => (),
            },
            MenuState::SaveWarning(ref mut warning) => match warning.handle_command(cmd)? {
                WarningOutcome::Dismissed => return Some(Screen::Game(self.play())),
                WarningOutcome::Quit => return Some(Screen::Quit),
            },
        }
        None
    }

    /// Create a new game
    fn play(&self) -> Game {
        Game::new(self.globals.clone())
    }
}

impl Widget for &MainMenu {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let display = get_display_area(area);
        let [logo_area, main_area] =
            Layout::vertical([Constraint::Length(Logo::HEIGHT), Constraint::Fill(1)])
                .spacing(1)
                .areas(display);

        let [logo_area] = Layout::horizontal([Logo::WIDTH])
            .flex(Flex::Center)
            .areas(logo_area);
        Logo.render(logo_area, buf);

        let [_, main_area, _] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(OptionsMenu::HEIGHT + 6),
            Constraint::Fill(2),
        ])
        .areas(main_area);
        let [form_area, instructions_area] =
            Layout::horizontal([OptionsMenu::WIDTH, Instructions::WIDTH])
                .flex(Flex::SpaceEvenly)
                .areas(main_area);

        let [play_area, options_area, hs_area, quit_area] =
            Layout::vertical([1, OptionsMenu::HEIGHT, 1, 1])
                .flex(Flex::Start)
                .spacing(1)
                .areas(form_area);

        let play_style = if self.selection == Selection::PlayButton {
            consts::MENU_SELECTION_STYLE
        } else {
            Style::new()
        };
        Line::from_iter([
            Span::styled("[Play (", play_style),
            Span::styled("p", consts::KEY_STYLE.patch(play_style)),
            Span::styled(")]", play_style),
        ])
        .centered()
        .render(play_area, buf);

        let [options_area] = Layout::horizontal([OptionsMenu::WIDTH])
            .flex(Flex::Center)
            .areas(options_area);
        let mut opts_state = self.selection;
        (&self.opts_menu).render(options_area, buf, &mut opts_state);

        let hsstyle = if self.selection == Selection::HighScores {
            consts::MENU_SELECTION_STYLE
        } else {
            Style::new()
        };
        Line::from_iter([
            Span::styled("[High Scores (", hsstyle),
            Span::styled("H", consts::KEY_STYLE.patch(hsstyle)),
            Span::styled(")]", hsstyle),
        ])
        .centered()
        .render(hs_area, buf);

        let qstyle = if self.selection == Selection::QuitButton {
            consts::MENU_SELECTION_STYLE
        } else {
            Style::new()
        };
        Line::from_iter([
            Span::styled("[Quit (", qstyle),
            Span::styled("q", consts::KEY_STYLE.patch(qstyle)),
            Span::styled(")]", qstyle),
        ])
        .centered()
        .render(quit_area, buf);

        let [instructions_area] = Layout::vertical([Instructions::HEIGHT])
            .flex(Flex::Center)
            .areas(instructions_area);
        Instructions.render(instructions_area, buf);

        if let MenuState::SaveWarning(warning) = &self.state {
            warning.render(display, buf);
        }
    }
}

/// An enum of the states that the main menu can be in
#[derive(Clone, Debug, Eq, PartialEq)]
enum MenuState {
    /// Normal operation
    Normal,

    /// A warning is being displayed about failure to save the chosen options
    /// to a file.
    ///
    /// Because options are only saved when the user selects to play a game,
    /// after this warning is dismissed, the application will transition to a
    /// new game.
    SaveWarning(Warning),
}

/// An enum of the form elements
#[derive(Clone, Copy, Debug, Default, Enum, Eq, PartialEq)]
enum Selection {
    /// The "[Play (p)]" button
    #[default]
    PlayButton,

    /// The options sub-menu
    Options(OptKey),

    /// The "[Randomize (&#x41;)]" button
    Randomize,

    /// The "[High Scores (H)]" button
    HighScores,

    /// The "[Quit (q)]" button
    QuitButton,
}

/// State of the options sub-menu
#[derive(Clone, Debug, Eq, PartialEq)]
struct OptionsMenu {
    /// Option values currently displayed in the submenu
    settings: EnumMap<OptKey, OptValue>,
}

impl OptionsMenu {
    /// The height that should be used for the `Rect` passed to
    /// `&OptionsMenu::render()`
    #[allow(clippy::cast_possible_truncation)]
    const HEIGHT: u16 = (OptKey::LENGTH as u16) + 2 /* for border */ + 2 /* for "Randomize" button */;

    /// The width of the horizontal padding on each inner side of the menu
    /// border
    const HORIZONTAL_PADDING: u16 = 1;

    /// The number of total display column cells used by the menu pointer
    const POINTER_WIDTH: u16 = 2;

    /// The number of display column cells between the option names and values
    const LABEL_VALUE_GUTTER: u16 = 2;

    /// The width that should be used for the `Rect` passed to
    /// `&OptionsMenu::render()`
    const WIDTH: u16 = 2 /* for border */ + 2 * Self::HORIZONTAL_PADDING + Self::POINTER_WIDTH + OptKey::DISPLAY_WIDTH + Self::LABEL_VALUE_GUTTER + OptValue::DISPLAY_WIDTH;

    /// Create a new `OptionsMenu` with the given `Options` as the initial
    /// values
    fn new(options: Options) -> Self {
        let settings = EnumMap::from_iter(OptKey::iter().map(|key| (key, options.get(key))));
        OptionsMenu { settings }
    }

    /// Return the `Options` currently selected in the menu
    fn to_options(&self) -> Options {
        let mut opts = Options::default();
        for key in OptKey::iter() {
            opts.set(key, self.settings[key]);
        }
        opts
    }

    /// Respond to a "Left" input by decreasing or unsetting the given
    /// option, if possible
    fn move_left(&mut self, opt: OptKey) {
        self.settings[opt].decrease();
    }

    /// Respond to a "Right" input by increasing or setting the given
    /// option, if possible
    fn move_right(&mut self, opt: OptKey) {
        self.settings[opt].increase();
    }

    /// Toggle the given option, if possible
    fn toggle(&mut self, opt: OptKey) {
        self.settings[opt].toggle();
    }

    fn randomize<R: Rng>(&mut self, mut rng: R) {
        self.settings = EnumMap::from_fn(|key| match key {
            OptKey::Wraparound => OptValue::Bool(rng.random()),
            OptKey::Obstacles => OptValue::Bool(rng.random()),
            OptKey::Fruits => OptValue::FruitQty(rng.random()),
            OptKey::LevelSize => OptValue::LevelSize(rng.random()),
        });
    }
}

impl StatefulWidget for &OptionsMenu {
    type State = Selection;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Selection) {
        let block = Block::bordered()
            .title(" Options: ")
            .padding(Padding::horizontal(OptionsMenu::HORIZONTAL_PADDING));
        let menu_area = block.inner(area);
        block.render(area, buf);
        for ((key, value), row) in OptKey::iter()
            .map(|key| (key, self.settings[key]))
            .zip(menu_area.rows())
        {
            let selected = *state == Selection::Options(key);
            let style = if selected {
                consts::MENU_SELECTION_STYLE
            } else {
                Style::new()
            };
            let s = format!(
                "{pointer:pwidth$}{key:lwidth$}{space:gutter$}{value}",
                pointer = if selected { "»" } else { "" },
                pwidth = usize::from(OptionsMenu::POINTER_WIDTH),
                lwidth = usize::from(OptKey::DISPLAY_WIDTH),
                space = "",
                gutter = usize::from(OptionsMenu::LABEL_VALUE_GUTTER),
            );
            Span::styled(s, style).render(row, buf);
        }
        if let Some(rand_row) = menu_area.rows().next_back() {
            let rand_style = if *state == Selection::Randomize {
                consts::MENU_SELECTION_STYLE
            } else {
                Style::new()
            };
            Line::from_iter([
                Span::styled("[Randomize (", rand_style),
                Span::styled("@", consts::KEY_STYLE.patch(rand_style)),
                Span::styled(")]", rand_style),
            ])
            .centered()
            .render(rand_row, buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod main_menu {
        use super::*;
        use crossterm::event::KeyCode;
        use ratatui::{buffer::Buffer, layout::Rect};

        #[test]
        fn draw_initial() {
            let menu = MainMenu::new(Globals::default());
            let area = Rect::new(0, 0, 80, 24);
            let mut buffer = Buffer::empty(area);
            menu.render(area, &mut buffer);
            #[rustfmt::skip]
            let mut expected = Buffer::with_lines([
                 "                    ____       _   ____              _                          ",
                r"                   |  _ \ __ _| |_/ ___| _ __   __ _| | _____                   ",
                r"                   | |_) / _` | __\___ \| '_ \ / _` | |/ / _ \                  ",
                 "                   |  _ < (_| | |_ ___) | | | | (_| |   <  __/                  ",
                r"                   |_| \_\__,_|\__|____/|_| |_|\__,_|_|\_\___|                  ",
                 "                                                                                ",
                 "                                 ⚬⚬⚬⚬⚬⚬⚬⚬⚬⚬⚬⚬<  ●                               ",
                 "                                                                                ",
                 "                                                                                ",
                 "                    [Play (p)]                                                  ",
                 "                                                                                ",
                 "           ┌ Options: ────────────────┐                                         ",
                 "           │   Wraparound     [ ]     │                                         ",
                 "           │   Obstacles      [ ]     │          Move the snake with:           ",
                 "           │   Fruits      ◁   1    ▶ │                 ← ↓ ↑ →                 ",
                 "           │   Level Size  ◀ Large  ▷ │             or: h j k l                 ",
                 "           │                          │             or: a s w d                 ",
                 "           │     [Randomize (@)]      │             or: 4 2 8 6                 ",
                 "           └──────────────────────────┘          Eat the fruit, but             ",
                 "                                                 don't hit yourself!            ",
                 "                [High Scores (H)]                                               ",
                 "                                                                                ",
                 "                    [Quit (q)]                                                  ",
                 "                                                                                ",
            ]);
            expected.set_style(Rect::new(19, 0, 15, 5), consts::FRUIT_STYLE); // "Rat"
            expected.set_style(Rect::new(34, 0, 28, 5), consts::SNAKE_STYLE); // "Snake"
            expected.set_style(Rect::new(33, 6, 13, 1), consts::SNAKE_STYLE); // ⚬⚬…⚬<
            expected.set_style(Rect::new(48, 6, 1, 1), consts::FRUIT_STYLE); // fruit in logo
            expected.set_style(Rect::new(27, 9, 1, 1), consts::KEY_STYLE); // `p`
            expected.set_style(Rect::new(20, 9, 10, 1), consts::MENU_SELECTION_STYLE); // Play button
            expected.set_style(Rect::new(29, 17, 1, 1), consts::KEY_STYLE); // `@`
            expected.set_style(Rect::new(30, 20, 1, 1), consts::KEY_STYLE); // `H`
            expected.set_style(Rect::new(27, 22, 1, 1), consts::KEY_STYLE); // `q`
            expected.set_style(Rect::new(56, 14, 1, 1), consts::KEY_STYLE); // `←`
            expected.set_style(Rect::new(58, 14, 1, 1), consts::KEY_STYLE); // `↓`
            expected.set_style(Rect::new(60, 14, 1, 1), consts::KEY_STYLE); // `↑`
            expected.set_style(Rect::new(62, 14, 1, 1), consts::KEY_STYLE); // `→`
            expected.set_style(Rect::new(56, 15, 1, 1), consts::KEY_STYLE); // `h`
            expected.set_style(Rect::new(58, 15, 1, 1), consts::KEY_STYLE); // `j`
            expected.set_style(Rect::new(60, 15, 1, 1), consts::KEY_STYLE); // `k`
            expected.set_style(Rect::new(62, 15, 1, 1), consts::KEY_STYLE); // `l`
            expected.set_style(Rect::new(56, 16, 1, 1), consts::KEY_STYLE); // `a`
            expected.set_style(Rect::new(58, 16, 1, 1), consts::KEY_STYLE); // `s`
            expected.set_style(Rect::new(60, 16, 1, 1), consts::KEY_STYLE); // `w`
            expected.set_style(Rect::new(62, 16, 1, 1), consts::KEY_STYLE); // `s`
            expected.set_style(Rect::new(56, 17, 1, 1), consts::KEY_STYLE); // `4`
            expected.set_style(Rect::new(58, 17, 1, 1), consts::KEY_STYLE); // `2`
            expected.set_style(Rect::new(60, 17, 1, 1), consts::KEY_STYLE); // `8`
            expected.set_style(Rect::new(62, 17, 1, 1), consts::KEY_STYLE); // `6`
            pretty_assertions::assert_eq!(buffer, expected);
        }

        #[test]
        fn interact_options() {
            let area = Rect::new(0, 0, 80, 24);
            let mut menu = MainMenu::new(Globals::default());
            assert!(
                menu.handle_event(Event::Key(KeyCode::Down.into()))
                    .is_none()
            );
            let mut buffer = Buffer::empty(area);
            menu.render(area, &mut buffer);
            #[rustfmt::skip]
            let mut expected = Buffer::with_lines([
                 "                    ____       _   ____              _                          ",
                r"                   |  _ \ __ _| |_/ ___| _ __   __ _| | _____                   ",
                r"                   | |_) / _` | __\___ \| '_ \ / _` | |/ / _ \                  ",
                 "                   |  _ < (_| | |_ ___) | | | | (_| |   <  __/                  ",
                r"                   |_| \_\__,_|\__|____/|_| |_|\__,_|_|\_\___|                  ",
                 "                                                                                ",
                 "                                 ⚬⚬⚬⚬⚬⚬⚬⚬⚬⚬⚬⚬<  ●                               ",
                 "                                                                                ",
                 "                                                                                ",
                 "                    [Play (p)]                                                  ",
                 "                                                                                ",
                 "           ┌ Options: ────────────────┐                                         ",
                 "           │ » Wraparound     [ ]     │                                         ",
                 "           │   Obstacles      [ ]     │          Move the snake with:           ",
                 "           │   Fruits      ◁   1    ▶ │                 ← ↓ ↑ →                 ",
                 "           │   Level Size  ◀ Large  ▷ │             or: h j k l                 ",
                 "           │                          │             or: a s w d                 ",
                 "           │     [Randomize (@)]      │             or: 4 2 8 6                 ",
                 "           └──────────────────────────┘          Eat the fruit, but             ",
                 "                                                 don't hit yourself!            ",
                 "                [High Scores (H)]                                               ",
                 "                                                                                ",
                 "                    [Quit (q)]                                                  ",
                 "                                                                                ",
            ]);
            expected.set_style(Rect::new(19, 0, 15, 5), consts::FRUIT_STYLE); // "Rat"
            expected.set_style(Rect::new(34, 0, 28, 5), consts::SNAKE_STYLE); // "Snake"
            expected.set_style(Rect::new(33, 6, 13, 1), consts::SNAKE_STYLE); // ⚬⚬…⚬<
            expected.set_style(Rect::new(48, 6, 1, 1), consts::FRUIT_STYLE); // fruit in logo
            expected.set_style(Rect::new(27, 9, 1, 1), consts::KEY_STYLE); // `p`
            expected.set_style(Rect::new(13, 12, 24, 1), consts::MENU_SELECTION_STYLE); // "Wraparound" option
            expected.set_style(Rect::new(29, 17, 1, 1), consts::KEY_STYLE); // `@`
            expected.set_style(Rect::new(30, 20, 1, 1), consts::KEY_STYLE); // `H`
            expected.set_style(Rect::new(27, 22, 1, 1), consts::KEY_STYLE); // `q`
            expected.set_style(Rect::new(56, 14, 1, 1), consts::KEY_STYLE); // `←`
            expected.set_style(Rect::new(58, 14, 1, 1), consts::KEY_STYLE); // `↓`
            expected.set_style(Rect::new(60, 14, 1, 1), consts::KEY_STYLE); // `↑`
            expected.set_style(Rect::new(62, 14, 1, 1), consts::KEY_STYLE); // `→`
            expected.set_style(Rect::new(56, 15, 1, 1), consts::KEY_STYLE); // `h`
            expected.set_style(Rect::new(58, 15, 1, 1), consts::KEY_STYLE); // `j`
            expected.set_style(Rect::new(60, 15, 1, 1), consts::KEY_STYLE); // `k`
            expected.set_style(Rect::new(62, 15, 1, 1), consts::KEY_STYLE); // `l`
            expected.set_style(Rect::new(56, 16, 1, 1), consts::KEY_STYLE); // `a`
            expected.set_style(Rect::new(58, 16, 1, 1), consts::KEY_STYLE); // `s`
            expected.set_style(Rect::new(60, 16, 1, 1), consts::KEY_STYLE); // `w`
            expected.set_style(Rect::new(62, 16, 1, 1), consts::KEY_STYLE); // `s`
            expected.set_style(Rect::new(56, 17, 1, 1), consts::KEY_STYLE); // `4`
            expected.set_style(Rect::new(58, 17, 1, 1), consts::KEY_STYLE); // `2`
            expected.set_style(Rect::new(60, 17, 1, 1), consts::KEY_STYLE); // `8`
            expected.set_style(Rect::new(62, 17, 1, 1), consts::KEY_STYLE); // `6`
            pretty_assertions::assert_eq!(buffer, expected);

            assert!(
                menu.handle_event(Event::Key(KeyCode::Char(' ').into()))
                    .is_none()
            );
            let mut buffer = Buffer::empty(area);
            menu.render(area, &mut buffer);
            #[rustfmt::skip]
            let mut expected = Buffer::with_lines([
                 "                    ____       _   ____              _                          ",
                r"                   |  _ \ __ _| |_/ ___| _ __   __ _| | _____                   ",
                r"                   | |_) / _` | __\___ \| '_ \ / _` | |/ / _ \                  ",
                 "                   |  _ < (_| | |_ ___) | | | | (_| |   <  __/                  ",
                r"                   |_| \_\__,_|\__|____/|_| |_|\__,_|_|\_\___|                  ",
                 "                                                                                ",
                 "                                 ⚬⚬⚬⚬⚬⚬⚬⚬⚬⚬⚬⚬<  ●                               ",
                 "                                                                                ",
                 "                                                                                ",
                 "                    [Play (p)]                                                  ",
                 "                                                                                ",
                 "           ┌ Options: ────────────────┐                                         ",
                 "           │ » Wraparound     [✓]     │                                         ",
                 "           │   Obstacles      [ ]     │          Move the snake with:           ",
                 "           │   Fruits      ◁   1    ▶ │                 ← ↓ ↑ →                 ",
                 "           │   Level Size  ◀ Large  ▷ │             or: h j k l                 ",
                 "           │                          │             or: a s w d                 ",
                 "           │     [Randomize (@)]      │             or: 4 2 8 6                 ",
                 "           └──────────────────────────┘          Eat the fruit, but             ",
                 "                                                 don't hit yourself!            ",
                 "                [High Scores (H)]                                               ",
                 "                                                                                ",
                 "                    [Quit (q)]                                                  ",
                 "                                                                                ",
            ]);
            expected.set_style(Rect::new(19, 0, 15, 5), consts::FRUIT_STYLE); // "Rat"
            expected.set_style(Rect::new(34, 0, 28, 5), consts::SNAKE_STYLE); // "Snake"
            expected.set_style(Rect::new(33, 6, 13, 1), consts::SNAKE_STYLE); // ⚬⚬…⚬<
            expected.set_style(Rect::new(48, 6, 1, 1), consts::FRUIT_STYLE); // fruit in logo
            expected.set_style(Rect::new(27, 9, 1, 1), consts::KEY_STYLE); // `p`
            expected.set_style(Rect::new(13, 12, 24, 1), consts::MENU_SELECTION_STYLE); // "Wraparound" option
            expected.set_style(Rect::new(29, 17, 1, 1), consts::KEY_STYLE); // `@`
            expected.set_style(Rect::new(30, 20, 1, 1), consts::KEY_STYLE); // `H`
            expected.set_style(Rect::new(27, 22, 1, 1), consts::KEY_STYLE); // `q`
            expected.set_style(Rect::new(56, 14, 1, 1), consts::KEY_STYLE); // `←`
            expected.set_style(Rect::new(58, 14, 1, 1), consts::KEY_STYLE); // `↓`
            expected.set_style(Rect::new(60, 14, 1, 1), consts::KEY_STYLE); // `↑`
            expected.set_style(Rect::new(62, 14, 1, 1), consts::KEY_STYLE); // `→`
            expected.set_style(Rect::new(56, 15, 1, 1), consts::KEY_STYLE); // `h`
            expected.set_style(Rect::new(58, 15, 1, 1), consts::KEY_STYLE); // `j`
            expected.set_style(Rect::new(60, 15, 1, 1), consts::KEY_STYLE); // `k`
            expected.set_style(Rect::new(62, 15, 1, 1), consts::KEY_STYLE); // `l`
            expected.set_style(Rect::new(56, 16, 1, 1), consts::KEY_STYLE); // `a`
            expected.set_style(Rect::new(58, 16, 1, 1), consts::KEY_STYLE); // `s`
            expected.set_style(Rect::new(60, 16, 1, 1), consts::KEY_STYLE); // `w`
            expected.set_style(Rect::new(62, 16, 1, 1), consts::KEY_STYLE); // `s`
            expected.set_style(Rect::new(56, 17, 1, 1), consts::KEY_STYLE); // `4`
            expected.set_style(Rect::new(58, 17, 1, 1), consts::KEY_STYLE); // `2`
            expected.set_style(Rect::new(60, 17, 1, 1), consts::KEY_STYLE); // `8`
            expected.set_style(Rect::new(62, 17, 1, 1), consts::KEY_STYLE); // `6`
            pretty_assertions::assert_eq!(buffer, expected);

            assert!(
                menu.handle_event(Event::Key(KeyCode::Down.into()))
                    .is_none()
            );
            assert!(
                menu.handle_event(Event::Key(KeyCode::Down.into()))
                    .is_none()
            );
            assert!(
                menu.handle_event(Event::Key(KeyCode::Down.into()))
                    .is_none()
            );
            assert!(
                menu.handle_event(Event::Key(KeyCode::Char(' ').into()))
                    .is_none()
            );
            let mut buffer = Buffer::empty(area);
            menu.render(area, &mut buffer);
            #[rustfmt::skip]
            let mut expected = Buffer::with_lines([
                 "                    ____       _   ____              _                          ",
                r"                   |  _ \ __ _| |_/ ___| _ __   __ _| | _____                   ",
                r"                   | |_) / _` | __\___ \| '_ \ / _` | |/ / _ \                  ",
                 "                   |  _ < (_| | |_ ___) | | | | (_| |   <  __/                  ",
                r"                   |_| \_\__,_|\__|____/|_| |_|\__,_|_|\_\___|                  ",
                 "                                                                                ",
                 "                                 ⚬⚬⚬⚬⚬⚬⚬⚬⚬⚬⚬⚬<  ●                               ",
                 "                                                                                ",
                 "                                                                                ",
                 "                    [Play (p)]                                                  ",
                 "                                                                                ",
                 "           ┌ Options: ────────────────┐                                         ",
                 "           │   Wraparound     [✓]     │                                         ",
                 "           │   Obstacles      [ ]     │          Move the snake with:           ",
                 "           │   Fruits      ◁   1    ▶ │                 ← ↓ ↑ →                 ",
                 "           │ » Level Size  ◀ Large  ▷ │             or: h j k l                 ",
                 "           │                          │             or: a s w d                 ",
                 "           │     [Randomize (@)]      │             or: 4 2 8 6                 ",
                 "           └──────────────────────────┘          Eat the fruit, but             ",
                 "                                                 don't hit yourself!            ",
                 "                [High Scores (H)]                                               ",
                 "                                                                                ",
                 "                    [Quit (q)]                                                  ",
                 "                                                                                ",
            ]);
            expected.set_style(Rect::new(19, 0, 15, 5), consts::FRUIT_STYLE); // "Rat"
            expected.set_style(Rect::new(34, 0, 28, 5), consts::SNAKE_STYLE); // "Snake"
            expected.set_style(Rect::new(33, 6, 13, 1), consts::SNAKE_STYLE); // ⚬⚬…⚬<
            expected.set_style(Rect::new(48, 6, 1, 1), consts::FRUIT_STYLE); // fruit in logo
            expected.set_style(Rect::new(27, 9, 1, 1), consts::KEY_STYLE); // `p`
            expected.set_style(Rect::new(13, 15, 24, 1), consts::MENU_SELECTION_STYLE); // "Level Size" option
            expected.set_style(Rect::new(29, 17, 1, 1), consts::KEY_STYLE); // `@`
            expected.set_style(Rect::new(30, 20, 1, 1), consts::KEY_STYLE); // `H`
            expected.set_style(Rect::new(27, 22, 1, 1), consts::KEY_STYLE); // `q`
            expected.set_style(Rect::new(56, 14, 1, 1), consts::KEY_STYLE); // `←`
            expected.set_style(Rect::new(58, 14, 1, 1), consts::KEY_STYLE); // `↓`
            expected.set_style(Rect::new(60, 14, 1, 1), consts::KEY_STYLE); // `↑`
            expected.set_style(Rect::new(62, 14, 1, 1), consts::KEY_STYLE); // `→`
            expected.set_style(Rect::new(56, 15, 1, 1), consts::KEY_STYLE); // `h`
            expected.set_style(Rect::new(58, 15, 1, 1), consts::KEY_STYLE); // `j`
            expected.set_style(Rect::new(60, 15, 1, 1), consts::KEY_STYLE); // `k`
            expected.set_style(Rect::new(62, 15, 1, 1), consts::KEY_STYLE); // `l`
            expected.set_style(Rect::new(56, 16, 1, 1), consts::KEY_STYLE); // `a`
            expected.set_style(Rect::new(58, 16, 1, 1), consts::KEY_STYLE); // `s`
            expected.set_style(Rect::new(60, 16, 1, 1), consts::KEY_STYLE); // `w`
            expected.set_style(Rect::new(62, 16, 1, 1), consts::KEY_STYLE); // `s`
            expected.set_style(Rect::new(56, 17, 1, 1), consts::KEY_STYLE); // `4`
            expected.set_style(Rect::new(58, 17, 1, 1), consts::KEY_STYLE); // `2`
            expected.set_style(Rect::new(60, 17, 1, 1), consts::KEY_STYLE); // `8`
            expected.set_style(Rect::new(62, 17, 1, 1), consts::KEY_STYLE); // `6`
            pretty_assertions::assert_eq!(buffer, expected);

            assert!(
                menu.handle_event(Event::Key(KeyCode::Left.into()))
                    .is_none()
            );
            let mut buffer = Buffer::empty(area);
            menu.render(area, &mut buffer);
            #[rustfmt::skip]
            let mut expected = Buffer::with_lines([
                 "                    ____       _   ____              _                          ",
                r"                   |  _ \ __ _| |_/ ___| _ __   __ _| | _____                   ",
                r"                   | |_) / _` | __\___ \| '_ \ / _` | |/ / _ \                  ",
                 "                   |  _ < (_| | |_ ___) | | | | (_| |   <  __/                  ",
                r"                   |_| \_\__,_|\__|____/|_| |_|\__,_|_|\_\___|                  ",
                 "                                                                                ",
                 "                                 ⚬⚬⚬⚬⚬⚬⚬⚬⚬⚬⚬⚬<  ●                               ",
                 "                                                                                ",
                 "                                                                                ",
                 "                    [Play (p)]                                                  ",
                 "                                                                                ",
                 "           ┌ Options: ────────────────┐                                         ",
                 "           │   Wraparound     [✓]     │                                         ",
                 "           │   Obstacles      [ ]     │          Move the snake with:           ",
                 "           │   Fruits      ◁   1    ▶ │                 ← ↓ ↑ →                 ",
                 "           │ » Level Size  ◀ Medium ▶ │             or: h j k l                 ",
                 "           │                          │             or: a s w d                 ",
                 "           │     [Randomize (@)]      │             or: 4 2 8 6                 ",
                 "           └──────────────────────────┘          Eat the fruit, but             ",
                 "                                                 don't hit yourself!            ",
                 "                [High Scores (H)]                                               ",
                 "                                                                                ",
                 "                    [Quit (q)]                                                  ",
                 "                                                                                ",
            ]);
            expected.set_style(Rect::new(19, 0, 15, 5), consts::FRUIT_STYLE); // "Rat"
            expected.set_style(Rect::new(34, 0, 28, 5), consts::SNAKE_STYLE); // "Snake"
            expected.set_style(Rect::new(33, 6, 13, 1), consts::SNAKE_STYLE); // ⚬⚬…⚬<
            expected.set_style(Rect::new(48, 6, 1, 1), consts::FRUIT_STYLE); // fruit in logo
            expected.set_style(Rect::new(27, 9, 1, 1), consts::KEY_STYLE); // `p`
            expected.set_style(Rect::new(13, 15, 24, 1), consts::MENU_SELECTION_STYLE); // "Level Size" option
            expected.set_style(Rect::new(29, 17, 1, 1), consts::KEY_STYLE); // `@`
            expected.set_style(Rect::new(30, 20, 1, 1), consts::KEY_STYLE); // `H`
            expected.set_style(Rect::new(27, 22, 1, 1), consts::KEY_STYLE); // `q`
            expected.set_style(Rect::new(56, 14, 1, 1), consts::KEY_STYLE); // `←`
            expected.set_style(Rect::new(58, 14, 1, 1), consts::KEY_STYLE); // `↓`
            expected.set_style(Rect::new(60, 14, 1, 1), consts::KEY_STYLE); // `↑`
            expected.set_style(Rect::new(62, 14, 1, 1), consts::KEY_STYLE); // `→`
            expected.set_style(Rect::new(56, 15, 1, 1), consts::KEY_STYLE); // `h`
            expected.set_style(Rect::new(58, 15, 1, 1), consts::KEY_STYLE); // `j`
            expected.set_style(Rect::new(60, 15, 1, 1), consts::KEY_STYLE); // `k`
            expected.set_style(Rect::new(62, 15, 1, 1), consts::KEY_STYLE); // `l`
            expected.set_style(Rect::new(56, 16, 1, 1), consts::KEY_STYLE); // `a`
            expected.set_style(Rect::new(58, 16, 1, 1), consts::KEY_STYLE); // `s`
            expected.set_style(Rect::new(60, 16, 1, 1), consts::KEY_STYLE); // `w`
            expected.set_style(Rect::new(62, 16, 1, 1), consts::KEY_STYLE); // `s`
            expected.set_style(Rect::new(56, 17, 1, 1), consts::KEY_STYLE); // `4`
            expected.set_style(Rect::new(58, 17, 1, 1), consts::KEY_STYLE); // `2`
            expected.set_style(Rect::new(60, 17, 1, 1), consts::KEY_STYLE); // `8`
            expected.set_style(Rect::new(62, 17, 1, 1), consts::KEY_STYLE); // `6`
            pretty_assertions::assert_eq!(buffer, expected);

            assert!(
                menu.handle_event(Event::Key(KeyCode::Left.into()))
                    .is_none()
            );
            let mut buffer = Buffer::empty(area);
            menu.render(area, &mut buffer);
            #[rustfmt::skip]
            let mut expected = Buffer::with_lines([
                 "                    ____       _   ____              _                          ",
                r"                   |  _ \ __ _| |_/ ___| _ __   __ _| | _____                   ",
                r"                   | |_) / _` | __\___ \| '_ \ / _` | |/ / _ \                  ",
                 "                   |  _ < (_| | |_ ___) | | | | (_| |   <  __/                  ",
                r"                   |_| \_\__,_|\__|____/|_| |_|\__,_|_|\_\___|                  ",
                 "                                                                                ",
                 "                                 ⚬⚬⚬⚬⚬⚬⚬⚬⚬⚬⚬⚬<  ●                               ",
                 "                                                                                ",
                 "                                                                                ",
                 "                    [Play (p)]                                                  ",
                 "                                                                                ",
                 "           ┌ Options: ────────────────┐                                         ",
                 "           │   Wraparound     [✓]     │                                         ",
                 "           │   Obstacles      [ ]     │          Move the snake with:           ",
                 "           │   Fruits      ◁   1    ▶ │                 ← ↓ ↑ →                 ",
                 "           │ » Level Size  ◁ Small  ▶ │             or: h j k l                 ",
                 "           │                          │             or: a s w d                 ",
                 "           │     [Randomize (@)]      │             or: 4 2 8 6                 ",
                 "           └──────────────────────────┘          Eat the fruit, but             ",
                 "                                                 don't hit yourself!            ",
                 "                [High Scores (H)]                                               ",
                 "                                                                                ",
                 "                    [Quit (q)]                                                  ",
                 "                                                                                ",
            ]);
            expected.set_style(Rect::new(19, 0, 15, 5), consts::FRUIT_STYLE); // "Rat"
            expected.set_style(Rect::new(34, 0, 28, 5), consts::SNAKE_STYLE); // "Snake"
            expected.set_style(Rect::new(33, 6, 13, 1), consts::SNAKE_STYLE); // ⚬⚬…⚬<
            expected.set_style(Rect::new(48, 6, 1, 1), consts::FRUIT_STYLE); // fruit in logo
            expected.set_style(Rect::new(27, 9, 1, 1), consts::KEY_STYLE); // `p`
            expected.set_style(Rect::new(13, 15, 24, 1), consts::MENU_SELECTION_STYLE); // "Level Size" option
            expected.set_style(Rect::new(29, 17, 1, 1), consts::KEY_STYLE); // `@`
            expected.set_style(Rect::new(30, 20, 1, 1), consts::KEY_STYLE); // `H`
            expected.set_style(Rect::new(27, 22, 1, 1), consts::KEY_STYLE); // `q`
            expected.set_style(Rect::new(56, 14, 1, 1), consts::KEY_STYLE); // `←`
            expected.set_style(Rect::new(58, 14, 1, 1), consts::KEY_STYLE); // `↓`
            expected.set_style(Rect::new(60, 14, 1, 1), consts::KEY_STYLE); // `↑`
            expected.set_style(Rect::new(62, 14, 1, 1), consts::KEY_STYLE); // `→`
            expected.set_style(Rect::new(56, 15, 1, 1), consts::KEY_STYLE); // `h`
            expected.set_style(Rect::new(58, 15, 1, 1), consts::KEY_STYLE); // `j`
            expected.set_style(Rect::new(60, 15, 1, 1), consts::KEY_STYLE); // `k`
            expected.set_style(Rect::new(62, 15, 1, 1), consts::KEY_STYLE); // `l`
            expected.set_style(Rect::new(56, 16, 1, 1), consts::KEY_STYLE); // `a`
            expected.set_style(Rect::new(58, 16, 1, 1), consts::KEY_STYLE); // `s`
            expected.set_style(Rect::new(60, 16, 1, 1), consts::KEY_STYLE); // `w`
            expected.set_style(Rect::new(62, 16, 1, 1), consts::KEY_STYLE); // `s`
            expected.set_style(Rect::new(56, 17, 1, 1), consts::KEY_STYLE); // `4`
            expected.set_style(Rect::new(58, 17, 1, 1), consts::KEY_STYLE); // `2`
            expected.set_style(Rect::new(60, 17, 1, 1), consts::KEY_STYLE); // `8`
            expected.set_style(Rect::new(62, 17, 1, 1), consts::KEY_STYLE); // `6`
            pretty_assertions::assert_eq!(buffer, expected);
        }

        /// Test that tabbing to the end of the options menu and then tabbing
        /// again until you loop back around to the options menu puts you at
        /// the start of the options.
        #[test]
        fn tab_wraparound() {
            let mut menu = MainMenu::new(Globals::default());
            assert_eq!(menu.selection, Selection::PlayButton);
            for _ in OptKey::iter() {
                assert!(menu.handle_event(Event::Key(KeyCode::Tab.into())).is_none());
            }
            assert_eq!(menu.selection, Selection::Options(OptKey::max()));
            assert!(menu.handle_event(Event::Key(KeyCode::Tab.into())).is_none());
            assert_eq!(menu.selection, Selection::Randomize);
            assert!(menu.handle_event(Event::Key(KeyCode::Tab.into())).is_none());
            assert!(menu.handle_event(Event::Key(KeyCode::Tab.into())).is_none());
            assert!(menu.handle_event(Event::Key(KeyCode::Tab.into())).is_none());
            assert!(menu.handle_event(Event::Key(KeyCode::Tab.into())).is_none());
            assert_eq!(menu.selection, Selection::Options(OptKey::min()));
        }
    }

    mod options_menu {
        use super::*;
        use crate::options::{FruitQty, LevelSize};

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
                fruits: FruitQty::new(4).unwrap(),
                level_size: LevelSize::Small,
            };
            let optmenu = OptionsMenu::new(opts);
            assert_eq!(optmenu.to_options(), opts);
        }
    }
}
