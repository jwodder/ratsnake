mod direction;
mod levels;
mod paused;
mod snake;
use self::direction::Direction;
use self::levels::LevelMap;
use self::paused::{PauseOpt, Paused};
use self::snake::Snake;
use crate::app::Screen;
use crate::command::Command;
use crate::consts;
use crate::util::{center_rect, get_display_area, Globals};
use crossterm::event::{poll, read, Event};
use rand::{seq::IteratorRandom, Rng};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Margin, Position, Rect, Size},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Widget},
    Frame,
};
use std::collections::HashSet;
use std::time::Instant;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Game<R = rand::rngs::ThreadRng> {
    rng: R,
    score: u32,
    snake: Snake,
    fruits: HashSet<Position>,
    state: GameState,
    map: LevelMap,
    globals: Globals,
    next_tick: Option<Instant>,
}

impl Game<rand::rngs::ThreadRng> {
    pub(crate) fn new(globals: Globals) -> Self {
        Game::new_with_rng(globals, rand::rng())
    }
}

impl<R: Rng> Game<R> {
    pub(crate) fn new_with_rng(globals: Globals, mut rng: R) -> Game<R> {
        let mut map = LevelMap::new(globals.options.level_bounds());
        if globals.options.obstacles {
            map.set_obstacles(&mut rng);
        }
        let snake = map.new_snake();
        let fruit_qty = globals.options.fruits.get();
        let mut game = Game {
            rng,
            score: 0,
            snake,
            fruits: HashSet::new(),
            state: GameState::Running,
            map,
            globals,
            next_tick: None,
        };
        for _ in 0..fruit_qty {
            game.place_fruit();
        }
        game
    }

    pub(crate) fn process_input(&mut self) -> std::io::Result<Option<Screen>> {
        if self.running() {
            if self.next_tick.is_none() {
                self.next_tick = Some(Instant::now() + consts::TICK_PERIOD);
            }
            let when = self.next_tick.expect("next_tick should be Some");
            let wait = when.saturating_duration_since(Instant::now());
            if wait.is_zero() || !poll(wait)? {
                self.advance();
                self.next_tick = None;
                Ok(None)
            } else {
                Ok(self.handle_event(read()?))
            }
        } else {
            Ok(self.handle_event(read()?))
        }
    }

    fn advance(&mut self) {
        if !self.running() {
            return;
        }
        if !self.snake.advance(self.map.bounds()) {
            self.state = GameState::Dead;
            return;
        }
        if self.fruits.remove(&self.snake.head()) {
            self.score += 1;
            self.snake.grow();
            self.place_fruit();
        } else if self.snake.body().contains(&self.snake.head())
            || self.map.obstacles().contains(&self.snake.head())
        {
            self.state = GameState::Dead;
        }
        if self.fruits.is_empty() {
            self.state = GameState::Exhausted;
        }
    }

    fn place_fruit(&mut self) {
        let mut occupied = &self.fruits | self.map.obstacles();
        occupied.insert(self.snake.head());
        occupied.extend(self.snake.body().iter().copied());
        self.fruits.extend(
            self.map
                .bounds()
                .positions()
                .filter(move |p| !occupied.contains(p))
                .choose(&mut self.rng),
        );
    }
}

impl<R> Game<R> {
    pub(crate) fn draw(&self, frame: &mut Frame<'_>) {
        frame.render_widget(self, frame.area());
    }

