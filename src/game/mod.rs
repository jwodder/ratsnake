mod levels;
mod paused;
mod snake;
use self::levels::LevelMap;
use self::paused::{PauseOpt, Paused};
use self::snake::Snake;
use crate::app::Screen;
use crate::command::Command;
use crate::consts;
use crate::direction::Direction;
use crate::util::{center_rect, get_display_area, Globals};
use crate::warning::{Warning, WarningOutcome};
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
use std::num::NonZeroU32;
use std::time::Instant;

/// Snake game screen
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Game<R = rand::rngs::ThreadRng> {
    /// The random-number generator used for generating obstacles and fruit
    rng: R,

    /// The current score, equal to the number of fruits eaten
    score: u32,

    /// The current high score for the current gameplay options.
    ///
    /// If `score` exceeds this value, `high_score` is not updated.
    high_score: Option<NonZeroU32>,

    /// The state of the snake itself
    snake: Snake,

    /// The positions of the fruits in the level
    fruits: HashSet<Position>,

    /// The state that the game is currently in
    state: GameState,

    /// The map of the game level
    map: LevelMap,

    /// Global data (options & high scores)
    globals: Globals,

    /// The next time at which the snake should move forwards.  If `None`, the
    /// next value will be calculated on the next call to
    /// [`Game::process_input()`]
    next_tick: Option<Instant>,
}

impl Game<rand::rngs::ThreadRng> {
    /// Create a new game from the given globals using the thread RNG
    pub(crate) fn new(globals: Globals) -> Self {
        Game::new_with_rng(globals, rand::rng())
    }
}

