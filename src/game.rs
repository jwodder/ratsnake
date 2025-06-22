use crate::app::AppState;
use crate::command::Command;
use crate::consts;
use crate::options::Options;
use crate::util::get_display_area;
use crossterm::event::{poll, read, Event};
use rand::{
    distr::{Bernoulli, Distribution},
    seq::IteratorRandom,
    Rng,
};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Margin, Position, Rect, Size},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Widget},
    Frame,
};
use std::collections::{HashSet, VecDeque};
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
    fruits: HashSet<Position>,
    dead: bool,
    level_size: Size,
    obstacles: HashSet<Position>,
    wraparound: bool,
}

impl Game<rand::rngs::ThreadRng> {
    pub(crate) fn new(options: Options) -> Self {
        Game::new_with_rng(options, rand::rng())
    }
}

impl<R: Rng> Game<R> {
    pub(crate) fn new_with_rng(options: Options, mut rng: R) -> Game<R> {
        let level_size = options.level_size.as_size();
        let snake_head = Position::new(level_size.width / 2, level_size.height / 2);
        let obstacles = if options.obstacles {
            let dist = Bernoulli::new(consts::OBSTACLE_PROBABILITY)
                .expect("OBSTACLE_PROBABILITY should be between 0 and 1");
            let mut obsts = HashSet::from_iter(
                Rect::from((Position::ORIGIN, level_size))
                    .positions()
                    .zip(dist.sample_iter(&mut rng))
                    .filter_map(|(pos, f)| f.then_some(pos)),
            );
            for y in std::iter::successors(Some(snake_head.y), |y| y.checked_add(1))
                .take(consts::BACKWARDS_CLEARANCE)
            {
                obsts.remove(&Position::new(snake_head.x, y));
            }
            for y in std::iter::successors(Some(snake_head.y), |y| y.checked_sub(1))
                .take(consts::FORWARDS_CLEARANCE)
            {
                obsts.remove(&Position::new(snake_head.x, y));
            }
            obsts
        } else {
            HashSet::new()
        };
        let mut game = Game {
            rng,
            score: 0,
            snake_body: VecDeque::new(),
            snake_head,
            snake_len: consts::INITIAL_SNAKE_LENGTH,
            direction: Direction::North,
            fruits: HashSet::new(),
            dead: false,
            level_size,
            obstacles,
            wraparound: options.wraparound,
        };
        for _ in 0..options.fruits {
            game.place_fruit();
        }
        game
    }

    pub(crate) fn process_input(&mut self) -> io::Result<Option<AppState>> {
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
        if self.dead {
            return;
        }
        if let Some(pos) = self
            .direction
            .advance(self.snake_head, self.level_size, self.wraparound)
        {
            self.snake_body.push_back(self.snake_head);
            self.snake_head = pos;
        } else {
            self.dead = true;
            return;
        }
        while self.snake_body.len() > self.snake_len {
            let _ = self.snake_body.pop_front();
        }
        if self.fruits.remove(&self.snake_head) {
            self.score += 1;
            self.snake_len += consts::SNAKE_GROWTH;
            self.place_fruit();
        } else if self.snake_body.contains(&self.snake_head)
            || self.obstacles.contains(&self.snake_head)
        {
            self.dead = true;
        }
    }

