use ratatui::layout::Size;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Options {
    wraparound: bool,
    obstacles: bool,
    fruits: usize,
    level_size: LevelSize,
}

impl Default for Options {
    fn default() -> Options {
        Options {
            wraparound: false,
            obstacles: false,
            fruits: 1,
            level_size: LevelSize::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum LevelSize {
    Small,
    Medium,
    #[default]
    Large,
}

impl LevelSize {
    pub(crate) fn as_size(self) -> Size {
        match self {
            LevelSize::Small => Size {
                width: 38,
                height: 8,
            },
            LevelSize::Medium => Size {
                width: 53,
                height: 12,
            },
            LevelSize::Large => Size {
                width: 76,
                height: 19,
            },
        }
    }
}