impl<R: Rng> Game<R> {
    /// Create a new game from the given globals using the given RNG
    pub(crate) fn new_with_rng(globals: Globals, mut rng: R) -> Game<R> {
        let mut map = LevelMap::new(globals.options.level_bounds());
        if globals.options.obstacles {
            map.set_obstacles(&mut rng);
        }
        let snake = map.new_snake();
        let fruit_qty = globals.options.fruits.get();
        let high_score = globals.high_scores.get(globals.options);
        let mut game = Game {
            rng,
            score: 0,
            high_score,
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

    /// Receive & handle the next input event.  If the game is currently
    /// running and no event is received before [`Game::next_tick`] passes, the
    /// snake advances and the method returns.
    ///
    /// Returns `Some(screen)` if the application should switch to a different
    /// screen or quit.
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

    /// Move the snake forwards and respond to any fruits or obstacles it came
    /// into contact with
    fn advance(&mut self) {
        if !self.running() {
            return;
        }
        if !self.snake.advance(self.map.bounds()) {
            self.state = GameState::Dead(self.finalize_score());
            return;
        }
        if self.fruits.remove(&self.snake.head()) {
            self.score += 1;
            self.snake.grow();
            self.place_fruit();
        } else if self.snake.body().contains(&self.snake.head())
            || self.map.obstacles().contains(&self.snake.head())
        {
            self.state = GameState::Dead(self.finalize_score());
        }
        if self.fruits.is_empty() {
            self.state = GameState::Exhausted(self.finalize_score());
        }
    }

    /// Place a fruit at a randomly-selected empty position in the level, if
    /// any
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
    /// Draw the game on the given frame
    pub(crate) fn draw(&self, frame: &mut Frame<'_>) {
        frame.render_widget(self, frame.area());
    }

    /// Handle the given input event.
    ///
    /// Returns `Some(screen)` if the application should switch to a different
    /// screen or quit.
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
            GameState::Dead(PostMortem {
                ref mut warning, ..
            })
            | GameState::Exhausted(PostMortem {
                ref mut warning, ..
            }) => {
                let cmd = Command::from_key_event(event.as_key_press_event()?)?;
                if let Some(wrn) = warning {
                    match wrn.handle_command(cmd)? {
                        WarningOutcome::Dismissed => *warning = None,
                        WarningOutcome::Quit => return Some(Screen::Quit),
                    }
                } else {
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
        }
        None
    }

    /// Check for a new high score and, if there is one, update the high scores
    /// and write them to disk.
    ///
    /// Any errors that occur while updating the high score file are converted
    /// into a [`Warning`] for display.
    fn finalize_score(&mut self) -> PostMortem {
        if let Some(score) = self.new_high_score() {
            self.globals.high_scores.set(self.globals.options, score);
            let warning = self
                .globals
                .config
                .save_high_scores(&self.globals.high_scores)
                .err()
                .map(Warning::from);
            PostMortem {
                new_high_score: true,
                warning,
            }
        } else {
            PostMortem {
                new_high_score: false,
                warning: None,
            }
        }
    }

    /// If the score exceeds the current high score, return the new high score.
    fn new_high_score(&self) -> Option<NonZeroU32> {
        NonZeroU32::new(self.score).filter(|&score| self.high_score.is_none_or(|hs| hs < score))
    }

    /// Is the game currently running (and not paused or over?)
    fn running(&self) -> bool {
        self.state == GameState::Running
    }

    /// Pause the game
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

        let hs_str = match self.high_score {
            Some(hs) => format!("High Score: {hs} "),
            None => String::from("High Score: - "),
        };
        Line::styled(hs_str, consts::SCORE_BAR_STYLE)
            .right_aligned()
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

        let glyphs = &self.globals.config.glyphs;
        let level_area = block_area.inner(Margin::new(1, 1));
        let mut level = Canvas {
            area: level_area,
            buf,
        };
        for &p in self.snake.body() {
            level.draw_cell(p, &glyphs.snake_body.symbol, glyphs.snake_body.style);
        }
        for &pos in &self.fruits {
            level.draw_cell(pos, &glyphs.fruit.symbol, glyphs.fruit.style);
        }
        for &pos in self.map.obstacles() {
            level.draw_cell(pos, &glyphs.obstacle.symbol, glyphs.obstacle.style);
        }
        // Draw the head last so that, if it's a collision, we overwrite
        // whatever it's colliding with
        if matches!(self.state, GameState::Dead(_)) {
            level.draw_cell(
                self.snake.head(),
                &glyphs.collision.symbol,
                glyphs.collision.style,
            );
        } else {
            level.draw_cell(
                self.snake.head(),
                glyphs.snake_head.symbol.for_direction(self.snake.direction),
                glyphs.snake_head.style,
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
            GameState::Dead(ref pm) | GameState::Exhausted(ref pm) => {
                Span::from(if pm.new_high_score {
                    " — GAME OVER — NEW HIGH SCORE! —"
                } else {
                    " — GAME OVER —"
                })
                .render(msg1_area, buf);
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
                if let Some(ref warning) = pm.warning {
                    warning.render(display, buf);
                }
            }
        }
    }
}

/// A portion of a [`Buffer`] that provides methods for drawing individual
/// cells using coordinates relative to the top-left corner of `area`
#[derive(Debug, Eq, PartialEq)]
struct Canvas<'a> {
    area: Rect,
    buf: &'a mut Buffer,
}

impl Canvas<'_> {
    /// Set the cell at `pos` to `symbol`
    fn draw_char(&mut self, pos: Position, symbol: char) {
        let Some(x) = self.area.x.checked_add(pos.x) else {
            return;
        };
        let Some(y) = self.area.y.checked_add(pos.y) else {
            return;
        };
        if let Some(cell) = self.buf.cell_mut((x, y)) {
            cell.set_char(symbol);
            cell.set_style(Style::reset());
        }
    }

    /// Set the cell at `pos` to `symbol` with the given style
    fn draw_cell<S: AsRef<str>>(&mut self, pos: Position, symbol: S, style: Style) {
        let Some(x) = self.area.x.checked_add(pos.x) else {
            return;
        };
        let Some(y) = self.area.y.checked_add(pos.y) else {
            return;
        };
        if let Some(cell) = self.buf.cell_mut((x, y)) {
            cell.set_symbol(symbol.as_ref());
            cell.set_style(Style::reset().patch(style));
        }
    }
}

