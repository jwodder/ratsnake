use super::levels::Bounds;
use ratatui::layout::Position;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    pub(super) fn advance(self, pos: Position, bounds: Bounds) -> Option<Position> {
        let Position { mut x, mut y } = pos;
        match self {
            Direction::North => {
                y = decrement_in_bounds(y, bounds.height, bounds.wrap)?;
            }
            Direction::East => {
                x = increment_in_bounds(x, bounds.width, bounds.wrap)?;
            }
            Direction::South => {
                y = increment_in_bounds(y, bounds.height, bounds.wrap)?;
            }
            Direction::West => {
                x = decrement_in_bounds(x, bounds.width, bounds.wrap)?;
            }
        }
        Some(Position { x, y })
    }

    pub(super) fn reverse(self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::East => Direction::West,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
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
    use ratatui::layout::Size;
    use rstest::rstest;

    #[rstest]
    #[case(
        Direction::North,
        Position::new(2, 7),
        Bounds::from((Size::new(10, 15), false)),
        Some(Position::new(2, 6))
    )]
    #[case(
        Direction::South,
        Position::new(2, 7),
        Bounds::from((Size::new(10, 15), false)),
        Some(Position::new(2, 8))
    )]
    #[case(
        Direction::East,
        Position::new(2, 7),
        Bounds::from((Size::new(10, 15), false)),
        Some(Position::new(3, 7))
    )]
    #[case(
        Direction::West,
        Position::new(2, 7),
        Bounds::from((Size::new(10, 15), false)),
        Some(Position::new(1, 7))
    )]
    #[case(
        Direction::North,
        Position::new(2, 0),
        Bounds::from((Size::new(10, 15), false)),
        None
    )]
    #[case(
        Direction::North,
        Position::new(2, 0),
        Bounds::from((Size::new(10, 15), true)),
        Some(Position::new(2, 14))
    )]
    #[case(
        Direction::South,
        Position::new(2, 14),
        Bounds::from((Size::new(10, 15), false)),
        None
    )]
    #[case(
        Direction::South,
        Position::new(2, 14),
        Bounds::from((Size::new(10, 15), true)),
        Some(Position::new(2, 0))
    )]
    #[case(
        Direction::East,
        Position::new(9, 7),
        Bounds::from((Size::new(10, 15), false)),
        None
    )]
    #[case(
        Direction::East,
        Position::new(9, 7),
        Bounds::from((Size::new(10, 15), true)),
        Some(Position::new(0, 7))
    )]
    #[case(
        Direction::West,
        Position::new(0, 7),
        Bounds::from((Size::new(10, 15), false)),
        None
    )]
    #[case(
        Direction::West,
        Position::new(0, 7),
        Bounds::from((Size::new(10, 15), true)),
        Some(Position::new(9, 7))
    )]
    fn test_direction_advance(
        #[case] d: Direction,
        #[case] pos: Position,
        #[case] bounds: Bounds,
        #[case] r: Option<Position>,
    ) {
        assert_eq!(d.advance(pos, bounds), r);
    }
}
