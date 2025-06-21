use crate::consts;
use crossterm::event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers};
use rand::{seq::IteratorRandom, Rng};
use ratatui::{
    backend::Backend,
    buffer::Buffer,
    layout::{Flex, Layout, Margin, Position, Rect, Size},
    style::Style,
    text::Span,
    widgets::{Block, Widget},
    Terminal,
};
use std::collections::VecDeque;
use std::io;
use std::time::Instant;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct App<R> {
    rng: R,
    score: u32,
    snake_head: Position,
    snake_body: VecDeque<Position>,
    snake_len: usize,
    direction: Direction,
    fruit: Option<Position>,
    collision: Option<Position>,
    quitting: bool,
    level_size: Size,
    // walls: HashSet<Position>,
}

impl<R: Rng> App<R> {
    pub(crate) fn new(rng: R) -> App<R> {
        let mut app = App {
            rng,
            score: 0,
            snake_head: Position::new(20, 10),
            snake_body: VecDeque::new(),
            snake_len: consts::INITIAL_SNAKE_LENGTH,
            direction: Direction::North,
            fruit: None,
            collision: None,
            quitting: false,
            level_size: Size::new(40, 20),
        };
        app.place_fruit();
        app
    }

    pub(crate) fn run<B: Backend>(mut self, mut terminal: Terminal<B>) -> io::Result<()> {
        while !self.quitting() {
            self.draw(&mut terminal)?;
            if self.dead() {
                if let Some(ev) = read()?.as_key_press_event() {
                    if ev == KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)
                        || ev == KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)
                    {
                        break;
                    }
                }
            } else {
                self.tick()?;
            }
        }
        Ok(())
    }

    fn tick(&mut self) -> io::Result<()> {
        let mut wait = consts::TICK_DURATION;
        loop {
            let now = Instant::now();
            if poll(wait)? {
                self.handle_event(read()?);
                if self.quitting() {
                    return Ok(());
                }
                wait = wait.saturating_sub(now.elapsed());
            } else {
                self.advance();
                break;
            }
        }
        Ok(())
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

impl<R> App<R> {
    fn draw<B: Backend>(&self, terminal: &mut Terminal<B>) -> io::Result<()> {
        terminal.draw(|frame| frame.render_widget(self, frame.area()))?;
        Ok(())
    }

    fn handle_event(&mut self, event: Event) {
        let normal_modifiers = KeyModifiers::NONE | KeyModifiers::SHIFT;
        if let Some(KeyEvent {
            code, modifiers, ..
        }) = event.as_key_press_event()
        {
            if (modifiers, code) == (KeyModifiers::CONTROL, KeyCode::Char('c')) {
                self.quitting = true;
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
    }

    fn dead(&self) -> bool {
        self.collision.is_some()
    }

    fn quitting(&self) -> bool {
        self.quitting
    }
}

impl<R> Widget for &App<R> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [block_area] = Layout::horizontal([self.level_size.width.saturating_add(2)])
            .flex(Flex::Center)
            .areas(area);
        let [block_area] = Layout::vertical([self.level_size.height.saturating_add(2)])
            .flex(Flex::Center)
            .areas(block_area);
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
        if let Some(y) = block_area.y.checked_sub(1) {
            Span::from(format!("Score: {}", self.score)).render(
                Rect {
                    y,
                    height: 1,
                    ..block_area
                },
                buf,
            );
        }
        if self.dead() {
            let y = block_area.bottom();
            Span::from("Oh dear, you are dead!  Press ENTER to exit.").render(
                Rect {
                    x: block_area.x,
                    y,
                    height: 1,
                    width: 45,
                },
                buf,
            );
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
