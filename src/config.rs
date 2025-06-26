use crate::consts;
use crate::direction::Direction;
use crate::highscores::HighScores;
use crate::options::Options;
use crate::util::{expanduser, LoadError, NoHomeError, SaveError};
use ratatui::style::Style;
use serde::{
    de::{Deserializer, Unexpected},
    Deserialize,
};
use std::borrow::Cow;
use std::fmt;
use std::path::{Path, PathBuf};
use thiserror::Error;
use unicode_properties::{GeneralCategoryGroup, UnicodeGeneralCategory};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Program configuration read from a configuration file
#[derive(Clone, Deserialize, Debug, Default, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Config {
    /// Default options when no options file is present
    #[serde(default)]
    pub(crate) options: Options,

    /// Settings about data files
    #[serde(default)]
    pub(crate) files: FileConfig,

    /// Game glyph settings
    #[serde(default)]
    pub(crate) glyphs: GlyphConfig,
}

impl Config {
    /// Return the default configuration file path
    pub(crate) fn default_path() -> Result<PathBuf, ConfigError> {
        dirs::config_local_dir()
            .map(|p| p.join("ratsnake").join("config.toml"))
            .ok_or(ConfigError::NoPath)
    }

    /// Read configuration from a file on disk.  If the file does not exist and
    /// `allow_missing` is true, a default `Config` value is returned.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the file could not be read or if the file's contents
    /// could not be deserialized.
    pub(crate) fn load(path: &Path, allow_missing: bool) -> Result<Config, ConfigError> {
        let content = match fs_err::read_to_string(path) {
            Ok(content) => content,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound && allow_missing => {
                return Ok(Config::default())
            }
            Err(e) => return Err(ConfigError::Read(e)),
        };
        toml::from_str(&content).map_err(Into::into)
    }

    /// Return the filepath at which gameplay options should be stored: the
    /// file given in the configuration or, if that is not set, the default
    /// options file path.  Return `None` if saving & loading of gameplay
    /// options is disabled.
    fn options_file(&self) -> Result<Option<Cow<'_, Path>>, NoHomeError> {
        match self.files.options_file {
            OptionsFile::Path(ref path) => Ok(Some(Cow::from(path))),
            OptionsFile::Default => match Options::default_path() {
                Some(path) => Ok(Some(Cow::from(path))),
                None => Err(NoHomeError),
            },
            OptionsFile::Off => Ok(None),
        }
    }

    /// Load gameplay options from a file, if enabled.  If the file does not
    /// exist, `self.options` is returned.
    pub(crate) fn load_options(&self) -> Result<Options, LoadError> {
        let r = match self.options_file() {
            Ok(Some(p)) => Options::load(&p),
            Ok(None) => Ok(None),
            Err(_) if self.files.ignore_errors => Ok(None),
            Err(_) => Err(LoadError::no_path("options")),
        };
        match r {
            Ok(Some(opts)) => Ok(opts),
            Ok(None) => Ok(self.options),
            Err(_) if self.files.ignore_errors => Ok(self.options),
            Err(e) => Err(e),
        }
    }

    /// Save the given gameplay options to a file, if enabled.
    pub(crate) fn save_options(&self, options: Options) -> Result<(), SaveError> {
        match self.options_file() {
            Ok(Some(p)) => {
                let r = options.save(&p);
                if r.is_err() && self.files.ignore_errors {
                    Ok(())
                } else {
                    r
                }
            }
            Ok(None) => Ok(()),
            Err(_) if self.files.ignore_errors => Ok(()),
            Err(_) => Err(SaveError::no_path("options")),
        }
    }

    /// Return the filepath at which high scores should be stored: a file
    /// inside the directory file given in the configuration or, if that is not
    /// set, the default high scores file path.
    fn high_scores_file(&self) -> Result<PathBuf, NoHomeError> {
        match self.files.high_scores_dir {
            Some(ref path) => Ok(path.join(HighScores::ARCADE_FILE_NAME)),
            None => HighScores::default_path().ok_or(NoHomeError),
        }
    }

    /// Load high scores from a file.  If the file does not exist, an empty
    /// `HighScores` value is returned.
    pub(crate) fn load_high_scores(&self) -> Result<HighScores, LoadError> {
        match self.high_scores_file() {
            Ok(p) => {
                let r = HighScores::load(&p);
                if r.is_err() && self.files.ignore_errors {
                    Ok(HighScores::default())
                } else {
                    r
                }
            }
            Err(_) if self.files.ignore_errors => Ok(HighScores::default()),
            Err(_) => Err(LoadError::no_path("high scores")),
        }
    }

    /// Save the given high scores to a file
    pub(crate) fn save_high_scores(&self, scores: &HighScores) -> Result<(), SaveError> {
        match self.high_scores_file() {
            Ok(p) => {
                let r = scores.save(&p);
                if r.is_err() && self.files.ignore_errors {
                    Ok(())
                } else {
                    r
                }
            }
            Err(_) if self.files.ignore_errors => Ok(()),
            Err(_) => Err(SaveError::no_path("high scores")),
        }
    }
}

