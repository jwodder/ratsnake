use super::direction::Direction;
use crate::consts;
use rand::{
    distr::{Bernoulli, Distribution},
    Rng,
};
use ratatui::layout::{Position, Positions, Rect, Size};
use std::collections::HashSet;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct LevelMap {
    bounds: Bounds,
    obstacles: HashSet<Position>,
    snake_start: (Position, Direction),
}

impl LevelMap {
    pub(super) fn new(bounds: Bounds) -> LevelMap {
        let snake_head = Position::new(bounds.width / 2, bounds.height / 2);
        LevelMap {
            bounds,
            obstacles: HashSet::new(),
            snake_start: (snake_head, Direction::North),
        }
    }

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

    pub(super) fn bounds(&self) -> Bounds {
        self.bounds
    }

    pub(super) fn obstacles(&self) -> &HashSet<Position> {
        &self.obstacles
    }

    pub(super) fn snake_start(&self) -> (Position, Direction) {
        self.snake_start
    }

    pub(super) fn size(&self) -> Size {
        self.bounds.size()
    }

    pub(super) fn wrap(&self) -> bool {
        self.bounds.wrap
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Bounds {
    pub(super) width: u16,
    pub(super) height: u16,
    pub(super) wrap: bool,
}

impl Bounds {
    pub(super) fn size(self) -> Size {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    pub(super) fn positions(self) -> Positions {
        Rect::from((Position::ORIGIN, self.size())).positions()
    }
}

impl From<(Size, bool)> for Bounds {
    fn from((size, wrap): (Size, bool)) -> Bounds {
        Bounds {
            width: size.width,
            height: size.height,
            wrap,
        }
    }
}
