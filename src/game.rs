use crate::app::AppState;
use crate::consts;
use crate::options::Options;
use crate::util::{get_display_area, RectExt, Side};
use crossterm::event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers};
use rand::{seq::IteratorRandom, Rng};
use ratatui::{
    buffer::Buffer,
    layout::{Margin, Position, Rect, Size},
    style::Style,
    text::Span,
    widgets::{Block, Widget},
    Frame,
};
use std::collections::VecDeque;
use std::io;
use std::time::Instant;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Game<R = rand::rngs::ThreadRng> {
    rng: R,
    score: u32,
    snake_head: Position,
    snake_body: VecDeque<Position>,
    snake_len: usize,
    direction: Direction,
    fruit: Option<Position>,
    collision: Option<Position>,
    level_size: Size,
    // walls: HashSet<Position>,
}

impl<R: Rng> Game<R> {
    pub(crate) fn new(options: Options, rng: R) -> Game<R> {
        // TODO: Use options
        let level_size = consts::LEVEL_SIZE;
        let mut game = Game {
            rng,
            score: 0,
            snake_head: Position::new(level_size.width / 2, level_size.height / 2),
            snake_body: VecDeque::new(),
            snake_len: consts::INITIAL_SNAKE_LENGTH,
            direction: Direction::North,
            fruit: None,
            collision: None,
            level_size,
        };
        game.place_fruit();
        game
    }

    pub(crate) fn process_input(&mut self) -> io::Result<Option<AppState>> {
        if self.dead() {
            if let Some(ev) = read()?.as_key_press_event() {
                if ev == KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)
                    || ev == KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)
                {
                    return Ok(Some(AppState::Quit));
                }
            }
        } else {
            self.tick()?;
        }
        Ok(None)
    }

    fn tick(&mut self) -> io::Result<Option<AppState>> {
        let mut wait = consts::TICK_DURATION;
        loop {
            let now = Instant::now();
            if poll(wait)? {
                if let st @ Some(_) = self.handle_event(read()?) {
                    return Ok(st);
                }
                wait = wait.saturating_sub(now.elapsed());
            } else {
                self.advance();
                break;
            }
        }
        Ok(None)
    }

    fn advance(&mut self) {
        if self.dead() {
            return;
        }
        let Position { mut x, mut y } = self.snake_head;
        match self.direction {
            Direction::North => {
                let Some(y2) = y.checked_sub(1) else {
                    self.collision = Some(self.snake_head);
                    return;
                };
                y = y2;
            }
            Direction::East => {
                let Some(x2) = x.checked_add(1) else {
                    self.collision = Some(self.snake_head);
                    return;
                };
                x = x2;
            }
            Direction::South => {
                let Some(y2) = y.checked_add(1) else {
                    self.collision = Some(self.snake_head);
                    return;
                };
                y = y2;
            }
            Direction::West => {
                let Some(x2) = x.checked_sub(1) else {
                    self.collision = Some(self.snake_head);
                    return;
                };
                x = x2;
            }
        }
        if x >= self.level_size.width || y >= self.level_size.height {
            self.collision = Some(self.snake_head);
            return;
        }
        self.snake_body.push_back(self.snake_head);
        self.snake_head = Position { x, y };
        while self.snake_body.len() > self.snake_len {
            let _ = self.snake_body.pop_front();
        }
        if Some(self.snake_head) == self.fruit {
            self.fruit = None;
            self.score += 1;
            self.snake_len += consts::SNAKE_GROWTH;
            self.place_fruit();
        } else if self.snake_body.contains(&self.snake_head) {
            self.collision = Some(self.snake_head);
        }
        // TODO later: Check for collision with walls
    }

    fn place_fruit(&mut self) {
        self.fruit = Rect::from((Position::ORIGIN, self.level_size))
            .positions()
            // TODO later: exclude walls from consideration
            .filter(|&p| p != self.snake_head && !self.snake_body.contains(&p))
            .choose(&mut self.rng);
    }
}

