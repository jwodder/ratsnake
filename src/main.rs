mod app;
mod command;
mod consts;
mod game;
mod highscores;
mod menu;
mod options;
mod util;
mod warning;
use crate::app::App;
use crate::util::Globals;
use crossterm::{
    event::{DisableFocusChange, EnableFocusChange},
    execute,
};
use std::io::{self, ErrorKind, Write};
use std::process::ExitCode;
use thiserror::Error;

fn main() -> ExitCode {
    let options = match options::Options::load() {
        Ok(opts) => opts,
        Err(e) => {
            let _ = writeln!(io::stderr().lock(), "ratsnake: {:?}", anyhow::Error::new(e));
            return ExitCode::FAILURE;
        }
    };
    let high_scores = match highscores::HighScores::load() {
        Ok(hs) => hs,
        Err(e) => {
            let _ = writeln!(io::stderr().lock(), "ratsnake: {:?}", anyhow::Error::new(e));
            return ExitCode::FAILURE;
        }
    };
    let terminal = match init_terminal() {
        Ok(term) => term,
        Err(e) => {
            return e.report();
        }
    };
    let r = App::new(Globals {
        options,
        high_scores,
    })
    .run(terminal)
    .map_err(MainError::App);
    let code = if let Err(e) = restore_terminal() {
        e.report()
    } else {
        ExitCode::SUCCESS
    };
    match r {
        Ok(()) => code,
        Err(e) => e.report(),
    }
}

/// Initialize the terminal, including enabling focus events
fn init_terminal() -> Result<ratatui::DefaultTerminal, MainError> {
    let terminal = ratatui::try_init().map_err(MainError::Init)?;
    match execute!(io::stdout(), EnableFocusChange) {
        Ok(()) => Ok(terminal),
        Err(e) => {
            ratatui::restore();
            Err(MainError::Init(e))
        }
    }
}

/// Clean up the terminal, undoing the changes made by [`init_terminal()`]
fn restore_terminal() -> Result<(), MainError> {
    execute!(io::stdout(), DisableFocusChange)
        .and(ratatui::try_restore())
        .map_err(MainError::Restore)
}

#[derive(Debug, Error)]
enum MainError {
    /// An error occurred while initializing the terminal
    #[error("failed to set up terminal: {0}")]
    Init(io::Error),

    /// An error occurred inside the application proper
    #[error(transparent)]
    App(io::Error),

    /// An error occurred while cleaning up the terminal
    #[error("failed to clean up terminal: {0}")]
    Restore(io::Error),
}

impl MainError {
    /// If the error is not due to a broken pipe, print an error message to
    /// stderr and return a failure exit code.
    fn report(&self) -> ExitCode {
        if matches!(self, MainError::App(e) if e.kind() == ErrorKind::BrokenPipe) {
            ExitCode::SUCCESS
        } else {
            let _ = writeln!(io::stderr().lock(), "ratsnake: {self}");
            ExitCode::from(2)
        }
    }
}
