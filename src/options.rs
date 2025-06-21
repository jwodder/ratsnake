use ratatui::layout::Size;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Options {
    pub(crate) wraparound: bool,
    pub(crate) obstacles: bool,
    pub(crate) fruits: usize,
    pub(crate) level_size: LevelSize,
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
    pub(crate) const MINIMUM: LevelSize = LevelSize::Small;
    pub(crate) const MAXIMUM: LevelSize = LevelSize::Large;

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

    pub(crate) fn increase(self) -> Option<LevelSize> {
        match self {
            LevelSize::Small => Some(LevelSize::Medium),
            LevelSize::Medium => Some(LevelSize::Large),
            LevelSize::Large => None,
        }
    }

    pub(crate) fn decrease(self) -> Option<LevelSize> {
        match self {
            LevelSize::Small => None,
            LevelSize::Medium => Some(LevelSize::Small),
            LevelSize::Large => Some(LevelSize::Medium),
        }
    }
}

impl fmt::Display for LevelSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            LevelSize::Small => "Small",
            LevelSize::Medium => "Medium",
            LevelSize::Large => "Large",
        };
        write!(f, "{name}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod level_size {
        use super::*;

        #[test]
        fn display_width() {
            assert_eq!(format!("{:6}", LevelSize::Small), "Small ");
        }
    }
}