impl<R> Game<R> {
    pub(crate) fn draw(&self, frame: &mut Frame<'_>) {
        frame.render_widget(self, frame.area());
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
                match code {
                    KeyCode::Char('w' | 'k') | KeyCode::Up => self.direction = Direction::North,
                    KeyCode::Char('a' | 'h') | KeyCode::Left => self.direction = Direction::West,
                    KeyCode::Char('s' | 'j') | KeyCode::Down => self.direction = Direction::South,
                    KeyCode::Char('d' | 'l') | KeyCode::Right => self.direction = Direction::East,
                    _ => (),
                }
            }
        }
        None
    }

    fn dead(&self) -> bool {
        self.collision.is_some()
    }
}

impl<R> Widget for &Game<R> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let display = get_display_area(area).shave(Side::Left).shave(Side::Right);
        Span::from(format!("Score: {}", self.score)).render(
            Rect {
                height: 1,
                ..display
            },
            buf,
        );
        let block_area = display
            .shave(Side::Top)
            .shave(Side::Bottom)
            .shave(Side::Bottom);
        let level_area = block_area.inner(Margin::new(1, 1));
        let mut level = Canvas {
            area: level_area,
            buf,
        };
        level.draw_cell(
            self.snake_head,
            consts::SNAKE_HEAD_SYMBOL,
            consts::SNAKE_STYLE,
        );
        for &p in &self.snake_body {
            level.draw_cell(p, consts::SNAKE_BODY_SYMBOL, consts::SNAKE_STYLE);
        }
        if let Some(pos) = self.fruit {
            level.draw_cell(pos, consts::FRUIT_SYMBOL, consts::FRUIT_STYLE);
        }
        // TODO later: Draw walls
        if let Some(pos) = self.collision {
            level.draw_cell(pos, consts::COLLISION_SYMBOL, consts::COLLISION_STYLE);
        }
        Block::bordered().render(block_area, buf);
        if self.dead() {
            let y = block_area.bottom();
            Span::from("Oh dear, you are dead!").render(
                Rect {
                    y,
                    height: 1,
                    ..display
                },
                buf,
            );
            if let Some(y) = y.checked_add(1) {
                Span::from("Press ENTER to exit.").render(
                    Rect {
                        y,
                        height: 1,
                        ..display
                    },
                    buf,
                );
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
    fn draw_cell(&mut self, pos: Position, symbol: char, style: Style) {
        let Some(x) = self.area.x.checked_add(pos.x) else {
            return;
        };
        let Some(y) = self.area.y.checked_add(pos.y) else {
            return;
        };
        if let Some(cell) = self.buf.cell_mut((x, y)) {
            cell.set_char(symbol);
            cell.set_style(style);
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Direction {
    North,
    East,
    South,
    West,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha12Rng;

    const RNG_SEED: u64 = 0x0123456789ABCDEF;

    #[test]
    fn draw_startup() {
        let game = Game::new(Options::default(), ChaCha12Rng::seed_from_u64(RNG_SEED));
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
            " │                                      @                                     │ ",
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
        expected.set_style(Rect::new(40, 11, 1, 1), consts::SNAKE_STYLE);
        expected.set_style(Rect::new(28, 10, 1, 1), consts::FRUIT_STYLE);
        assert_eq!(buffer, expected);
    }

    #[test]
    fn draw_self_collision() {
        let mut game = Game::new(Options::default(), ChaCha12Rng::seed_from_u64(RNG_SEED));
        game.score = 3;
        game.snake_head = Position::new(30, 6);
        game.snake_body = VecDeque::from([
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
        game.snake_len = 12;
        game.direction = Direction::North;
        game.collision = Some(Position::new(30, 6));
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
            " │                              *~~~                                          │ ",
            " │                              ~  ~                                          │ ",
            " │                          ●   ~  ~                                          │ ",
            " │                              ~~~~                                          │ ",
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
            " Oh dear, you are dead!",
            " Press ENTER to exit.",
        ]);
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
        assert_eq!(buffer, expected);
    }
}