    fn handle_event(&mut self, event: Event) -> Option<Screen> {
        match self.state {
            GameState::Running => {
                if event == Event::FocusLost {
                    self.pause();
                } else {
                    match Command::from_key_event(event.as_key_press_event()?)? {
                        Command::Quit => return Some(Screen::Quit),
                        Command::Up => self.snake.turn(Direction::North),
                        Command::Left => self.snake.turn(Direction::West),
                        Command::Down => self.snake.turn(Direction::South),
                        Command::Right => self.snake.turn(Direction::East),
                        Command::Esc => self.pause(),
                        _ => (),
                    }
                }
            }
            GameState::Paused(ref mut paused) => match paused.handle_event(event)? {
                PauseOpt::Resume => self.state = GameState::Running,
                PauseOpt::Restart => return Some(Screen::Game(Game::new(self.globals.clone()))),
                PauseOpt::MainMenu => {
                    return Some(Screen::Main(crate::menu::MainMenu::new(
                        self.globals.clone(),
                    )))
                }
                PauseOpt::Quit => return Some(Screen::Quit),
            },
            GameState::Dead | GameState::Exhausted => {
                match Command::from_key_event(event.as_key_press_event()?)? {
                    Command::R => return Some(Screen::Game(Game::new(self.globals.clone()))),
                    Command::M => {
                        return Some(Screen::Main(crate::menu::MainMenu::new(
                            self.globals.clone(),
                        )))
                    }
                    Command::Quit | Command::Q => return Some(Screen::Quit),
                    _ => (),
                }
            }
        }
        None
    }

    fn running(&self) -> bool {
        self.state == GameState::Running
    }

    fn pause(&mut self) {
        self.state = GameState::Paused(Paused::new());
    }
}

