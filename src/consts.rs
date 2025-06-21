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

pub(crate) const LEVEL_SIZE: Size = Size {
    width: 76,
    height: 19,
};

/// Maximum snake length before any fruits have been eaten
pub(crate) const INITIAL_SNAKE_LENGTH: usize = 3;

/// How many cells the snake's length should increase by upon eating a fruit
pub(crate) const SNAKE_GROWTH: usize = 3;

pub(crate) const SNAKE_HEAD_SYMBOL: char = '@';

pub(crate) const SNAKE_BODY_SYMBOL: char = '~';

pub(crate) const FRUIT_SYMBOL: char = '\u{25CF}';

//pub(crate) const WALL_SYMBOL: char = '\u{2588}';

pub(crate) const COLLISION_SYMBOL: char = '*';

pub(crate) const SNAKE_STYLE: Style = Style::new().fg(Color::LightGreen);

pub(crate) const FRUIT_STYLE: Style = Style::new().fg(Color::LightRed);

//pub(crate) const WALL_STYLE: Style = Style::new().fg(Color::Yellow);

pub(crate) const COLLISION_STYLE: Style = Style::new()
    .fg(Color::LightRed)
    .add_modifier(Modifier::REVERSED);
