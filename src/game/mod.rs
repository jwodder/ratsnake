mod direction;
mod levels;
mod snake;
use self::direction::Direction;
use self::levels::{Bounds, LevelMap};
use self::snake::Snake;
use crate::app::AppState;
use crate::command::Command;
use crate::consts;
use crate::options::Options;
use crate::util::get_display_area;
use crossterm::event::{poll, read, Event};
use rand::{seq::IteratorRandom, Rng};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Margin, Position, Rect},
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
    dead: bool,
    map: LevelMap,
}

impl Game<rand::rngs::ThreadRng> {
    pub(crate) fn new(options: Options) -> Self {
        Game::new_with_rng(options, rand::rng())
    }
}

impl<R: Rng> Game<R> {
    pub(crate) fn new_with_rng(options: Options, mut rng: R) -> Game<R> {
        let mut map = LevelMap::new(Bounds::from((
            options.level_size.as_size(),
            options.wraparound,
        )));
        if options.obstacles {
            map.set_obstacles(&mut rng);
        }
        let snake = map.new_snake();
        let mut game = Game {
            rng,
            score: 0,
            snake,
            fruits: HashSet::new(),
            dead: false,
            map,
        };
        for _ in 0..options.fruits.get() {
            game.place_fruit();
        }
        game
    }

    pub(crate) fn process_input(&mut self) -> std::io::Result<Option<AppState>> {
        if self.dead {
            if let Some(ev) = read()?.as_key_press_event() {
                if matches!(
                    Command::from_key_event(ev),
                    Some(Command::Quit | Command::Enter)
                ) {
                    return Ok(Some(AppState::Quit));
                }
            }
        } else {
            self.tick()?;
        }
        Ok(None)
    }

    fn tick(&mut self) -> std::io::Result<Option<AppState>> {
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
        if self.dead {
            return;
        }
        if !self.snake.advance(self.map.bounds()) {
            self.dead = true;
            return;
        }
        if self.fruits.remove(&self.snake.head()) {
            self.score += 1;
            self.snake.grow();
            self.place_fruit();
        } else if self.snake.body().contains(&self.snake.head())
            || self.map.obstacles().contains(&self.snake.head())
        {
            self.dead = true;
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

    fn handle_event(&mut self, event: Event) -> Option<AppState> {
        match Command::from_key_event(event.as_key_press_event()?)? {
            Command::Quit => return Some(AppState::Quit),
            Command::Up => self.snake.turn(Direction::North),
            Command::Left => self.snake.turn(Direction::West),
            Command::Down => self.snake.turn(Direction::South),
            Command::Right => self.snake.turn(Direction::East),
            _ => (),
        }
        None
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

        let [block_area] = Layout::horizontal([self.map.size().width.saturating_add(2)])
            .flex(Flex::Center)
            .areas(block_area);
        let [block_area] = Layout::vertical([self.map.size().height.saturating_add(2)])
            .flex(Flex::Center)
            .areas(block_area);
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
        if self.dead {
            level.draw_cell(
                self.snake.head(),
                consts::COLLISION_SYMBOL,
                consts::COLLISION_STYLE,
            );
        } else {
            level.draw_cell(
                self.snake.head(),
                consts::SNAKE_HEAD_SYMBOL,
                consts::SNAKE_STYLE,
            );
        }

        if self.dead {
            Span::from(" Oh dear, you are dead!").render(msg1_area, buf);
            Span::from(" Press ENTER to exit.").render(msg2_area, buf);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::LevelSize;
    use rand::SeedableRng;
    use rand_chacha::ChaCha12Rng;
    use std::collections::VecDeque;

    const RNG_SEED: u64 = 0x0123456789ABCDEF;

    #[test]
    fn new_game() {
        let game = Game::new_with_rng(Options::default(), ChaCha12Rng::seed_from_u64(RNG_SEED));
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
        expected.set_style(Rect::new(0, 0, 80, 1), consts::SCORE_BAR_STYLE);
        expected.set_style(Rect::new(40, 11, 1, 1), consts::SNAKE_STYLE);
        expected.set_style(Rect::new(28, 10, 1, 1), consts::FRUIT_STYLE);
        pretty_assertions::assert_eq!(buffer, expected);
    }

    #[test]
    fn new_wraparound_game() {
        let game = Game::new_with_rng(
            Options {
                wraparound: true,
                ..Options::default()
            },
            ChaCha12Rng::seed_from_u64(RNG_SEED),
        );
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
            " ⋮                                      @                                     ⋮ ",
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
        let mut game = Game::new_with_rng(Options::default(), ChaCha12Rng::seed_from_u64(RNG_SEED));
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
        game.dead = true;
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
        pretty_assertions::assert_eq!(buffer, expected);
    }

    #[test]
    fn new_medium_game() {
        let game = Game::new_with_rng(
            Options {
                level_size: LevelSize::Medium,
                ..Options::default()
            },
            ChaCha12Rng::seed_from_u64(RNG_SEED),
        );
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
            "             │                          @                          │            ",
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
}
