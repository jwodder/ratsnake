use crate::consts;
use enum_dispatch::enum_dispatch;
use ratatui::layout::Size;
use std::fmt;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct Options {
    pub(crate) wraparound: bool,
    pub(crate) obstacles: bool,
    pub(crate) fruits: FruitQty,
    pub(crate) level_size: LevelSize,
}

#[enum_dispatch]
pub(crate) trait Adjustable {
    fn increase(&mut self);
    fn decrease(&mut self);
    fn toggle(&mut self);
    fn can_increase(&self) -> bool;
    fn can_decrease(&self) -> bool;
}

#[enum_dispatch(Adjustable)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OptValue {
    Bool(bool),
    FruitQty,
    LevelSize,
}

impl fmt::Display for OptValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            OptValue::Bool(false) => write!(f, "   [ ]    "),
            OptValue::Bool(true) => write!(f, "   [✓]    "),
            OptValue::FruitQty(frq) => {
                write!(
                    f,
                    "{left} {frq:^6} {right}",
                    left = if frq.can_decrease() { '◀' } else { '◁' },
                    right = if frq.can_increase() { '▶' } else { '▷' }
                )
            }
            OptValue::LevelSize(sz) => {
                write!(
                    f,
                    "{left} {sz:6} {right}",
                    left = if sz.can_decrease() { '◀' } else { '◁' },
                    right = if sz.can_increase() { '▶' } else { '▷' }
                )
            }
        }
    }
}

impl Adjustable for bool {
    fn increase(&mut self) {
        *self = true;
    }

    fn decrease(&mut self) {
        *self = false;
    }

    fn toggle(&mut self) {
        *self = !*self;
    }

    fn can_increase(&self) -> bool {
        !*self
    }

    fn can_decrease(&self) -> bool {
        *self
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
}

impl fmt::Display for LevelSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            LevelSize::Small => "Small",
            LevelSize::Medium => "Medium",
            LevelSize::Large => "Large",
        };
        f.pad(name)
    }
}

impl Adjustable for LevelSize {
    fn increase(&mut self) {
        match self {
            LevelSize::Small => *self = LevelSize::Medium,
            LevelSize::Medium => *self = LevelSize::Large,
            LevelSize::Large => (),
        }
    }

    fn decrease(&mut self) {
        match self {
            LevelSize::Small => (),
            LevelSize::Medium => *self = LevelSize::Small,
            LevelSize::Large => *self = LevelSize::Medium,
        }
    }

    fn toggle(&mut self) {}

    fn can_increase(&self) -> bool {
        *self != Self::MAXIMUM
    }

    fn can_decrease(&self) -> bool {
        *self != Self::MINIMUM
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FruitQty(usize);

impl FruitQty {
    #[allow(unused)]
    pub(crate) fn new(qty: usize) -> Option<FruitQty> {
        (1..=consts::MAX_FRUITS)
            .contains(&qty)
            .then_some(FruitQty(qty))
    }

    pub(crate) fn get(self) -> usize {
        self.0
    }
}

impl Default for FruitQty {
    fn default() -> FruitQty {
        FruitQty(1)
    }
}

impl fmt::Display for FruitQty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(&self.0.to_string())
    }
}

impl Adjustable for FruitQty {
    fn increase(&mut self) {
        if self.can_increase() {
            self.0 += 1;
        }
    }

    fn decrease(&mut self) {
        if self.can_decrease() {
            self.0 -= 1;
        }
    }

    fn toggle(&mut self) {}

    fn can_increase(&self) -> bool {
        self.0 < consts::MAX_FRUITS
    }

    fn can_decrease(&self) -> bool {
        self.0 > 1
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
