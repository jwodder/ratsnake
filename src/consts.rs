use ratatui::{
    layout::Size,
    style::{Color, Modifier, Style},
};
use std::time::Duration;

pub(crate) const TICK_DURATION: Duration = Duration::from_millis(200);

pub(crate) const DISPLAY_SIZE: Size = Size {
    width: 80,
    height: 24,
};

pub(crate) const MAX_FRUITS: usize = 10;

/// Maximum snake length before any fruits have been eaten
pub(crate) const INITIAL_SNAKE_LENGTH: usize = 3;

/// How many cells the snake's length should increase by upon eating a fruit
pub(crate) const SNAKE_GROWTH: usize = 3;

pub(crate) const SNAKE_HEAD_NORTH_SYMBOL: char = 'v';
pub(crate) const SNAKE_HEAD_SOUTH_SYMBOL: char = '^';
pub(crate) const SNAKE_HEAD_EAST_SYMBOL: char = '<';
pub(crate) const SNAKE_HEAD_WEST_SYMBOL: char = '>';

pub(crate) const SNAKE_BODY_SYMBOL: char = '⚬';

pub(crate) const FRUIT_SYMBOL: char = '●';

pub(crate) const OBSTACLE_SYMBOL: char = '█';

pub(crate) const COLLISION_SYMBOL: char = '×';

pub(crate) const SNAKE_STYLE: Style = Style::new().fg(Color::Green).add_modifier(Modifier::BOLD);

pub(crate) const FRUIT_STYLE: Style = Style::new().fg(Color::LightRed);

pub(crate) const OBSTACLE_STYLE: Style = Style::new().fg(Color::Gray);

pub(crate) const COLLISION_STYLE: Style = Style::new()
    .fg(Color::LightRed)
    .add_modifier(Modifier::REVERSED);

pub(crate) const KEY_STYLE: Style = Style::new().fg(Color::Yellow);

pub(crate) const SCORE_BAR_STYLE: Style = Style::new().add_modifier(Modifier::REVERSED);

pub(crate) const MENU_SELECTION_STYLE: Style = Style::new().add_modifier(Modifier::UNDERLINED);

pub(crate) const OBSTACLE_PROBABILITY: f64 = 0.03;

/// When creating a level with random obstacles, remove any obstacles behind
/// the snake's head this many cells backwards.
pub(crate) const BACKWARDS_CLEARANCE: usize = 3;

/// When creating a level with random obstacles, remove any obstacles in front
/// of the snake's head this many cells forwards.
pub(crate) const FORWARDS_CLEARANCE: usize = 7;
