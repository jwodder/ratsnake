use crate::consts;
use crate::util::{data_dir, Bounds, LoadError, SaveError};
use enum_dispatch::enum_dispatch;
use enum_map::Enum;
use ratatui::layout::Size;
use serde::{
    de::{Deserializer, Unexpected},
    ser::Serializer,
    Deserialize, Serialize,
};
use std::fmt;
use std::path::{Path, PathBuf};

/// Gameplay options
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub(crate) struct Options {
    /// Should levels wrap around at the borders?
    #[serde(default)]
    pub(crate) wraparound: bool,

    /// Should randomly-generated obstacles be placed in levels?
    #[serde(default)]
    pub(crate) obstacles: bool,

    /// Number of fruits present in a level at one time
    #[serde(default)]
    pub(crate) fruits: FruitQty,

    /// Size of levels
    #[serde(default)]
    pub(crate) level_size: LevelSize,
}

impl Options {
    /// Return the default filepath used for storing gameplay options
    pub(crate) fn default_path() -> Option<PathBuf> {
        data_dir().map(|p| p.join("options.json"))
    }

    /// Save the options to a file on disk
    ///
    /// # Errors
    ///
    /// Returns `Err` if creating the file's parent directories failed, if
    /// serializing the options failed, or if writing the serialized options
    /// failed.
    pub(crate) fn save(&self, path: &Path) -> Result<(), SaveError> {
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            fs_err::create_dir_all(parent).map_err(|e| SaveError::mkdir("options", e))?;
        }
        let mut src =
            serde_json::to_string(self).map_err(|e| SaveError::serialize("options", e))?;
        src.push('\n');
        fs_err::write(path, &src).map_err(|e| SaveError::write("options", e))?;
        Ok(())
    }

    /// Read options from a file on disk.  If the file does not exist and
    /// `allow_missing` is true, a default `Options` value is returned.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the file could not be read or if the file's contents
    /// could not be deserialized.
    pub(crate) fn load(path: &Path, allow_missing: bool) -> Result<Options, LoadError> {
        let src = match fs_err::read(path) {
            Ok(src) => src,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound && allow_missing => {
                return Ok(Options::default())
            }
            Err(e) => return Err(LoadError::read("options", e)),
        };
        serde_json::from_slice(&src).map_err(|e| LoadError::deserialize("options", e))
    }

    /// Retrieve the value of the given option as an [`OptValue`]
    pub(crate) fn get(&self, key: OptKey) -> OptValue {
        match key {
            OptKey::Wraparound => self.wraparound.into(),
            OptKey::Obstacles => self.obstacles.into(),
            OptKey::Fruits => self.fruits.into(),
            OptKey::LevelSize => self.level_size.into(),
        }
    }

    /// Set the value of the option `key` to `value`
    ///
    /// # Panics
    ///
    /// Panics if the `value` variant is not the one that the value of `key`
    /// requires
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

    /// Return level bounds as configured by the options
    pub(crate) fn level_bounds(&self) -> Bounds {
        Bounds::new(self.level_size.as_size(), self.wraparound)
    }
}

/// An enum of the individual option fields in [`Options`]
#[derive(Clone, Copy, Debug, Enum, Eq, PartialEq)]
pub(crate) enum OptKey {
    Wraparound,
    Obstacles,
    Fruits,
    LevelSize,
}

impl OptKey {
    /// The maximum display column width of `opt_key.to_string()` for `opt_key:
    /// OptKey`
    pub(crate) const DISPLAY_WIDTH: u16 = 10;

    /// Return a human-readable name for the option
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

/// A trait for modifiable option values
#[enum_dispatch]
pub(crate) trait Adjustable {
    /// Increase the value if possible
    fn increase(&mut self);

    /// Decrease the value if possible
    fn decrease(&mut self);

    /// Toggle the value on/off if applicable
    fn toggle(&mut self);

    /// Returns `true` if `increase()` would change `self`
    fn can_increase(&self) -> bool;

