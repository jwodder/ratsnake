use super::direction::Direction;
use super::levels::Bounds;
use crate::consts;
use ratatui::layout::Position;
use std::collections::VecDeque;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Snake {
    pub(super) head: Position,
    pub(super) body: VecDeque<Position>,
    pub(super) max_len: usize,
    pub(super) direction: Direction,
}

impl Snake {
    pub(super) fn new(head: Position, direction: Direction) -> Snake {
        Snake {
            head,
            body: VecDeque::new(),
            max_len: consts::INITIAL_SNAKE_LENGTH,
            direction,
        }
    }

    pub(super) fn head(&self) -> Position {
        self.head
    }

    pub(super) fn body(&self) -> &VecDeque<Position> {
        &self.body
    }

    pub(super) fn turn(&mut self, direction: Direction) {
        self.direction = direction;
    }

    // Returns `false` if it was unable to advance due to hitting an edge
    pub(super) fn advance(&mut self, bounds: Bounds) -> bool {
        let Some(pos) = self.direction.advance(self.head, bounds) else {
            return false;
        };
        self.body.push_back(self.head);
        self.head = pos;
        while self.body.len() > self.max_len {
            let _ = self.body.pop_front();
        }
        true
    }

    pub(super) fn grow(&mut self) {
        self.max_len += consts::SNAKE_GROWTH;
    }
}
