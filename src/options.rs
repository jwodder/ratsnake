use crate::consts;
use enum_dispatch::enum_dispatch;
use enum_map::Enum;
use ratatui::layout::Size;
use std::fmt;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct Options {
    pub(crate) wraparound: bool,
    pub(crate) obstacles: bool,
    pub(crate) fruits: FruitQty,
    pub(crate) level_size: LevelSize,
}

impl Options {
    pub(crate) fn get(&self, key: OptKey) -> OptValue {
        match key {
            OptKey::Wraparound => self.wraparound.into(),
            OptKey::Obstacles => self.obstacles.into(),
            OptKey::Fruits => self.fruits.into(),
            OptKey::LevelSize => self.level_size.into(),
        }
    }

    pub(crate) fn set(&mut self, key: OptKey, value: OptValue) {
        match key {
            OptKey::Wraparound => {
                self.wraparound = value
                    .try_into()
                    .expect("Options::set(Wraparound, value) called with non-Bool value");
            }
            OptKey::Obstacles => {
                self.obstacles = value
                    .try_into()
                    .expect("Options::set(Obstacles, value) called with non-Bool value");
            }
            OptKey::Fruits => {
                self.fruits = value
                    .try_into()
                    .expect("Options::set(Fruits, value) called with non-FruitQty value");
            }
            OptKey::LevelSize => {
                self.level_size = value
                    .try_into()
                    .expect("Options::set(LevelSize, value) called with non-LevelSize value");
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Enum, Eq, PartialEq)]
pub(crate) enum OptKey {
    Wraparound,
    Obstacles,
    Fruits,
    LevelSize,
}

impl OptKey {
    pub(crate) const DISPLAY_WIDTH: u16 = 10;

    pub(crate) fn iter() -> impl Iterator<Item = OptKey> {
        (0..Self::LENGTH).map(Self::from_usize)
    }

    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            OptKey::Wraparound => "Wraparound",
            OptKey::Obstacles => "Obstacles",
            OptKey::Fruits => "Fruits",
            OptKey::LevelSize => "Level Size",
        }
    }
}

impl fmt::Display for OptKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(self.as_str())
    }
}

#[enum_dispatch]
pub(crate) trait Adjustable {
    fn increase(&mut self);
    fn decrease(&mut self);
    fn toggle(&mut self);
    fn can_increase(&self) -> bool;
    fn can_decrease(&self) -> bool;
}

#[enum_dispatch(Adjustable)] // This also gives us From and TryInto
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OptValue {
    Bool(bool),
    FruitQty,
    LevelSize,
}

impl OptValue {
    pub(crate) const DISPLAY_WIDTH: u16 = 10;
}

// This is needed for EnumMap to be convenient to construct.
impl Default for OptValue {
    fn default() -> OptValue {
        OptValue::Bool(false)
    }
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

    mod opt_key {
        use super::*;

        #[test]
        fn display_width() {
            let actual_width = OptKey::iter()
                .map(|key| key.as_str().chars().count())
                .max()
                .unwrap();
            assert_eq!(actual_width, usize::from(OptKey::DISPLAY_WIDTH));
        }

        #[test]
        fn fmt_width() {
            assert_eq!(
                format!(
                    "{:width$}",
                    OptKey::Fruits,
                    width = usize::from(OptKey::DISPLAY_WIDTH)
                ),
                "Fruits    "
            );
        }
    }

    mod opt_value {
        use super::*;

        #[test]
        fn display_width() {
            let actual_width = [
                OptValue::Bool(false),
                OptValue::Bool(true),
                OptValue::FruitQty(FruitQty(1)),
                OptValue::FruitQty(FruitQty(consts::MAX_FRUITS)),
                OptValue::LevelSize(LevelSize::Small),
                OptValue::LevelSize(LevelSize::Medium),
                OptValue::LevelSize(LevelSize::Large),
            ]
            .iter()
            .map(|value| value.to_string().chars().count())
            .max()
            .unwrap();
            assert_eq!(actual_width, usize::from(OptValue::DISPLAY_WIDTH));
        }
    }

    mod level_size {
        use super::*;

        #[test]
        fn fmt_width() {
            assert_eq!(format!("{:6}", LevelSize::Small), "Small ");
        }
    }
}