    /// Returns `true` if `decrease()` would change `self`
    fn can_decrease(&self) -> bool;
}

/// An enum of value types used  by [`Options`]
#[enum_dispatch(Adjustable)] // This also gives us From and TryInto
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OptValue {
    Bool(bool),
    FruitQty,
    LevelSize,
}

impl OptValue {
    /// The display column width of `opt_value.to_string()` for all `opt_value:
    /// OptValue`
    pub(crate) const DISPLAY_WIDTH: u16 = 10;
}

// This is needed for EnumMap to be convenient to construct.
impl Default for OptValue {
    fn default() -> OptValue {
        OptValue::Bool(false) // An arbitrary value
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

/// Possible level sizes that the user can choose from
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum LevelSize {
    /// A 38×8 level
    Small,

    /// A 53×12 level
    Medium,

    /// A 76×19 level
    #[default]
    Large,
}

impl LevelSize {
    pub(crate) const MINIMUM: LevelSize = LevelSize::Small;
    pub(crate) const MAXIMUM: LevelSize = LevelSize::Large;

    /// Return the actual size for the level size choice
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

/// The number of fruits present in a level at one time.  The value is
/// restricted to between 1 and [`crate::consts::MAX_FRUITS`], inclusive.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct FruitQty(usize);

impl FruitQty {
    /// Create a new `FruitQty` with the given value.  Returns `None` if `qty`
    /// is out of bounds.
    pub(crate) fn new(qty: usize) -> Option<FruitQty> {
        (1..=consts::MAX_FRUITS)
            .contains(&qty)
            .then_some(FruitQty(qty))
    }

    /// Return the value as a `usize`
    pub(crate) fn get(self) -> usize {
        self.0
    }
}

impl Default for FruitQty {
    /// One fruit
    fn default() -> FruitQty {
        FruitQty(1)
    }
}

impl fmt::Display for FruitQty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(&self.0.to_string())
    }
}

impl Serialize for FruitQty {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

macro_rules! try_visit_int {
    ($($t:ty, $visit:ident),* $(,)?) => {
        $(
            fn $visit<E>(self, value: $t) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                usize::try_from(value).ok().and_then(FruitQty::new)
                    .ok_or_else(|| E::invalid_value(Unexpected::Signed(value.into()), &self))
            }
        )*
    }
}

macro_rules! try_visit_uint {
    ($($t:ty, $visit:ident),* $(,)?) => {
        $(
            fn $visit<E>(self, value: $t) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                usize::try_from(value).ok().and_then(FruitQty::new)
                    .ok_or_else(|| E::invalid_value(Unexpected::Unsigned(value.into()), &self))
            }
        )*
    }
}

impl<'de> Deserialize<'de> for FruitQty {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;

        impl serde::de::Visitor<'_> for Visitor {
            type Value = FruitQty;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "a number from 1 to {}", consts::MAX_FRUITS)
            }

            try_visit_int!(i8, visit_i8, i16, visit_i16, i32, visit_i32, i64, visit_i64);
            try_visit_uint!(u8, visit_u8, u16, visit_u16, u32, visit_u32, u64, visit_u64);
        }

        deserializer.deserialize_any(Visitor)
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
        use crate::util::EnumExt;

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
            assert!(
                [
                    OptValue::Bool(false),
                    OptValue::Bool(true),
                    OptValue::FruitQty(FruitQty(1)),
                    OptValue::FruitQty(FruitQty(consts::MAX_FRUITS)),
                    OptValue::LevelSize(LevelSize::Small),
                    OptValue::LevelSize(LevelSize::Medium),
                    OptValue::LevelSize(LevelSize::Large),
                ]
                .iter()
                .all(|value| value.to_string().chars().count()
                    == usize::from(OptValue::DISPLAY_WIDTH))
            );
        }
    }

    mod fruit_qty {
        use super::*;
        use rstest::rstest;

        #[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
        struct FruitStruct {
            fruits: FruitQty,
        }

        #[rstest]
        #[case(1)]
        #[case(5)]
        #[case(10)]
        fn deserialize_good_json(#[case] qty: usize) {
            let src = format!(r#"{{"fruits": {qty}}}"#);
            let value = serde_json::from_str::<FruitStruct>(&src).unwrap();
            assert_eq!(value.fruits.get(), qty);
        }

        #[rstest]
        #[case(-1)]
        #[case(0)]
        #[case(15)]
        fn deserialize_bad_json(#[case] qty: isize) {
            let src = format!(r#"{{"fruits": {qty}}}"#);
            assert!(serde_json::from_str::<FruitStruct>(&src).is_err());
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
