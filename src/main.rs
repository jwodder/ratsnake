mod app;
mod command;
mod config;
mod consts;
mod game;
mod highscores;
mod menu;
mod options;
mod util;
mod warning;
use crate::app::App;
use crate::config::Config;
use crate::util::Globals;
use anyhow::Context;
use crossterm::{
    event::{DisableFocusChange, EnableFocusChange},
    execute,
};
use lexopt::{Arg, Parser, ValueExt};
use std::io::{self, ErrorKind, Write};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Clone, Debug, Eq, PartialEq)]
enum Command {
    Run(ConfigSource),
    Version,
}

impl Command {
    fn from_parser(mut parser: Parser) -> Result<Command, lexopt::Error> {
        let mut cfg_src = ConfigSource::DefaultPath;
        while let Some(arg) = parser.next()? {
            match arg {
                Arg::Short('V') | Arg::Long("version") => return Ok(Command::Version),
                Arg::Short('c') | Arg::Long("config") => {
                    cfg_src = ConfigSource::Path(parser.value()?.parse()?);
                }
                _ => return Err(arg.unexpected()),
            }
        }
        Ok(Command::Run(cfg_src))
    }

    fn run(self) -> anyhow::Result<()> {
        match self {
            Command::Run(cfg_src) => {
                let config = cfg_src.load()?;
                let options = options::Options::load()?;
                let high_scores = highscores::HighScores::load()?;
                let terminal = init_terminal()?;
                let r = App::new(Globals {
                    config,
                    options,
                    high_scores,
                })
                .run(terminal)
                .map_err(anyhow::Error::from);
                match restore_terminal() {
                    Ok(()) => r,
                    Err(e) if r.is_ok() => Err(e),
                    Err(e) => {
                        errmsg(e);
                        r
                    }
                }
            }
            Command::Version => {
                writeln!(
                    io::stdout().lock(),
                    "{} {}",
                    env!("CARGO_PKG_NAME"),
                    env!("CARGO_PKG_VERSION")
                )?;
                Ok(())
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ConfigSource {
    DefaultPath,
    Path(PathBuf),
}

impl ConfigSource {
    fn load(&self) -> anyhow::Result<Config> {
        match self {
            ConfigSource::DefaultPath => match Config::load(&Config::default_path()?) {
                Ok(config) => Ok(config),
                Err(e) if e.is_not_found() => Ok(Config::default()),
                Err(e) => Err(e.into()),
            },
            ConfigSource::Path(p) => Config::load(p).map_err(Into::into),
        }
    }
}

fn main() -> ExitCode {
    match Command::from_parser(Parser::from_env())
        .map_err(anyhow::Error::from)
        .and_then(Command::run)
    {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            for cause in e.chain() {
                if let Some(ioerr) = cause.downcast_ref::<io::Error>() {
                    if ioerr.kind() == ErrorKind::BrokenPipe {
                        return ExitCode::SUCCESS;
                    }
                }
            }
            errmsg(e);
            ExitCode::FAILURE
        }
    }
}

/// Initialize the terminal, including enabling focus events
fn init_terminal() -> anyhow::Result<ratatui::DefaultTerminal> {
    let terminal = ratatui::try_init().context("failed to set up terminal")?;
    match execute!(io::stdout(), EnableFocusChange) {
        Ok(()) => Ok(terminal),
        Err(e) => {
            ratatui::restore();
            Err(e).context("failed to set up terminal")
        }
    }
}

/// Clean up the terminal, undoing the changes made by [`init_terminal()`]
fn restore_terminal() -> anyhow::Result<()> {
    execute!(io::stdout(), DisableFocusChange)
        .and(ratatui::try_restore())
        .context("failed to clean up terminal")
}

/// Report an error to stderr, discarding any errors that occur in the process.
fn errmsg(e: anyhow::Error) {
    let _ = writeln!(io::stderr().lock(), "ratsnake: {e:?}");
}
