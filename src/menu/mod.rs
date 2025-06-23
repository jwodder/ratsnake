mod widgets;
use self::widgets::{Instructions, Logo};
use crate::app::AppState;
use crate::command::Command;
use crate::consts;
use crate::game::Game;
use crate::options::{Adjustable, OptKey, OptValue, Options};
use crate::util::{get_display_area, EnumExt};
use crossterm::event::{read, Event};
use enum_map::{Enum, EnumMap};
use ratatui::{
    buffer::Buffer,
    layout::{Flex, Layout, Rect},
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
    options: OptionsMenu,
}

impl MainMenu {
    pub(crate) fn new(options: Options) -> Self {
        MainMenu {
            selection: Selection::default(),
            options: OptionsMenu::new(options),
        }
    }

    pub(crate) fn draw(&self, frame: &mut Frame<'_>) {
        frame.render_widget(self, frame.area());
    }

    pub(crate) fn process_input(&mut self) -> std::io::Result<Option<AppState>> {
        Ok(self.handle_event(read()?))
    }

    fn handle_event(&mut self, event: Event) -> Option<AppState> {
        match (
            self.selection,
            Command::from_key_event(event.as_key_press_event()?)?,
        ) {
            (_, Command::Quit) => return Some(AppState::Quit),
            (_, Command::Home) => self.select(Selection::PlayButton, None),
            (_, Command::End) => self.select(Selection::QuitButton, None),
            (Selection::PlayButton, Command::Enter) | (_, Command::P) => {
                return Some(AppState::Game(self.play()))
            }
            (Selection::PlayButton, Command::Prev) => self.select(Selection::QuitButton, None),
            (Selection::PlayButton, Command::Down | Command::Next) => {
                self.select(Selection::Options, Some(true));
            }
            (Selection::Options, Command::Up | Command::Prev) => {
                if let Some(sel) = self.options.move_up() {
                    self.select(sel, None);
                }
            }
            (Selection::Options, Command::Down | Command::Next) => {
                if let Some(sel) = self.options.move_down() {
                    self.select(sel, None);
                }
            }
            (Selection::Options, Command::Left) => self.options.move_left(),
            (Selection::Options, Command::Right) => self.options.move_right(),
            (Selection::Options, Command::Space | Command::Enter) => self.options.toggle(),
            (Selection::QuitButton, Command::Enter) | (_, Command::Q) => {
                return Some(AppState::Quit);
            }
            (Selection::QuitButton, Command::Next) => self.select(Selection::PlayButton, None),
            (Selection::QuitButton, Command::Up | Command::Prev) => {
                self.select(Selection::Options, Some(false));
            }
            _ => (),
        }
        None
    }

    fn play(&self) -> Game {
        Game::new(self.options.to_options())
    }

    fn select(&mut self, selection: Selection, first_option: Option<bool>) {
        self.selection = selection;
        if selection == Selection::Options {
            if let Some(first) = first_option {
                self.options.selection = if first {
                    Some(OptKey::min())
                } else {
                    Some(OptKey::max())
                };
            } else {
                self.options.selection = None;
            }
        }
    }
}

impl Widget for &MainMenu {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let display = get_display_area(area);
        let [logo_area, instructions_area, play_area, options_area, quit_area] =
            Layout::vertical([
                Logo::HEIGHT,
                Instructions::HEIGHT,
                1,
                OptionsMenu::HEIGHT,
                1,
            ])
            .flex(Flex::Start)
            .spacing(1)
            .areas(display);

        let [logo_area] = Layout::horizontal([Logo::WIDTH])
            .flex(Flex::Center)
            .areas(logo_area);
        Logo.render(logo_area, buf);

