//! Assorted constants & hard-coded configuration
use ratatui::{
    layout::Size,
    style::{Color, Modifier, Style},
};
use std::time::Duration;

/// Time between movements of the snake
pub(crate) const TICK_PERIOD: Duration = Duration::from_millis(200);

/// Draw everything inside a rectangle of this size in the center of the
/// terminal window.
///
/// Cf. [`crate::util::get_display_area()`]
pub(crate) const DISPLAY_SIZE: Size = Size {
    width: 80,
    height: 24,
};

/// The maximum number of fruits that can be present on a level at one time
pub(crate) const MAX_FRUITS: usize = 10;

/// Maximum snake length before any fruits have been eaten
pub(crate) const INITIAL_SNAKE_LENGTH: usize = 3;

/// How many cells the snake's length should increase by upon eating a fruit
pub(crate) const SNAKE_GROWTH: usize = 3;

/// Glyph for the snake's head when it is moving north/up
pub(crate) const SNAKE_HEAD_NORTH_SYMBOL: char = 'v';

/// Glyph for the snake's head when it is moving south/down
pub(crate) const SNAKE_HEAD_SOUTH_SYMBOL: char = '^';

/// Glyph for the snake's head when it is moving east/right
pub(crate) const SNAKE_HEAD_EAST_SYMBOL: char = '<';

/// Glyph for the snake's head when it is moving west/left
pub(crate) const SNAKE_HEAD_WEST_SYMBOL: char = '>';

/// Glyph for the parts of the snake's body
pub(crate) const SNAKE_BODY_SYMBOL: char = '⚬';

/// Glyph for the fruit
pub(crate) const FRUIT_SYMBOL: char = '●';

/// Glyph for obstacles
pub(crate) const OBSTACLE_SYMBOL: char = '█';

/// Glyph for the snake's head when it's collided with an obstacle or wall
pub(crate) const COLLISION_SYMBOL: char = '×';

/// Style for the snake's head and body
pub(crate) const SNAKE_STYLE: Style = Style::new().fg(Color::Green).add_modifier(Modifier::BOLD);

/// Style for the fruit
pub(crate) const FRUIT_STYLE: Style = Style::new().fg(Color::LightRed);

/// Style for obstacles
pub(crate) const OBSTACLE_STYLE: Style = Style::new().fg(Color::Gray);

/// Style for [`COLLISION_SYMBOL`]
pub(crate) const COLLISION_STYLE: Style = Style::new()
    .fg(Color::LightRed)
    .add_modifier(Modifier::REVERSED);

/// Style for key codes shown in the interface
pub(crate) const KEY_STYLE: Style = Style::new().fg(Color::Yellow);

/// Style for the score bar at the top of the game screen
pub(crate) const SCORE_BAR_STYLE: Style = Style::new().add_modifier(Modifier::REVERSED);

/// Style for the currently-selected menu item
pub(crate) const MENU_SELECTION_STYLE: Style = Style::new().add_modifier(Modifier::UNDERLINED);

/// Probability of placing an obstacle in a given cell
pub(crate) const OBSTACLE_PROBABILITY: f64 = 0.03;

/// When creating a level with random obstacles, remove any obstacles behind
/// the snake's head this many cells backwards.
pub(crate) const BACKWARDS_CLEARANCE: usize = 3;

/// When creating a level with random obstacles, remove any obstacles in front
/// of the snake's head this many cells forwards.
pub(crate) const FORWARDS_CLEARANCE: usize = 7;
