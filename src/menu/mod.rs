mod widgets;
use self::widgets::{Instructions, Logo};
use crate::app::Screen;
use crate::command::Command;
use crate::consts;
use crate::game::Game;
use crate::options::{Adjustable, OptKey, OptValue, Options};
use crate::util::{get_display_area, EnumExt, Globals};
use crate::warning::{Warning, WarningOutcome};
use crossterm::event::{read, Event};
use enum_map::{Enum, EnumMap};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{
        block::{Block, Padding},
        Widget,
    },
    Frame,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MainMenu {
    selection: Selection,
    opts_menu: OptionsMenu,
    state: MenuState,
    globals: Globals,
}

impl MainMenu {
    pub(crate) fn new(globals: Globals) -> Self {
        MainMenu {
            selection: Selection::default(),
            opts_menu: OptionsMenu::new(globals.options),
            state: MenuState::Plain,
            globals,
        }
    }

    pub(crate) fn draw(&self, frame: &mut Frame<'_>) {
        frame.render_widget(self, frame.area());
    }

    pub(crate) fn process_input(&mut self) -> std::io::Result<Option<Screen>> {
        Ok(self.handle_event(read()?))
    }

    fn handle_event(&mut self, event: Event) -> Option<Screen> {
        let cmd = Command::from_key_event(event.as_key_press_event()?)?;
        if cmd == Command::Quit {
            return Some(Screen::Quit);
        }
        match self.state {
            MenuState::Plain => match (self.selection, cmd) {
                (_, Command::Home) => self.select(Selection::PlayButton, None),
                (_, Command::End) => self.select(Selection::QuitButton, None),
                (Selection::PlayButton, Command::Enter) | (_, Command::P) => {
                    let options = self.opts_menu.to_options();
                    self.globals.options = options;
                    match options.save() {
                        Ok(()) => return Some(Screen::Game(self.play())),
                        Err(e) => self.state = MenuState::SaveWarning(Warning::from(e)),
                    }
                }
                (Selection::PlayButton, Command::Prev) => self.select(Selection::QuitButton, None),
                (Selection::PlayButton, Command::Down | Command::Next) => {
                    self.select(Selection::Options, Some(true));
                }
                (Selection::Options, Command::Up | Command::Prev) => {
                    if let Some(sel) = self.opts_menu.move_up() {
                        self.select(sel, None);
                    }
                }
                (Selection::Options, Command::Down | Command::Next) => {
                    if let Some(sel) = self.opts_menu.move_down() {
                        self.select(sel, None);
                    }
                }
                (Selection::Options, Command::Left) => self.opts_menu.move_left(),
                (Selection::Options, Command::Right) => self.opts_menu.move_right(),
                (Selection::Options, Command::Space | Command::Enter) => self.opts_menu.toggle(),
                (Selection::QuitButton, Command::Enter) | (_, Command::Q) => {
                    return Some(Screen::Quit);
                }
                (Selection::QuitButton, Command::Next) => self.select(Selection::PlayButton, None),
                (Selection::QuitButton, Command::Up | Command::Prev) => {
                    self.select(Selection::Options, Some(false));
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

    fn play(&self) -> Game {
        Game::new(self.globals.clone())
    }

    fn select(&mut self, selection: Selection, first_option: Option<bool>) {
        self.selection = selection;
        if selection == Selection::Options {
            if let Some(first) = first_option {
                self.opts_menu.selection = if first {
                    Some(OptKey::min())
                } else {
                    Some(OptKey::max())
                };
            } else {
                self.opts_menu.selection = None;
            }
        }
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

        let [main_area] = Layout::vertical([OptionsMenu::HEIGHT + 4])
            .flex(Flex::Start)
            .areas(main_area);
        let [form_area, instructions_area] =
            Layout::horizontal([OptionsMenu::WIDTH, Instructions::WIDTH])
                .flex(Flex::SpaceAround)
                .areas(main_area);

        let [play_area, options_area, quit_area] = Layout::vertical([1, OptionsMenu::HEIGHT, 1])
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
        (&self.opts_menu).render(options_area, buf);

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

#[derive(Clone, Debug, Eq, PartialEq)]
enum MenuState {
    Plain,
    SaveWarning(Warning),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum Selection {
    #[default]
    PlayButton,
    Options,
    QuitButton,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct OptionsMenu {
    /// If the currently-selected main menu item is an element of this menu,
    /// then `selection` is `Some(key)`, where `key` is the key of the selected
    /// item within the `OptionsMenu`.
    selection: Option<OptKey>,
    settings: EnumMap<OptKey, OptValue>,
}

impl OptionsMenu {
    #[allow(clippy::cast_possible_truncation)]
    const HEIGHT: u16 = (OptKey::LENGTH as u16) + 2 /* for border */;
    const HORIZONTAL_PADDING: u16 = 1; // padding on each side
    const POINTER_WIDTH: u16 = 2;
    const LABEL_VALUE_GUTTER: u16 = 2;
    const WIDTH: u16 = 2 /* for border */ + 2 * Self::HORIZONTAL_PADDING + Self::POINTER_WIDTH + OptKey::DISPLAY_WIDTH + Self::LABEL_VALUE_GUTTER + OptValue::DISPLAY_WIDTH;

    fn new(options: Options) -> Self {
        let settings = EnumMap::from_iter(OptKey::iter().map(|key| (key, options.get(key))));
        OptionsMenu {
            selection: None,
            settings,
        }
    }

    fn to_options(&self) -> Options {
        let mut opts = Options::default();
        for key in OptKey::iter() {
            opts.set(key, self.settings[key]);
        }
        opts
    }

    fn move_up(&mut self) -> Option<Selection> {
        self.selection = self.selection?.prev();
        self.selection.is_none().then_some(Selection::PlayButton)
    }

    fn move_down(&mut self) -> Option<Selection> {
        self.selection = self.selection?.next();
        self.selection.is_none().then_some(Selection::QuitButton)
    }

    fn move_left(&mut self) {
        if let Some(sel) = self.selection {
            self.settings[sel].decrease();
        }
    }

    fn move_right(&mut self) {
        if let Some(sel) = self.selection {
            self.settings[sel].increase();
        }
    }

    fn toggle(&mut self) {
        if let Some(sel) = self.selection {
            self.settings[sel].toggle();
        }
    }
}

impl Widget for &OptionsMenu {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title(" Options: ")
            .padding(Padding::horizontal(OptionsMenu::HORIZONTAL_PADDING));
        let menu_area = block.inner(area);
        block.render(area, buf);
        for ((key, value), row) in OptKey::iter()
            .map(|key| (key, self.settings[key]))
            .zip(menu_area.rows())
        {
            let selected = Some(key) == self.selection;
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
                 "                    [Play (p)]                                                  ",
                 "                                                                                ",
                 "           ┌ Options: ────────────────┐          Move the snake with:           ",
                 "           │   Wraparound     [ ]     │                 ← ↓ ↑ →                 ",
                 "           │   Obstacles      [ ]     │             or: h j k l                 ",
                 "           │   Fruits      ◁   1    ▶ │             or: a s w d                 ",
                 "           │   Level Size  ◀ Large  ▷ │             or: 4 2 8 6                 ",
                 "           └──────────────────────────┘          Eat the fruit, but             ",
                 "                                                 don't hit yourself!            ",
                 "                    [Quit (q)]                                                  ",
                 "                                                                                ",
                 "                                                                                ",
                 "                                                                                ",
                 "                                                                                ",
                 "                                                                                ",
                 "                                                                                ",
            ]);
            expected.set_style(Rect::new(19, 0, 15, 5), consts::FRUIT_STYLE); // "Rat"
            expected.set_style(Rect::new(34, 0, 28, 5), consts::SNAKE_STYLE); // "Snake"
            expected.set_style(Rect::new(33, 6, 13, 1), consts::SNAKE_STYLE); // ⚬⚬…⚬<
            expected.set_style(Rect::new(48, 6, 1, 1), consts::FRUIT_STYLE); // fruit in logo
            expected.set_style(Rect::new(27, 8, 1, 1), consts::KEY_STYLE); // `p`
            expected.set_style(Rect::new(20, 8, 10, 1), consts::MENU_SELECTION_STYLE); // Play button
            expected.set_style(Rect::new(27, 17, 1, 1), consts::KEY_STYLE); // `q`
            expected.set_style(Rect::new(56, 11, 1, 1), consts::KEY_STYLE); // `←`
            expected.set_style(Rect::new(58, 11, 1, 1), consts::KEY_STYLE); // `↓`
            expected.set_style(Rect::new(60, 11, 1, 1), consts::KEY_STYLE); // `↑`
            expected.set_style(Rect::new(62, 11, 1, 1), consts::KEY_STYLE); // `→`
            expected.set_style(Rect::new(56, 12, 1, 1), consts::KEY_STYLE); // `h`
            expected.set_style(Rect::new(58, 12, 1, 1), consts::KEY_STYLE); // `j`
            expected.set_style(Rect::new(60, 12, 1, 1), consts::KEY_STYLE); // `k`
            expected.set_style(Rect::new(62, 12, 1, 1), consts::KEY_STYLE); // `l`
            expected.set_style(Rect::new(56, 13, 1, 1), consts::KEY_STYLE); // `a`
            expected.set_style(Rect::new(58, 13, 1, 1), consts::KEY_STYLE); // `s`
            expected.set_style(Rect::new(60, 13, 1, 1), consts::KEY_STYLE); // `w`
            expected.set_style(Rect::new(62, 13, 1, 1), consts::KEY_STYLE); // `s`
            expected.set_style(Rect::new(56, 14, 1, 1), consts::KEY_STYLE); // `4`
            expected.set_style(Rect::new(58, 14, 1, 1), consts::KEY_STYLE); // `2`
            expected.set_style(Rect::new(60, 14, 1, 1), consts::KEY_STYLE); // `8`
            expected.set_style(Rect::new(62, 14, 1, 1), consts::KEY_STYLE); // `6`
            pretty_assertions::assert_eq!(buffer, expected);
        }

        #[test]
        fn interact_options() {
            let area = Rect::new(0, 0, 80, 24);
            let mut menu = MainMenu::new(Globals::default());
            assert!(menu
                .handle_event(Event::Key(KeyCode::Down.into()))
                .is_none());
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
                 "                    [Play (p)]                                                  ",
                 "                                                                                ",
                 "           ┌ Options: ────────────────┐          Move the snake with:           ",
                 "           │ » Wraparound     [ ]     │                 ← ↓ ↑ →                 ",
                 "           │   Obstacles      [ ]     │             or: h j k l                 ",
                 "           │   Fruits      ◁   1    ▶ │             or: a s w d                 ",
                 "           │   Level Size  ◀ Large  ▷ │             or: 4 2 8 6                 ",
                 "           └──────────────────────────┘          Eat the fruit, but             ",
                 "                                                 don't hit yourself!            ",
                 "                    [Quit (q)]                                                  ",
                 "                                                                                ",
                 "                                                                                ",
                 "                                                                                ",
                 "                                                                                ",
                 "                                                                                ",
                 "                                                                                ",
            ]);
            expected.set_style(Rect::new(19, 0, 15, 5), consts::FRUIT_STYLE); // "Rat"
            expected.set_style(Rect::new(34, 0, 28, 5), consts::SNAKE_STYLE); // "Snake"
            expected.set_style(Rect::new(33, 6, 13, 1), consts::SNAKE_STYLE); // ⚬⚬…⚬<
            expected.set_style(Rect::new(48, 6, 1, 1), consts::FRUIT_STYLE); // fruit in logo
            expected.set_style(Rect::new(27, 8, 1, 1), consts::KEY_STYLE); // `p`
            expected.set_style(Rect::new(13, 11, 24, 1), consts::MENU_SELECTION_STYLE); // "Wraparound" option
            expected.set_style(Rect::new(27, 17, 1, 1), consts::KEY_STYLE); // `q`
            expected.set_style(Rect::new(56, 11, 1, 1), consts::KEY_STYLE); // `←`
            expected.set_style(Rect::new(58, 11, 1, 1), consts::KEY_STYLE); // `↓`
            expected.set_style(Rect::new(60, 11, 1, 1), consts::KEY_STYLE); // `↑`
            expected.set_style(Rect::new(62, 11, 1, 1), consts::KEY_STYLE); // `→`
            expected.set_style(Rect::new(56, 12, 1, 1), consts::KEY_STYLE); // `h`
            expected.set_style(Rect::new(58, 12, 1, 1), consts::KEY_STYLE); // `j`
            expected.set_style(Rect::new(60, 12, 1, 1), consts::KEY_STYLE); // `k`
            expected.set_style(Rect::new(62, 12, 1, 1), consts::KEY_STYLE); // `l`
            expected.set_style(Rect::new(56, 13, 1, 1), consts::KEY_STYLE); // `a`
            expected.set_style(Rect::new(58, 13, 1, 1), consts::KEY_STYLE); // `s`
            expected.set_style(Rect::new(60, 13, 1, 1), consts::KEY_STYLE); // `w`
            expected.set_style(Rect::new(62, 13, 1, 1), consts::KEY_STYLE); // `s`
            expected.set_style(Rect::new(56, 14, 1, 1), consts::KEY_STYLE); // `4`
            expected.set_style(Rect::new(58, 14, 1, 1), consts::KEY_STYLE); // `2`
            expected.set_style(Rect::new(60, 14, 1, 1), consts::KEY_STYLE); // `8`
            expected.set_style(Rect::new(62, 14, 1, 1), consts::KEY_STYLE); // `6`
            pretty_assertions::assert_eq!(buffer, expected);

            assert!(menu
                .handle_event(Event::Key(KeyCode::Char(' ').into()))
                .is_none());
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
                 "                    [Play (p)]                                                  ",
                 "                                                                                ",
                 "           ┌ Options: ────────────────┐          Move the snake with:           ",
                 "           │ » Wraparound     [✓]     │                 ← ↓ ↑ →                 ",
                 "           │   Obstacles      [ ]     │             or: h j k l                 ",
                 "           │   Fruits      ◁   1    ▶ │             or: a s w d                 ",
                 "           │   Level Size  ◀ Large  ▷ │             or: 4 2 8 6                 ",
                 "           └──────────────────────────┘          Eat the fruit, but             ",
                 "                                                 don't hit yourself!            ",
                 "                    [Quit (q)]                                                  ",
                 "                                                                                ",
                 "                                                                                ",
                 "                                                                                ",
                 "                                                                                ",
                 "                                                                                ",
                 "                                                                                ",
            ]);
            expected.set_style(Rect::new(19, 0, 15, 5), consts::FRUIT_STYLE); // "Rat"
            expected.set_style(Rect::new(34, 0, 28, 5), consts::SNAKE_STYLE); // "Snake"
            expected.set_style(Rect::new(33, 6, 13, 1), consts::SNAKE_STYLE); // ⚬⚬…⚬<
            expected.set_style(Rect::new(48, 6, 1, 1), consts::FRUIT_STYLE); // fruit in logo
            expected.set_style(Rect::new(27, 8, 1, 1), consts::KEY_STYLE); // `p`
            expected.set_style(Rect::new(13, 11, 24, 1), consts::MENU_SELECTION_STYLE); // "Wraparound" option
            expected.set_style(Rect::new(27, 17, 1, 1), consts::KEY_STYLE); // `q`
            expected.set_style(Rect::new(56, 11, 1, 1), consts::KEY_STYLE); // `←`
            expected.set_style(Rect::new(58, 11, 1, 1), consts::KEY_STYLE); // `↓`
            expected.set_style(Rect::new(60, 11, 1, 1), consts::KEY_STYLE); // `↑`
            expected.set_style(Rect::new(62, 11, 1, 1), consts::KEY_STYLE); // `→`
            expected.set_style(Rect::new(56, 12, 1, 1), consts::KEY_STYLE); // `h`
            expected.set_style(Rect::new(58, 12, 1, 1), consts::KEY_STYLE); // `j`
            expected.set_style(Rect::new(60, 12, 1, 1), consts::KEY_STYLE); // `k`
            expected.set_style(Rect::new(62, 12, 1, 1), consts::KEY_STYLE); // `l`
            expected.set_style(Rect::new(56, 13, 1, 1), consts::KEY_STYLE); // `a`
            expected.set_style(Rect::new(58, 13, 1, 1), consts::KEY_STYLE); // `s`
            expected.set_style(Rect::new(60, 13, 1, 1), consts::KEY_STYLE); // `w`
            expected.set_style(Rect::new(62, 13, 1, 1), consts::KEY_STYLE); // `s`
            expected.set_style(Rect::new(56, 14, 1, 1), consts::KEY_STYLE); // `4`
            expected.set_style(Rect::new(58, 14, 1, 1), consts::KEY_STYLE); // `2`
            expected.set_style(Rect::new(60, 14, 1, 1), consts::KEY_STYLE); // `8`
            expected.set_style(Rect::new(62, 14, 1, 1), consts::KEY_STYLE); // `6`
            pretty_assertions::assert_eq!(buffer, expected);

            assert!(menu
                .handle_event(Event::Key(KeyCode::Down.into()))
                .is_none());
            assert!(menu
                .handle_event(Event::Key(KeyCode::Down.into()))
                .is_none());
            assert!(menu
                .handle_event(Event::Key(KeyCode::Down.into()))
                .is_none());
            assert!(menu
                .handle_event(Event::Key(KeyCode::Char(' ').into()))
                .is_none());
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
                 "                    [Play (p)]                                                  ",
                 "                                                                                ",
                 "           ┌ Options: ────────────────┐          Move the snake with:           ",
                 "           │   Wraparound     [✓]     │                 ← ↓ ↑ →                 ",
                 "           │   Obstacles      [ ]     │             or: h j k l                 ",
                 "           │   Fruits      ◁   1    ▶ │             or: a s w d                 ",
                 "           │ » Level Size  ◀ Large  ▷ │             or: 4 2 8 6                 ",
                 "           └──────────────────────────┘          Eat the fruit, but             ",
                 "                                                 don't hit yourself!            ",
                 "                    [Quit (q)]                                                  ",
                 "                                                                                ",
                 "                                                                                ",
                 "                                                                                ",
                 "                                                                                ",
                 "                                                                                ",
                 "                                                                                ",
            ]);
            expected.set_style(Rect::new(19, 0, 15, 5), consts::FRUIT_STYLE); // "Rat"
            expected.set_style(Rect::new(34, 0, 28, 5), consts::SNAKE_STYLE); // "Snake"
            expected.set_style(Rect::new(33, 6, 13, 1), consts::SNAKE_STYLE); // ⚬⚬…⚬<
            expected.set_style(Rect::new(48, 6, 1, 1), consts::FRUIT_STYLE); // fruit in logo
            expected.set_style(Rect::new(27, 8, 1, 1), consts::KEY_STYLE); // `p`
            expected.set_style(Rect::new(13, 14, 24, 1), consts::MENU_SELECTION_STYLE); // "Level Size" option
            expected.set_style(Rect::new(27, 17, 1, 1), consts::KEY_STYLE); // `q`
            expected.set_style(Rect::new(56, 11, 1, 1), consts::KEY_STYLE); // `←`
            expected.set_style(Rect::new(58, 11, 1, 1), consts::KEY_STYLE); // `↓`
            expected.set_style(Rect::new(60, 11, 1, 1), consts::KEY_STYLE); // `↑`
            expected.set_style(Rect::new(62, 11, 1, 1), consts::KEY_STYLE); // `→`
            expected.set_style(Rect::new(56, 12, 1, 1), consts::KEY_STYLE); // `h`
            expected.set_style(Rect::new(58, 12, 1, 1), consts::KEY_STYLE); // `j`
            expected.set_style(Rect::new(60, 12, 1, 1), consts::KEY_STYLE); // `k`
            expected.set_style(Rect::new(62, 12, 1, 1), consts::KEY_STYLE); // `l`
            expected.set_style(Rect::new(56, 13, 1, 1), consts::KEY_STYLE); // `a`
            expected.set_style(Rect::new(58, 13, 1, 1), consts::KEY_STYLE); // `s`
            expected.set_style(Rect::new(60, 13, 1, 1), consts::KEY_STYLE); // `w`
            expected.set_style(Rect::new(62, 13, 1, 1), consts::KEY_STYLE); // `s`
            expected.set_style(Rect::new(56, 14, 1, 1), consts::KEY_STYLE); // `4`
            expected.set_style(Rect::new(58, 14, 1, 1), consts::KEY_STYLE); // `2`
            expected.set_style(Rect::new(60, 14, 1, 1), consts::KEY_STYLE); // `8`
            expected.set_style(Rect::new(62, 14, 1, 1), consts::KEY_STYLE); // `6`
            pretty_assertions::assert_eq!(buffer, expected);

            assert!(menu
                .handle_event(Event::Key(KeyCode::Left.into()))
                .is_none());
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
                 "                    [Play (p)]                                                  ",
                 "                                                                                ",
                 "           ┌ Options: ────────────────┐          Move the snake with:           ",
                 "           │   Wraparound     [✓]     │                 ← ↓ ↑ →                 ",
                 "           │   Obstacles      [ ]     │             or: h j k l                 ",
                 "           │   Fruits      ◁   1    ▶ │             or: a s w d                 ",
                 "           │ » Level Size  ◀ Medium ▶ │             or: 4 2 8 6                 ",
                 "           └──────────────────────────┘          Eat the fruit, but             ",
                 "                                                 don't hit yourself!            ",
                 "                    [Quit (q)]                                                  ",
                 "                                                                                ",
                 "                                                                                ",
                 "                                                                                ",
                 "                                                                                ",
                 "                                                                                ",
                 "                                                                                ",
            ]);
            expected.set_style(Rect::new(19, 0, 15, 5), consts::FRUIT_STYLE); // "Rat"
            expected.set_style(Rect::new(34, 0, 28, 5), consts::SNAKE_STYLE); // "Snake"
            expected.set_style(Rect::new(33, 6, 13, 1), consts::SNAKE_STYLE); // ⚬⚬…⚬<
            expected.set_style(Rect::new(48, 6, 1, 1), consts::FRUIT_STYLE); // fruit in logo
            expected.set_style(Rect::new(27, 8, 1, 1), consts::KEY_STYLE); // `p`
            expected.set_style(Rect::new(13, 14, 24, 1), consts::MENU_SELECTION_STYLE); // "Level Size" option
            expected.set_style(Rect::new(27, 17, 1, 1), consts::KEY_STYLE); // `q`
            expected.set_style(Rect::new(56, 11, 1, 1), consts::KEY_STYLE); // `←`
            expected.set_style(Rect::new(58, 11, 1, 1), consts::KEY_STYLE); // `↓`
            expected.set_style(Rect::new(60, 11, 1, 1), consts::KEY_STYLE); // `↑`
            expected.set_style(Rect::new(62, 11, 1, 1), consts::KEY_STYLE); // `→`
            expected.set_style(Rect::new(56, 12, 1, 1), consts::KEY_STYLE); // `h`
            expected.set_style(Rect::new(58, 12, 1, 1), consts::KEY_STYLE); // `j`
            expected.set_style(Rect::new(60, 12, 1, 1), consts::KEY_STYLE); // `k`
            expected.set_style(Rect::new(62, 12, 1, 1), consts::KEY_STYLE); // `l`
            expected.set_style(Rect::new(56, 13, 1, 1), consts::KEY_STYLE); // `a`
            expected.set_style(Rect::new(58, 13, 1, 1), consts::KEY_STYLE); // `s`
            expected.set_style(Rect::new(60, 13, 1, 1), consts::KEY_STYLE); // `w`
            expected.set_style(Rect::new(62, 13, 1, 1), consts::KEY_STYLE); // `s`
            expected.set_style(Rect::new(56, 14, 1, 1), consts::KEY_STYLE); // `4`
            expected.set_style(Rect::new(58, 14, 1, 1), consts::KEY_STYLE); // `2`
            expected.set_style(Rect::new(60, 14, 1, 1), consts::KEY_STYLE); // `8`
            expected.set_style(Rect::new(62, 14, 1, 1), consts::KEY_STYLE); // `6`
            pretty_assertions::assert_eq!(buffer, expected);

            assert!(menu
                .handle_event(Event::Key(KeyCode::Left.into()))
                .is_none());
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
                 "                    [Play (p)]                                                  ",
                 "                                                                                ",
                 "           ┌ Options: ────────────────┐          Move the snake with:           ",
                 "           │   Wraparound     [✓]     │                 ← ↓ ↑ →                 ",
                 "           │   Obstacles      [ ]     │             or: h j k l                 ",
                 "           │   Fruits      ◁   1    ▶ │             or: a s w d                 ",
                 "           │ » Level Size  ◁ Small  ▶ │             or: 4 2 8 6                 ",
                 "           └──────────────────────────┘          Eat the fruit, but             ",
                 "                                                 don't hit yourself!            ",
                 "                    [Quit (q)]                                                  ",
                 "                                                                                ",
                 "                                                                                ",
                 "                                                                                ",
                 "                                                                                ",
                 "                                                                                ",
                 "                                                                                ",
            ]);
            expected.set_style(Rect::new(19, 0, 15, 5), consts::FRUIT_STYLE); // "Rat"
            expected.set_style(Rect::new(34, 0, 28, 5), consts::SNAKE_STYLE); // "Snake"
            expected.set_style(Rect::new(33, 6, 13, 1), consts::SNAKE_STYLE); // ⚬⚬…⚬<
            expected.set_style(Rect::new(48, 6, 1, 1), consts::FRUIT_STYLE); // fruit in logo
            expected.set_style(Rect::new(27, 8, 1, 1), consts::KEY_STYLE); // `p`
            expected.set_style(Rect::new(13, 14, 24, 1), consts::MENU_SELECTION_STYLE); // "Level Size" option
            expected.set_style(Rect::new(27, 17, 1, 1), consts::KEY_STYLE); // `q`
            expected.set_style(Rect::new(56, 11, 1, 1), consts::KEY_STYLE); // `←`
            expected.set_style(Rect::new(58, 11, 1, 1), consts::KEY_STYLE); // `↓`
            expected.set_style(Rect::new(60, 11, 1, 1), consts::KEY_STYLE); // `↑`
            expected.set_style(Rect::new(62, 11, 1, 1), consts::KEY_STYLE); // `→`
            expected.set_style(Rect::new(56, 12, 1, 1), consts::KEY_STYLE); // `h`
            expected.set_style(Rect::new(58, 12, 1, 1), consts::KEY_STYLE); // `j`
            expected.set_style(Rect::new(60, 12, 1, 1), consts::KEY_STYLE); // `k`
            expected.set_style(Rect::new(62, 12, 1, 1), consts::KEY_STYLE); // `l`
            expected.set_style(Rect::new(56, 13, 1, 1), consts::KEY_STYLE); // `a`
            expected.set_style(Rect::new(58, 13, 1, 1), consts::KEY_STYLE); // `s`
            expected.set_style(Rect::new(60, 13, 1, 1), consts::KEY_STYLE); // `w`
            expected.set_style(Rect::new(62, 13, 1, 1), consts::KEY_STYLE); // `s`
            expected.set_style(Rect::new(56, 14, 1, 1), consts::KEY_STYLE); // `4`
            expected.set_style(Rect::new(58, 14, 1, 1), consts::KEY_STYLE); // `2`
            expected.set_style(Rect::new(60, 14, 1, 1), consts::KEY_STYLE); // `8`
            expected.set_style(Rect::new(62, 14, 1, 1), consts::KEY_STYLE); // `6`
            pretty_assertions::assert_eq!(buffer, expected);
        }

        /// Test that tabbing to the end of the options menu and then tabbing
        /// again until you loop back around to the options menu puts you at
        /// the start of the options.
        #[test]
        fn tab_wraparound() {
            let mut menu = MainMenu::new(Globals::default());
            assert_eq!(menu.opts_menu.selection, None);
            for _ in OptKey::iter() {
                assert!(menu.handle_event(Event::Key(KeyCode::Tab.into())).is_none());
            }
            assert_eq!(menu.opts_menu.selection, Some(OptKey::max()));
            assert!(menu.handle_event(Event::Key(KeyCode::Tab.into())).is_none());
            assert_eq!(menu.opts_menu.selection, None);
            assert!(menu.handle_event(Event::Key(KeyCode::Tab.into())).is_none());
            assert!(menu.handle_event(Event::Key(KeyCode::Tab.into())).is_none());
            assert_eq!(menu.opts_menu.selection, Some(OptKey::min()));
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
