use super::direction::Direction;
use crate::consts;
use crate::util::Bounds;
use ratatui::layout::Position;
use std::collections::VecDeque;

/// Snake state.  Snate.
///
/// All positions are relative to the top-left corner of the level the snake is
/// on.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Snake {
    /// The position of the snake's head
    pub(super) head: Position,

    /// The positions of all of the cells in the snake's body, with the most
    /// recent at the end.
    pub(super) body: VecDeque<Position>,

    /// The maximum length of the body
    pub(super) max_len: usize,

    /// The direction in which the snake is currently facing
    pub(super) direction: Direction,
}

impl Snake {
    /// Create a new snake with its head at `head` and facing in `direction`.
    /// The body will be empty, and `max_len` will be set to
    /// [`INITIAL_SNAKE_LENGTH`][consts::INITIAL_SNAKE_LENGTH].
    pub(super) fn new(head: Position, direction: Direction) -> Snake {
        Snake {
            head,
            body: VecDeque::new(),
            max_len: consts::INITIAL_SNAKE_LENGTH,
            direction,
        }
    }

    /// Return the position of the snake's head
    pub(super) fn head(&self) -> Position {
        self.head
    }

    /// Return the glyph to use for drawing the snake's head
    pub(super) fn head_symbol(&self) -> char {
        match self.direction {
            Direction::North => consts::SNAKE_HEAD_NORTH_SYMBOL,
            Direction::South => consts::SNAKE_HEAD_SOUTH_SYMBOL,
            Direction::East => consts::SNAKE_HEAD_EAST_SYMBOL,
            Direction::West => consts::SNAKE_HEAD_WEST_SYMBOL,
        }
    }

    /// Return the positions of the cells in the snake's body
    pub(super) fn body(&self) -> &VecDeque<Position> {
        &self.body
    }

    /// Change the snake's direction to `direction`
    pub(super) fn turn(&mut self, direction: Direction) {
        self.direction = direction;
    }

    /// Move the snake forwards one cell in the current direction within
    /// `bounds`.  Returns `false` if the snake was unable to advance due to
    /// hitting a non-wraparound edge.
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

    /// Extend the snake's maximum length in response to eating a fruit
    pub(super) fn grow(&mut self) {
        self.max_len += consts::SNAKE_GROWTH;
    }
}