impl<R> Widget for &Game<R> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let display = get_display_area(area);
        let [score_area, block_area, msg1_area, msg2_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(display);
        Line::styled(format!(" Score: {}", self.score), consts::SCORE_BAR_STYLE)
            .render(score_area, buf);

        let mut block_size = self.map.size();
        block_size.width = block_size.width.saturating_add(2);
        block_size.height = block_size.height.saturating_add(2);
        let block_area = center_rect(block_area, block_size);
        if self.map.wrap() {
            DottedBorder.render(block_area, buf);
        } else {
            Block::bordered().render(block_area, buf);
        }

        let level_area = block_area.inner(Margin::new(1, 1));
        let mut level = Canvas {
            area: level_area,
            buf,
        };
        for &p in self.snake.body() {
            level.draw_cell(p, consts::SNAKE_BODY_SYMBOL, consts::SNAKE_STYLE);
        }
        for &pos in &self.fruits {
            level.draw_cell(pos, consts::FRUIT_SYMBOL, consts::FRUIT_STYLE);
        }
        for &pos in self.map.obstacles() {
            level.draw_cell(pos, consts::OBSTACLE_SYMBOL, consts::OBSTACLE_STYLE);
        }
        // Draw the head last so that, if it's a collision, we overwrite
        // whatever it's colliding with
        if self.state == GameState::Dead {
            level.draw_cell(
                self.snake.head(),
                consts::COLLISION_SYMBOL,
                consts::COLLISION_STYLE,
            );
        } else {
            level.draw_cell(
                self.snake.head(),
                self.snake.head_symbol(),
                consts::SNAKE_STYLE,
            );
        }

        match self.state {
            GameState::Running => (),
            GameState::Paused(paused) => {
                let pause_area = center_rect(
                    display,
                    Size {
                        width: Paused::WIDTH,
                        height: Paused::HEIGHT,
                    },
                );
                paused.render(pause_area, buf);
            }
            GameState::Dead | GameState::Exhausted => {
                Span::from(" — GAME OVER —").render(msg1_area, buf);
                Line::from_iter([
                    Span::raw(" Choose One: Restart ("),
                    Span::styled("r", consts::KEY_STYLE),
                    Span::raw(") — Main Menu ("),
                    Span::styled("m", consts::KEY_STYLE),
                    Span::raw(") — Quit ("),
                    Span::styled("q", consts::KEY_STYLE),
                    Span::raw(")"),
                ])
                .render(msg2_area, buf);
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Canvas<'a> {
    area: Rect,
    buf: &'a mut Buffer,
}

impl Canvas<'_> {
    fn draw_char(&mut self, pos: Position, symbol: char) {
        let Some(x) = self.area.x.checked_add(pos.x) else {
            return;
        };
        let Some(y) = self.area.y.checked_add(pos.y) else {
            return;
        };
        if let Some(cell) = self.buf.cell_mut((x, y)) {
            cell.set_char(symbol);
        }
    }

    fn draw_cell(&mut self, pos: Position, symbol: char, style: Style) {
        let Some(x) = self.area.x.checked_add(pos.x) else {
            return;
        };
        let Some(y) = self.area.y.checked_add(pos.y) else {
            return;
        };
        if let Some(cell) = self.buf.cell_mut((x, y)) {
            cell.set_char(symbol);
            cell.set_style(Style::reset().patch(style));
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DottedBorder;

impl Widget for DottedBorder {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }
        let size = area.as_size();
        let max_x = size.width.saturating_sub(1);
        let max_y = size.height.saturating_sub(1);
        let mut canvas = Canvas { area, buf };
        canvas.draw_char(Position::ORIGIN, '·');
        canvas.draw_char(Position::new(max_x, 0), '·');
        canvas.draw_char(Position::new(max_x, max_y), '·');
        canvas.draw_char(Position::new(0, max_y), '·');
        for x in 1..max_x {
            canvas.draw_char(Position::new(x, 0), '⋯');
            canvas.draw_char(Position::new(x, max_y), '⋯');
        }
        for y in 1..max_y {
            canvas.draw_char(Position::new(0, y), '⋮');
            canvas.draw_char(Position::new(max_x, y), '⋮');
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum GameState {
    Running,
    Paused(Paused),
    Dead,
    /// The snake has filled the board and there are no more spaces to place
    /// fruits in.
    Exhausted,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::LevelSize;
    use crossterm::event::KeyCode;
    use rand::SeedableRng;
    use rand_chacha::ChaCha12Rng;
    use std::collections::VecDeque;

    const RNG_SEED: u64 = 0x0123456789ABCDEF;

    #[test]
    fn new_game() {
        let game = Game::new_with_rng(Globals::default(), ChaCha12Rng::seed_from_u64(RNG_SEED));
        let area = Rect::new(0, 0, 80, 24);
        let mut buffer = Buffer::empty(area);
        game.render(area, &mut buffer);
        let mut expected = Buffer::with_lines([
            " Score: 0",
            " ┌────────────────────────────────────────────────────────────────────────────┐ ",
            " │                                                                            │ ",
            " │                                                                            │ ",
            " │                                                                            │ ",
            " │                                                                            │ ",
            " │                                                                            │ ",
            " │                                                                            │ ",
            " │                                                                            │ ",
            " │                                                                            │ ",
            " │                          ●                                                 │ ",
            " │                                      v                                     │ ",
            " │                                                                            │ ",
            " │                                                                            │ ",
            " │                                                                            │ ",
            " │                                                                            │ ",
            " │                                                                            │ ",
            " │                                                                            │ ",
            " │                                                                            │ ",
            " │                                                                            │ ",
            " │                                                                            │ ",
            " └────────────────────────────────────────────────────────────────────────────┘ ",
            "",
            "",
        ]);
        expected.set_style(Rect::new(0, 0, 80, 1), consts::SCORE_BAR_STYLE);
        expected.set_style(Rect::new(40, 11, 1, 1), consts::SNAKE_STYLE);
        expected.set_style(Rect::new(28, 10, 1, 1), consts::FRUIT_STYLE);
        pretty_assertions::assert_eq!(buffer, expected);
    }

    #[test]
    fn new_wraparound_game() {
        let mut globals = Globals::default();
        globals.options.wraparound = true;
        let game = Game::new_with_rng(globals, ChaCha12Rng::seed_from_u64(RNG_SEED));
        let area = Rect::new(0, 0, 80, 24);
        let mut buffer = Buffer::empty(area);
        game.render(area, &mut buffer);
        let mut expected = Buffer::with_lines([
            " Score: 0",
            " ·⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯· ",
            " ⋮                                                                            ⋮ ",
            " ⋮                                                                            ⋮ ",
            " ⋮                                                                            ⋮ ",
            " ⋮                                                                            ⋮ ",
            " ⋮                                                                            ⋮ ",
            " ⋮                                                                            ⋮ ",
            " ⋮                                                                            ⋮ ",
            " ⋮                                                                            ⋮ ",
            " ⋮                          ●                                                 ⋮ ",
            " ⋮                                      v                                     ⋮ ",
            " ⋮                                                                            ⋮ ",
            " ⋮                                                                            ⋮ ",
            " ⋮                                                                            ⋮ ",
            " ⋮                                                                            ⋮ ",
            " ⋮                                                                            ⋮ ",
            " ⋮                                                                            ⋮ ",
            " ⋮                                                                            ⋮ ",
            " ⋮                                                                            ⋮ ",
            " ⋮                                                                            ⋮ ",
            " ·⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯⋯· ",
            "",
            "",
        ]);
        expected.set_style(Rect::new(0, 0, 80, 1), consts::SCORE_BAR_STYLE);
        expected.set_style(Rect::new(40, 11, 1, 1), consts::SNAKE_STYLE);
        expected.set_style(Rect::new(28, 10, 1, 1), consts::FRUIT_STYLE);
        pretty_assertions::assert_eq!(buffer, expected);
    }

    #[test]
    fn self_collision() {
        let mut game = Game::new_with_rng(Globals::default(), ChaCha12Rng::seed_from_u64(RNG_SEED));
        game.score = 3;
        game.snake.head = Position::new(30, 6);
        game.snake.body = VecDeque::from([
            Position::new(30, 6),
            Position::new(31, 6),
            Position::new(32, 6),
            Position::new(33, 6),
            Position::new(33, 7),
            Position::new(33, 8),
            Position::new(33, 9),
            Position::new(32, 9),
            Position::new(31, 9),
            Position::new(30, 9),
            Position::new(30, 8),
            Position::new(30, 7),
        ]);
        game.snake.max_len = 12;
        game.snake.direction = Direction::North;
        game.state = GameState::Dead;
        let area = Rect::new(0, 0, 80, 24);
        let mut buffer = Buffer::empty(area);
        game.render(area, &mut buffer);
        let mut expected = Buffer::with_lines([
            " Score: 3",
            " ┌────────────────────────────────────────────────────────────────────────────┐ ",
            " │                                                                            │ ",
            " │                                                                            │ ",
            " │                                                                            │ ",
            " │                                                                            │ ",
            " │                                                                            │ ",
            " │                                                                            │ ",
            " │                              ×⚬⚬⚬                                          │ ",
            " │                              ⚬  ⚬                                          │ ",
            " │                          ●   ⚬  ⚬                                          │ ",
            " │                              ⚬⚬⚬⚬                                          │ ",
            " │                                                                            │ ",
            " │                                                                            │ ",
            " │                                                                            │ ",
            " │                                                                            │ ",
            " │                                                                            │ ",
            " │                                                                            │ ",
            " │                                                                            │ ",
            " │                                                                            │ ",
            " │                                                                            │ ",
            " └────────────────────────────────────────────────────────────────────────────┘ ",
            " — GAME OVER —",
            " Choose One: Restart (r) — Main Menu (m) — Quit (q)",
        ]);
        expected.set_style(Rect::new(0, 0, 80, 1), consts::SCORE_BAR_STYLE);
        expected.set_style(Rect::new(32, 8, 1, 1), consts::COLLISION_STYLE);
        expected.set_style(Rect::new(33, 8, 1, 1), consts::SNAKE_STYLE);
        expected.set_style(Rect::new(34, 8, 1, 1), consts::SNAKE_STYLE);
        expected.set_style(Rect::new(35, 8, 1, 1), consts::SNAKE_STYLE);
        expected.set_style(Rect::new(35, 9, 1, 1), consts::SNAKE_STYLE);
        expected.set_style(Rect::new(35, 10, 1, 1), consts::SNAKE_STYLE);
        expected.set_style(Rect::new(35, 11, 1, 1), consts::SNAKE_STYLE);
        expected.set_style(Rect::new(34, 11, 1, 1), consts::SNAKE_STYLE);
        expected.set_style(Rect::new(33, 11, 1, 1), consts::SNAKE_STYLE);
        expected.set_style(Rect::new(32, 11, 1, 1), consts::SNAKE_STYLE);
        expected.set_style(Rect::new(32, 10, 1, 1), consts::SNAKE_STYLE);
        expected.set_style(Rect::new(32, 9, 1, 1), consts::SNAKE_STYLE);
        expected.set_style(Rect::new(28, 10, 1, 1), consts::FRUIT_STYLE);
        expected.set_style(Rect::new(22, 23, 1, 1), consts::KEY_STYLE);
        expected.set_style(Rect::new(38, 23, 1, 1), consts::KEY_STYLE);
        expected.set_style(Rect::new(49, 23, 1, 1), consts::KEY_STYLE);
        pretty_assertions::assert_eq!(buffer, expected);
    }

    #[test]
    fn new_medium_game() {
        let mut globals = Globals::default();
        globals.options.level_size = LevelSize::Medium;
        let game = Game::new_with_rng(globals, ChaCha12Rng::seed_from_u64(RNG_SEED));
        let area = Rect::new(0, 0, 80, 24);
        let mut buffer = Buffer::empty(area);
        game.render(area, &mut buffer);
        let mut expected = Buffer::with_lines([
            " Score: 0",
            "",
            "",
            "",
            "",
            "             ┌─────────────────────────────────────────────────────┐            ",
            "             │                                                     │            ",
            "             │                                                     │            ",
            "             │                                                     │            ",
            "             │                                                     │            ",
            "             │                                                     │            ",
            "             │                                                     │            ",
            "             │                          v                          │            ",
            "             │                                                     │            ",
            "             │                                                     │            ",
            "             │                                                     │            ",
            "             │                                                     │            ",
            "             │                                                    ●│            ",
            "             └─────────────────────────────────────────────────────┘            ",
            "",
            "",
            "",
            "",
            "",
        ]);
        expected.set_style(Rect::new(0, 0, 80, 1), consts::SCORE_BAR_STYLE);
        expected.set_style(Rect::new(40, 12, 1, 1), consts::SNAKE_STYLE);
        expected.set_style(Rect::new(66, 17, 1, 1), consts::FRUIT_STYLE);
        pretty_assertions::assert_eq!(buffer, expected);
    }

    #[test]
    fn paused() {
        let mut game = Game::new_with_rng(Globals::default(), ChaCha12Rng::seed_from_u64(RNG_SEED));
        let area = Rect::new(0, 0, 80, 24);
        let mut buffer = Buffer::empty(area);
        assert!(game.handle_event(Event::Key(KeyCode::Esc.into())).is_none());
        game.render(area, &mut buffer);
        let mut expected = Buffer::with_lines([
            " Score: 0",
            " ┌────────────────────────────────────────────────────────────────────────────┐ ",
            " │                                                                            │ ",
            " │                                                                            │ ",
            " │                                                                            │ ",
            " │                                                                            │ ",
            " │                                                                            │ ",
            " │                                                                            │ ",
            " │                                                                            │ ",
            " │                             ┌──── PAUSED ─────┐                            │ ",
            " │                          ●  │ » Resume (Esc)  │                            │ ",
            " │                             │   Restart (r)   │                            │ ",
            " │                             │   Main Menu (m) │                            │ ",
            " │                             │   Quit (q)      │                            │ ",
            " │                             └─────────────────┘                            │ ",
            " │                                                                            │ ",
            " │                                                                            │ ",
            " │                                                                            │ ",
            " │                                                                            │ ",
            " │                                                                            │ ",
            " │                                                                            │ ",
            " └────────────────────────────────────────────────────────────────────────────┘ ",
            "",
            "",
        ]);
        expected.set_style(Rect::new(0, 0, 80, 1), consts::SCORE_BAR_STYLE);
        expected.set_style(Rect::new(28, 10, 1, 1), consts::FRUIT_STYLE);
        expected.set_style(Rect::new(43, 10, 3, 1), consts::KEY_STYLE);
        expected.set_style(Rect::new(33, 10, 15, 1), consts::MENU_SELECTION_STYLE);
        expected.set_style(Rect::new(44, 11, 1, 1), consts::KEY_STYLE);
        expected.set_style(Rect::new(46, 12, 1, 1), consts::KEY_STYLE);
        expected.set_style(Rect::new(41, 13, 1, 1), consts::KEY_STYLE);
        pretty_assertions::assert_eq!(buffer, expected);
    }
}