#[derive(Clone, Deserialize, Debug, Default, Eq, PartialEq)]
#[serde(try_from = "RawFileConfig")]
pub(crate) struct FileConfig {
    /// Path at which gameplay options should be stored
    options_file: OptionsFile<PathBuf>,

    /// Path to the directory in which high scores should be saved
    // This is a directory path in anticipation of eventually also storing
    // level high scores here.
    high_scores_dir: Option<PathBuf>,

    /// Whether to ignore errors that occur while saving & loading options &
    /// high-score files.
    ignore_errors: bool,
}

#[derive(Clone, Deserialize, Debug, Default, Eq, PartialEq)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
struct RawFileConfig {
    options_file: OptionsFile<String>,
    high_scores_dir: Option<String>,
    ignore_errors: bool,
}

impl TryFrom<RawFileConfig> for FileConfig {
    type Error = NoHomeError;

    fn try_from(value: RawFileConfig) -> Result<FileConfig, NoHomeError> {
        Ok(FileConfig {
            options_file: value.options_file.expanduser()?,
            high_scores_dir: value
                .high_scores_dir
                .as_deref()
                .map(expanduser)
                .transpose()?,
            ignore_errors: value.ignore_errors,
        })
    }
}

/// Possible settings for the `files.options-file` configuration
#[derive(Clone, Debug, Default, Eq, PartialEq)]
enum OptionsFile<T> {
    /// Save & load gameplay options to/from the given path
    Path(T),

    /// Save & load gameplay options to/from the default path
    #[default]
    Default,

    /// Do not save or load gameplay options to/from disk
    Off,
}

impl OptionsFile<String> {
    fn expanduser(self) -> Result<OptionsFile<PathBuf>, NoHomeError> {
        match self {
            OptionsFile::Path(p) => Ok(OptionsFile::Path(expanduser(&p)?)),
            OptionsFile::Default => Ok(OptionsFile::Default),
            OptionsFile::Off => Ok(OptionsFile::Off),
        }
    }
}

impl<'de> Deserialize<'de> for OptionsFile<String> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;

        impl serde::de::Visitor<'_> for Visitor {
            type Value = OptionsFile<String>;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a string or boolean")
            }

            fn visit_bool<E>(self, input: bool) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if input {
                    Ok(OptionsFile::Default)
                } else {
                    Ok(OptionsFile::Off)
                }
            }

            fn visit_str<E>(self, input: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(OptionsFile::Path(input.to_owned()))
            }

            fn visit_string<E>(self, input: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(OptionsFile::Path(input))
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(default, rename_all = "kebab-case")]
pub(crate) struct GlyphConfig {
    pub(crate) snake_head: SnakeHeadConfig,
    pub(crate) snake_body: Glyph,
    pub(crate) fruit: Glyph,
    pub(crate) obstacle: Glyph,
    pub(crate) collision: Glyph,
}