/// A widget for drawing a border made of dots around the edge of an area.
///
/// Like [`Block::bordered()`], but with different characters.
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

/// An enum of the states that a game can be in
#[derive(Clone, Debug, Eq, PartialEq)]
enum GameState {
    /// The game is currently running
    Running,

    /// The game is currently paused
    Paused(Paused),

    /// The game ended due to the snake colliding with something
    Dead(PostMortem),

    /// The snake has filled the board and there are no more spaces to place
    /// fruits in
    Exhausted(PostMortem),
}

/// End-of-game report
#[derive(Clone, Debug, Eq, PartialEq)]
struct PostMortem {
    /// True if a new high score was set
    new_high_score: bool,

    /// A warning to display about an error, if any, that occurred while
    /// updating the high score file
    warning: Option<Warning>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::LevelSize;
    use crossterm::event::KeyCode;
    use rand::SeedableRng;
    use rand_chacha::ChaCha12Rng;
    use rstest::rstest;
    use std::collections::VecDeque;

    const RNG_SEED: u64 = 0x0123456789ABCDEF;

    #[test]
    fn new_game() {
        let game = Game::new_with_rng(Globals::default(), ChaCha12Rng::seed_from_u64(RNG_SEED));
        let area = Rect::new(0, 0, 80, 24);
        let mut buffer = Buffer::empty(area);
        game.render(area, &mut buffer);
        let mut expected = Buffer::with_lines([
            " Score: 0                                                         High Score: - ",
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
            " Score: 0                                                         High Score: - ",
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
    fn new_game_with_high_score() {
        let mut globals = Globals::default();
        globals
            .high_scores
            .set(globals.options, NonZeroU32::new(42).unwrap());
        let game = Game::new_with_rng(globals, ChaCha12Rng::seed_from_u64(RNG_SEED));
        let area = Rect::new(0, 0, 80, 24);
        let mut buffer = Buffer::empty(area);
        game.render(area, &mut buffer);
        let mut expected = Buffer::with_lines([
            " Score: 0                                                        High Score: 42 ",
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
        game.state = GameState::Dead(PostMortem {
            new_high_score: false,
            warning: None,
        });
        let area = Rect::new(0, 0, 80, 24);
        let mut buffer = Buffer::empty(area);
        game.render(area, &mut buffer);
        let mut expected = Buffer::with_lines([
            " Score: 3                                                         High Score: - ",
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
    fn self_collision_new_high_score() {
        let mut globals = Globals::default();
        globals
            .high_scores
            .set(globals.options, NonZeroU32::new(2).unwrap());
        let mut game = Game::new_with_rng(globals, ChaCha12Rng::seed_from_u64(RNG_SEED));
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
        game.state = GameState::Dead(PostMortem {
            new_high_score: true,
            warning: None,
        });
        let area = Rect::new(0, 0, 80, 24);
        let mut buffer = Buffer::empty(area);
        game.render(area, &mut buffer);
        let mut expected = Buffer::with_lines([
            " Score: 3                                                         High Score: 2 ",
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
            " — GAME OVER — NEW HIGH SCORE! —",
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
            " Score: 0                                                         High Score: - ",
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
            " Score: 0                                                         High Score: - ",
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

    #[rstest]
    #[case(0, None, false)]
    #[case(1, None, true)]
    #[case(42, None, true)]
    #[case(0, NonZeroU32::new(1), false)]
    #[case(1, NonZeroU32::new(1), false)]
    #[case(1, NonZeroU32::new(42), false)]
    #[case(42, NonZeroU32::new(42), false)]
    #[case(42, NonZeroU32::new(1), true)]
    fn test_new_high_score(
        #[case] score: u32,
        #[case] old_high_score: Option<NonZeroU32>,
        #[case] new: bool,
    ) {
        let mut game = Game::new(Globals::default());
        game.score = score;
        game.high_score = old_high_score;
        if new {
            let hs = NonZeroU32::new(score);
            assert!(hs.is_some());
            assert_eq!(game.new_high_score(), hs);
        } else {
            assert_eq!(game.new_high_score(), None);
        }
    }
}