    fn place_fruit(&mut self) {
        let mut occupied = &self.fruits | &self.obstacles;
        occupied.insert(self.snake_head);
        occupied.extend(self.snake_body.iter().copied());
        self.fruits.extend(
            Rect::from((Position::ORIGIN, self.level_size))
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
            Command::Up => self.direction = Direction::North,
            Command::Left => self.direction = Direction::West,
            Command::Down => self.direction = Direction::South,
            Command::Right => self.direction = Direction::East,
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

        let [block_area] = Layout::horizontal([self.level_size.width.saturating_add(2)])
            .flex(Flex::Center)
            .areas(block_area);
        let [block_area] = Layout::vertical([self.level_size.height.saturating_add(2)])
            .flex(Flex::Center)
            .areas(block_area);
        if self.wraparound {
            DottedBorder.render(block_area, buf);
        } else {
            Block::bordered().render(block_area, buf);
        }

        let level_area = block_area.inner(Margin::new(1, 1));
        let mut level = Canvas {
            area: level_area,
            buf,
        };
        for &p in &self.snake_body {
            level.draw_cell(p, consts::SNAKE_BODY_SYMBOL, consts::SNAKE_STYLE);
        }
        for &pos in &self.fruits {
            level.draw_cell(pos, consts::FRUIT_SYMBOL, consts::FRUIT_STYLE);
        }
        for &pos in &self.obstacles {
            level.draw_cell(pos, consts::OBSTACLE_SYMBOL, consts::OBSTACLE_STYLE);
        }
        // Draw the head last so that, if it's a collision, we overwrite
        // whatever it's colliding with
        if self.dead {
            level.draw_cell(
                self.snake_head,
                consts::COLLISION_SYMBOL,
                consts::COLLISION_STYLE,
            );
        } else {
            level.draw_cell(
                self.snake_head,
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
enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    fn advance(self, pos: Position, size: Size, wraparound: bool) -> Option<Position> {
        let Position { mut x, mut y } = pos;
        match self {
            Direction::North => {
                y = decrement_in_bounds(y, size.height, wraparound)?;
            }
            Direction::East => {
                x = increment_in_bounds(x, size.width, wraparound)?;
            }
            Direction::South => {
                y = increment_in_bounds(y, size.height, wraparound)?;
            }
            Direction::West => {
                x = decrement_in_bounds(x, size.width, wraparound)?;
            }
        }
        Some(Position { x, y })
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

fn decrement_in_bounds(x: u16, max: u16, wrap: bool) -> Option<u16> {
    if let Some(x2) = x.checked_sub(1) {
        Some(x2)
    } else if wrap {
        Some(max - 1)
    } else {
        None
    }
}

fn increment_in_bounds(x: u16, max: u16, wrap: bool) -> Option<u16> {
    if let Some(x2) = x.checked_add(1).filter(|&xx| xx < max) {
        Some(x2)
    } else if wrap {
        Some(0)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    mod render_game {
        use super::*;
        use crate::options::LevelSize;
        use rand::SeedableRng;
        use rand_chacha::ChaCha12Rng;

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
            let mut game =
                Game::new_with_rng(Options::default(), ChaCha12Rng::seed_from_u64(RNG_SEED));
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

    #[rstest]
    #[case(
        Direction::North,
        Position::new(2, 7),
        Size::new(10, 15),
        false,
        Some(Position::new(2, 6))
    )]
    #[case(
        Direction::South,
        Position::new(2, 7),
        Size::new(10, 15),
        false,
        Some(Position::new(2, 8))
    )]
    #[case(
        Direction::East,
        Position::new(2, 7),
        Size::new(10, 15),
        false,
        Some(Position::new(3, 7))
    )]
    #[case(
        Direction::West,
        Position::new(2, 7),
        Size::new(10, 15),
        false,
        Some(Position::new(1, 7))
    )]
    #[case(Direction::North, Position::new(2, 0), Size::new(10, 15), false, None)]
    #[case(
        Direction::North,
        Position::new(2, 0),
        Size::new(10, 15),
        true,
        Some(Position::new(2, 14))
    )]
    #[case(Direction::South, Position::new(2, 14), Size::new(10, 15), false, None)]
    #[case(
        Direction::South,
        Position::new(2, 14),
        Size::new(10, 15),
        true,
        Some(Position::new(2, 0))
    )]
    #[case(Direction::East, Position::new(9, 7), Size::new(10, 15), false, None)]
    #[case(
        Direction::East,
        Position::new(9, 7),
        Size::new(10, 15),
        true,
        Some(Position::new(0, 7))
    )]
    #[case(Direction::West, Position::new(0, 7), Size::new(10, 15), false, None)]
    #[case(
        Direction::West,
        Position::new(0, 7),
        Size::new(10, 15),
        true,
        Some(Position::new(9, 7))
    )]
    fn test_direction_advance(
        #[case] d: Direction,
        #[case] pos: Position,
        #[case] size: Size,
        #[case] wrap: bool,
        #[case] r: Option<Position>,
    ) {
        assert_eq!(d.advance(pos, size, wrap), r);
    }
}