impl Default for GlyphConfig {
    fn default() -> GlyphConfig {
        GlyphConfig {
            snake_head: SnakeHeadConfig::default(),
            snake_body: Glyph {
                symbol: Symbol::try_from(consts::SNAKE_BODY_SYMBOL)
                    .expect("SNAKE_BODY_SYMBOL should be a valid Symbol"),
                style: consts::SNAKE_STYLE,
            },
            fruit: Glyph {
                symbol: Symbol::try_from(consts::FRUIT_SYMBOL)
                    .expect("FRUIT_SYMBOL should be a valid Symbol"),
                style: consts::FRUIT_STYLE,
            },
            obstacle: Glyph {
                symbol: Symbol::try_from(consts::OBSTACLE_SYMBOL)
                    .expect("OBSTACLE_SYMBOL should be a valid Symbol"),
                style: consts::OBSTACLE_STYLE,
            },
            collision: Glyph {
                symbol: Symbol::try_from(consts::COLLISION_SYMBOL)
                    .expect("COLLISION_SYMBOL should be a valid Symbol"),
                style: consts::COLLISION_STYLE,
            },
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(crate) struct Glyph {
    pub(crate) symbol: Symbol,

    #[serde(default, with = "parse_style::serde::ratatui::style")]
    pub(crate) style: Style,
}

/// A single non-control grapheme that occupies exactly one display column
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Symbol(String);

impl AsRef<str> for Symbol {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl TryFrom<char> for Symbol {
    type Error = ParseSymbolError;

    fn try_from(ch: char) -> Result<Symbol, ParseSymbolError> {
        if ch.width() != Some(1) {
            return Err(ParseSymbolError::Wide);
        }
        if ch.general_category_group() == GeneralCategoryGroup::Other {
            return Err(ParseSymbolError::Control);
        }
        Ok(Symbol(String::from(ch)))
    }
}

impl std::str::FromStr for Symbol {
    type Err = ParseSymbolError;

    fn from_str(s: &str) -> Result<Symbol, ParseSymbolError> {
        if s.graphemes(true).count() != 1 {
            return Err(ParseSymbolError::Graphemes);
        }
        if s.width() != 1 {
            return Err(ParseSymbolError::Wide);
        }
        if s.chars()
            .any(|c| c.general_category_group() == GeneralCategoryGroup::Other)
        {
            return Err(ParseSymbolError::Control);
        }
        Ok(Symbol(s.to_owned()))
    }
}

impl<'de> Deserialize<'de> for Symbol {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;

        impl serde::de::Visitor<'_> for Visitor {
            type Value = Symbol;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a single-grapheme string exactly one display column wide")
            }

            fn visit_str<E>(self, input: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                input
                    .parse::<Symbol>()
                    .map_err(|_| E::invalid_value(Unexpected::Str(input), &self))
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(crate) struct SnakeHeadConfig {
    pub(crate) symbol: SnakeHeadSymbol,

    #[serde(default, with = "parse_style::serde::ratatui::style")]
    pub(crate) style: Style,
}

impl Default for SnakeHeadConfig {
    fn default() -> SnakeHeadConfig {
        SnakeHeadConfig {
            symbol: SnakeHeadSymbol::Split {
                north: Symbol::try_from(consts::SNAKE_HEAD_NORTH_SYMBOL)
                    .expect("SNAKE_HEAD_NORTH_SYMBOL should be a valid Symbol"),
                south: Symbol::try_from(consts::SNAKE_HEAD_SOUTH_SYMBOL)
                    .expect("SNAKE_HEAD_SOUTH_SYMBOL should be a valid Symbol"),
                east: Symbol::try_from(consts::SNAKE_HEAD_EAST_SYMBOL)
                    .expect("SNAKE_HEAD_EAST_SYMBOL should be a valid Symbol"),
                west: Symbol::try_from(consts::SNAKE_HEAD_WEST_SYMBOL)
                    .expect("SNAKE_HEAD_WEST_SYMBOL should be a valid Symbol"),
            },
            style: consts::SNAKE_STYLE,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(untagged)]
pub(crate) enum SnakeHeadSymbol {
    All(Symbol),
    Split {
        north: Symbol,
        south: Symbol,
        east: Symbol,
        west: Symbol,
    },
}

impl SnakeHeadSymbol {
    pub(crate) fn for_direction(&self, dir: Direction) -> &Symbol {
        match (self, dir) {
            (SnakeHeadSymbol::All(sym), _) => sym,
            (SnakeHeadSymbol::Split { north, .. }, Direction::North) => north,
            (SnakeHeadSymbol::Split { south, .. }, Direction::South) => south,
            (SnakeHeadSymbol::Split { east, .. }, Direction::East) => east,
            (SnakeHeadSymbol::Split { west, .. }, Direction::West) => west,
        }
    }
}

#[derive(Debug, Error)]
pub(crate) enum ConfigError {
    #[error("failed to determine path to local configuration directory")]
    NoPath,
    #[error("failed to read configuration file")]
    Read(#[from] std::io::Error),
    #[error("failed to parse configuration file")]
    Parse(#[from] toml::de::Error),
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub(crate) enum ParseSymbolError {
    #[error("input contained control character")]
    Control,
    #[error("input does not render as a single display column")]
    Wide,
    #[error("input is not a single grapheme")]
    Graphemes,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    mod deser {
        use super::*;
        use ratatui::style::{Color, Modifier};
        use tempfile::NamedTempFile;

        #[test]
        fn options_file_path() {
            let tmp = NamedTempFile::new().unwrap();
            std::fs::write(
                tmp.path(),
                concat!(
                    "[files]\n",
                    "options-file = \"/home/luser/stuff/ratsnake/options.json\"\n"
                ),
            )
            .unwrap();
            let cfg = Config::load(tmp.path(), false).unwrap();
            assert_eq!(
                cfg,
                Config {
                    files: FileConfig {
                        options_file: OptionsFile::Path(PathBuf::from(
                            "/home/luser/stuff/ratsnake/options.json"
                        )),
                        ..FileConfig::default()
                    },
                    ..Config::default()
                }
            );
            assert_eq!(
                cfg.options_file(),
                Ok(Some(Cow::from(PathBuf::from(
                    "/home/luser/stuff/ratsnake/options.json"
                ))))
            );
        }

        #[test]
        fn options_file_missing() {
            let tmp = NamedTempFile::new().unwrap();
            std::fs::write(tmp.path(), "[files]\n").unwrap();
            let cfg = Config::load(tmp.path(), false).unwrap();
            assert_eq!(
                cfg,
                Config {
                    files: FileConfig {
                        options_file: OptionsFile::Default,
                        ..FileConfig::default()
                    },
                    ..Config::default()
                }
            );
            assert!(cfg.options_file().unwrap().is_some());
        }

        #[test]
        fn options_file_true() {
            let tmp = NamedTempFile::new().unwrap();
            std::fs::write(tmp.path(), "[files]\noptions-file = true\n").unwrap();
            let cfg = Config::load(tmp.path(), false).unwrap();
            assert_eq!(
                cfg,
                Config {
                    files: FileConfig {
                        options_file: OptionsFile::Default,
                        ..FileConfig::default()
                    },
                    ..Config::default()
                }
            );
            assert!(cfg.options_file().unwrap().is_some());
        }

        #[test]
        fn options_file_false() {
            let tmp = NamedTempFile::new().unwrap();
            std::fs::write(tmp.path(), "[files]\noptions-file = false\n").unwrap();
            let cfg = Config::load(tmp.path(), false).unwrap();
            assert_eq!(
                cfg,
                Config {
                    files: FileConfig {
                        options_file: OptionsFile::Off,
                        ..FileConfig::default()
                    },
                    ..Config::default()
                }
            );
            assert_eq!(cfg.options_file(), Ok(None));
        }

        #[test]
        fn snake_head_str() {
            let tmp = NamedTempFile::new().unwrap();
            std::fs::write(tmp.path(), "[glyphs]\nsnake-head.symbol = \"@\"\n").unwrap();
            let cfg = Config::load(tmp.path(), false).unwrap();
            assert_eq!(
                cfg,
                Config {
                    glyphs: GlyphConfig {
                        snake_head: SnakeHeadConfig {
                            symbol: SnakeHeadSymbol::All("@".parse::<Symbol>().unwrap()),
                            style: Style::new(),
                        },
                        ..GlyphConfig::default()
                    },
                    ..Config::default()
                }
            );
        }

        #[test]
        fn snake_head_styled_str() {
            let tmp = NamedTempFile::new().unwrap();
            std::fs::write(
                tmp.path(),
                "[glyphs]\nsnake-head = { symbol = \"@\", style = \"red\" }\n",
            )
            .unwrap();
            let cfg = Config::load(tmp.path(), false).unwrap();
            assert_eq!(
                cfg,
                Config {
                    glyphs: GlyphConfig {
                        snake_head: SnakeHeadConfig {
                            symbol: SnakeHeadSymbol::All("@".parse::<Symbol>().unwrap()),
                            style: Style::new().fg(Color::Indexed(1)),
                        },
                        ..GlyphConfig::default()
                    },
                    ..Config::default()
                }
            );
        }

        #[test]
        fn snake_head_directions() {
            let tmp = NamedTempFile::new().unwrap();
            std::fs::write(
                tmp.path(),
                concat!(
                    "[glyphs.snake-head]\n",
                    "symbol = { north = \"↑\", south = \"↓\", east = \"→\", west = \"←\" }\n",
                ),
            )
            .unwrap();
            let cfg = Config::load(tmp.path(), false).unwrap();
            assert_eq!(
                cfg,
                Config {
                    glyphs: GlyphConfig {
                        snake_head: SnakeHeadConfig {
                            symbol: SnakeHeadSymbol::Split {
                                north: "↑".parse::<Symbol>().unwrap(),
                                south: "↓".parse::<Symbol>().unwrap(),
                                east: "→".parse::<Symbol>().unwrap(),
                                west: "←".parse::<Symbol>().unwrap(),
                            },
                            style: Style::new(),
                        },
                        ..GlyphConfig::default()
                    },
                    ..Config::default()
                }
            );
        }

        #[test]
        fn snake_head_styled_directions() {
            let tmp = NamedTempFile::new().unwrap();
            std::fs::write(
                tmp.path(),
                concat!(
                    "[glyphs.snake-head]\n",
                    "symbol = { north = \"↑\", south = \"↓\", east = \"→\", west = \"←\" }\n",
                    "style = \"bold blue on white\"\n",
                ),
            )
            .unwrap();
            let cfg = Config::load(tmp.path(), false).unwrap();
            assert_eq!(
                cfg,
                Config {
                    glyphs: GlyphConfig {
                        snake_head: SnakeHeadConfig {
                            symbol: SnakeHeadSymbol::Split {
                                north: "↑".parse::<Symbol>().unwrap(),
                                south: "↓".parse::<Symbol>().unwrap(),
                                east: "→".parse::<Symbol>().unwrap(),
                                west: "←".parse::<Symbol>().unwrap(),
                            },
                            style: Style::new()
                                .fg(Color::Indexed(4))
                                .bg(Color::Indexed(7))
                                .add_modifier(Modifier::BOLD),
                        },
                        ..GlyphConfig::default()
                    },
                    ..Config::default()
                }
            );
        }

        #[test]
        fn fruit() {
            let tmp = NamedTempFile::new().unwrap();
            std::fs::write(tmp.path(), "[glyphs.fruit]\nsymbol = \"ó\"\n").unwrap();
            let cfg = Config::load(tmp.path(), false).unwrap();
            assert_eq!(
                cfg,
                Config {
                    glyphs: GlyphConfig {
                        fruit: Glyph {
                            symbol: "ó".parse::<Symbol>().unwrap(),
                            style: Style::new(),
                        },
                        ..GlyphConfig::default()
                    },
                    ..Config::default()
                }
            );
        }

        #[test]
        fn styled_fruit() {
            let tmp = NamedTempFile::new().unwrap();
            std::fs::write(
                tmp.path(),
                "[glyphs.fruit]\nsymbol = \"ó\"\nstyle = \"bold red1\"",
            )
            .unwrap();
            let cfg = Config::load(tmp.path(), false).unwrap();
            assert_eq!(
                cfg,
                Config {
                    glyphs: GlyphConfig {
                        fruit: Glyph {
                            symbol: "ó".parse::<Symbol>().unwrap(),
                            style: Style::new()
                                .fg(Color::Indexed(196))
                                .add_modifier(Modifier::BOLD),
                        },
                        ..GlyphConfig::default()
                    },
                    ..Config::default()
                }
            );
        }
    }

    #[rstest]
    #[case("", false)]
    #[case(" ", true)]
    #[case("A", true)]
    #[case("Á", true)]
    #[case("A\u{0301}", true)]
    #[case("A\u{00B4}", false)] // multiple graphemes
    #[case("\u{0301}", false)] // modifier without base
    #[case("\u{02CA}", true)]
    #[case("\0", false)] // control
    #[case("\t", false)] // control
    #[case("\n", false)] // control
    #[case("\r\n", false)] // control
    #[case("\x1B", false)] // control
    #[case("\x7F", false)] // control
    #[case("\u{0080}", false)] // control
    #[case("\u{FF10}", false)] // wide
    #[case("\u{1F601}", false)] // wide
    #[case("\u{200D}", false)] // zero-width
    #[case("\u{F8FF}", false)] // private use
    #[case("\u{FFFF}", false)] // unassigned
    #[case("\u{AD}", false)] // soft hyphen (formatting character)
    #[case("\u{200E}", false)] // left-to-right mark (formatting character)
    #[case("\u{200B}A", false)] // zero width char + another grapheme
    #[case("\u{231A}\u{FE0E}", true)] // text presentation sequence (Problem: WezTerm says this is two columns, but unicode-width says 1)
    #[ignore]
    #[case("0\u{fe0f}\u{20e3}", true)] // emoji keycap sequence (WezTerm says this is one column, unicode-width says two; who's right?)
    fn test_accept_symbol_str(#[case] s: &str, #[case] ok: bool) {
        assert_eq!(s.parse::<Symbol>().is_ok(), ok);
    }

    #[rstest]
    #[case(' ', true)]
    #[case('A', true)]
    #[case('Á', true)]
    #[case('\u{0301}', false)] // modifier without base
    #[case('\u{02CA}', true)]
    #[case('\0', false)] // control
    #[case('\t', false)] // control
    #[case('\n', false)] // control
    #[case('\x1B', false)] // control
    #[case('\x7F', false)] // control
    #[case('\u{0080}', false)] // control
    #[case('\u{FF10}', false)] // wide
    #[case('\u{1F601}', false)] // wide
    #[case('\u{200D}', false)] // zero-width
    #[case('\u{F8FF}', false)] // private use
    #[case('\u{FFFF}', false)] // unassigned
    #[case('\u{AD}', false)] // soft hyphen (formatting character)
    #[case('\u{200E}', false)] // left-to-right mark (formatting character)
    fn test_accept_symbol_char(#[case] ch: char, #[case] ok: bool) {
        assert_eq!(Symbol::try_from(ch).is_ok(), ok);
    }

    #[test]
    fn test_default_glyph_config() {
        GlyphConfig::default();
    }
}
