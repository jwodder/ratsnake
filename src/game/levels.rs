use super::snake::Snake;
use crate::consts;
use crate::direction::Direction;
use crate::util::Bounds;
use rand::{
    distr::{Bernoulli, Distribution},
    Rng,
};
use ratatui::layout::{Position, Size};
use std::collections::HashSet;

/// A map of a game level
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct LevelMap {
    /// The level's bounds (size and wraparound)
    bounds: Bounds,

    /// The locations of any obstacles in the level
    obstacles: HashSet<Position>,

    /// The snake's starting position and direction
    snake_start: (Position, Direction),
}

impl LevelMap {
    /// Create a new level with the given bounds, no obstacles, and the snake
    /// located in the center facing north
    pub(super) fn new(bounds: Bounds) -> LevelMap {
        let snake_head = Position::new(bounds.width / 2, bounds.height / 2);
        LevelMap {
            bounds,
            obstacles: HashSet::new(),
            snake_start: (snake_head, Direction::North),
        }
    }

    /// Populate the level with randomly-generated obstacles using the given
    /// RNG.  Any previously-generated obstacles are discarded.
    pub(super) fn set_obstacles<R: Rng>(&mut self, rng: R) {
        let dist = Bernoulli::new(consts::OBSTACLE_PROBABILITY)
            .expect("OBSTACLE_PROBABILITY should be between 0 and 1");
        self.obstacles = HashSet::from_iter(
            self.bounds
                .positions()
                .zip(dist.sample_iter(rng))
                .filter_map(|(pos, f)| f.then_some(pos)),
        );
        let (snake_head, snake_dir) = self.snake_start;
        for pos in std::iter::successors(Some(snake_head), |&p| snake_dir.advance(p, self.bounds))
            .take(consts::FORWARDS_CLEARANCE)
        {
            self.obstacles.remove(&pos);
        }
        let rid_ekans = snake_dir.reverse();
        for pos in std::iter::successors(Some(snake_head), |&p| rid_ekans.advance(p, self.bounds))
            .take(consts::BACKWARDS_CLEARANCE)
        {
            self.obstacles.remove(&pos);
        }
    }

    /// Return a new `Snake` value with this level's starting location &
    /// direction
    pub(super) fn new_snake(&self) -> Snake {
        let (head, direction) = self.snake_start;
        Snake::new(head, direction)
    }

    /// Return the level's bounds
    pub(super) fn bounds(&self) -> Bounds {
        self.bounds
    }

    /// Return the locations of any obstacles in the level
    pub(super) fn obstacles(&self) -> &HashSet<Position> {
        &self.obstacles
    }

    /// Return the level's size
    pub(super) fn size(&self) -> Size {
        self.bounds.size()
    }

    /// Return `true` if wraparound is enabled for the level, `false` otherwise
    pub(super) fn wrap(&self) -> bool {
        self.bounds.wrap
    }
}