        let [instructions_area] = Layout::horizontal([Instructions::WIDTH])
            .flex(Flex::Center)
            .areas(instructions_area);
        Instructions.render(instructions_area, buf);

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
        (&self.options).render(options_area, buf);

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
    }
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
            let menu = MainMenu::new(Options::default());
            let area = Rect::new(0, 0, 80, 24);
            let mut buffer = Buffer::empty(area);
            menu.render(area, &mut buffer);
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
                "                                   [Play (p)]                                   ",
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
            expected.set_style(Rect::new(37, 7, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(39, 7, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(41, 7, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(43, 7, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(37, 8, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(39, 8, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(41, 8, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(43, 8, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(37, 9, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(39, 9, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(41, 9, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(43, 9, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(42, 13, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(35, 13, 10, 1), consts::MENU_SELECTION_STYLE);
            expected.set_style(Rect::new(42, 22, 1, 1), consts::KEY_STYLE);
            pretty_assertions::assert_eq!(buffer, expected);
        }

        #[test]
        fn interact_options() {
            let area = Rect::new(0, 0, 80, 24);
            let mut menu = MainMenu::new(Options::default());
            assert!(menu
                .handle_event(Event::Key(KeyCode::Down.into()))
                .is_none());
            let mut buffer = Buffer::empty(area);
            menu.render(area, &mut buffer);
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
                "                                   [Play (p)]                                   ",
                "                                                                                ",
                "                          ┌ Options: ────────────────┐                          ",
                "                          │ » Wraparound     [ ]     │                          ",
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
            expected.set_style(Rect::new(37, 7, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(39, 7, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(41, 7, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(43, 7, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(37, 8, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(39, 8, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(41, 8, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(43, 8, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(37, 9, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(39, 9, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(41, 9, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(43, 9, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(42, 13, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(28, 16, 24, 1), consts::MENU_SELECTION_STYLE);
            expected.set_style(Rect::new(42, 22, 1, 1), consts::KEY_STYLE);
            pretty_assertions::assert_eq!(buffer, expected);

            assert!(menu
                .handle_event(Event::Key(KeyCode::Char(' ').into()))
                .is_none());
            let mut buffer = Buffer::empty(area);
            menu.render(area, &mut buffer);
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
                "                                   [Play (p)]                                   ",
                "                                                                                ",
                "                          ┌ Options: ────────────────┐                          ",
                "                          │ » Wraparound     [✓]     │                          ",
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
            expected.set_style(Rect::new(28, 16, 24, 1), consts::MENU_SELECTION_STYLE);
            expected.set_style(Rect::new(37, 7, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(39, 7, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(41, 7, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(43, 7, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(37, 8, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(39, 8, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(41, 8, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(43, 8, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(37, 9, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(39, 9, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(41, 9, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(43, 9, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(42, 13, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(42, 22, 1, 1), consts::KEY_STYLE);
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
                "                                   [Play (p)]                                   ",
                "                                                                                ",
                "                          ┌ Options: ────────────────┐                          ",
                "                          │   Wraparound     [✓]     │                          ",
                "                          │   Obstacles      [ ]     │                          ",
                "                          │   Fruits      ◁   1    ▶ │                          ",
                "                          │ » Level Size  ◀ Large  ▷ │                          ",
                "                          └──────────────────────────┘                          ",
                "                                                                                ",
                "                                   [Quit (q)]                                   ",
                "                                                                                ",
            ]);
            expected.set_style(Rect::new(19, 0, 15, 5), consts::FRUIT_STYLE);
            expected.set_style(Rect::new(34, 0, 28, 5), consts::SNAKE_STYLE);
            expected.set_style(Rect::new(37, 7, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(39, 7, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(41, 7, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(43, 7, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(37, 8, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(39, 8, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(41, 8, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(43, 8, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(37, 9, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(39, 9, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(41, 9, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(43, 9, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(42, 13, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(28, 19, 24, 1), consts::MENU_SELECTION_STYLE);
            expected.set_style(Rect::new(42, 22, 1, 1), consts::KEY_STYLE);
            pretty_assertions::assert_eq!(buffer, expected);

            assert!(menu
                .handle_event(Event::Key(KeyCode::Left.into()))
                .is_none());
            let mut buffer = Buffer::empty(area);
            menu.render(area, &mut buffer);
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
                "                                   [Play (p)]                                   ",
                "                                                                                ",
                "                          ┌ Options: ────────────────┐                          ",
                "                          │   Wraparound     [✓]     │                          ",
                "                          │   Obstacles      [ ]     │                          ",
                "                          │   Fruits      ◁   1    ▶ │                          ",
                "                          │ » Level Size  ◀ Medium ▶ │                          ",
                "                          └──────────────────────────┘                          ",
                "                                                                                ",
                "                                   [Quit (q)]                                   ",
                "                                                                                ",
            ]);
            expected.set_style(Rect::new(19, 0, 15, 5), consts::FRUIT_STYLE);
            expected.set_style(Rect::new(34, 0, 28, 5), consts::SNAKE_STYLE);
            expected.set_style(Rect::new(37, 7, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(39, 7, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(41, 7, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(43, 7, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(37, 8, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(39, 8, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(41, 8, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(43, 8, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(37, 9, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(39, 9, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(41, 9, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(43, 9, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(42, 13, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(28, 19, 24, 1), consts::MENU_SELECTION_STYLE);
            expected.set_style(Rect::new(42, 22, 1, 1), consts::KEY_STYLE);
            pretty_assertions::assert_eq!(buffer, expected);

            assert!(menu
                .handle_event(Event::Key(KeyCode::Left.into()))
                .is_none());
            let mut buffer = Buffer::empty(area);
            menu.render(area, &mut buffer);
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
                "                                   [Play (p)]                                   ",
                "                                                                                ",
                "                          ┌ Options: ────────────────┐                          ",
                "                          │   Wraparound     [✓]     │                          ",
                "                          │   Obstacles      [ ]     │                          ",
                "                          │   Fruits      ◁   1    ▶ │                          ",
                "                          │ » Level Size  ◁ Small  ▶ │                          ",
                "                          └──────────────────────────┘                          ",
                "                                                                                ",
                "                                   [Quit (q)]                                   ",
                "                                                                                ",
            ]);
            expected.set_style(Rect::new(19, 0, 15, 5), consts::FRUIT_STYLE);
            expected.set_style(Rect::new(34, 0, 28, 5), consts::SNAKE_STYLE);
            expected.set_style(Rect::new(37, 7, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(39, 7, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(41, 7, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(43, 7, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(37, 8, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(39, 8, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(41, 8, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(43, 8, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(37, 9, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(39, 9, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(41, 9, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(43, 9, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(42, 13, 1, 1), consts::KEY_STYLE);
            expected.set_style(Rect::new(28, 19, 24, 1), consts::MENU_SELECTION_STYLE);
            expected.set_style(Rect::new(42, 22, 1, 1), consts::KEY_STYLE);
            pretty_assertions::assert_eq!(buffer, expected);
        }

        /// Test that tabbing to the end of the options menu and then tabbing
        /// again until you loop back around to the options menu puts you at
        /// the start of the options.
        #[test]
        fn tab_wraparound() {
            let mut menu = MainMenu::new(Options::default());
            assert_eq!(menu.options.selection, None);
            for _ in OptKey::iter() {
                assert!(menu.handle_event(Event::Key(KeyCode::Tab.into())).is_none());
            }
            assert_eq!(menu.options.selection, Some(OptKey::max()));
            assert!(menu.handle_event(Event::Key(KeyCode::Tab.into())).is_none());
            assert_eq!(menu.options.selection, None);
            assert!(menu.handle_event(Event::Key(KeyCode::Tab.into())).is_none());
            assert!(menu.handle_event(Event::Key(KeyCode::Tab.into())).is_none());
            assert_eq!(menu.options.selection, Some(OptKey::min()));
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
