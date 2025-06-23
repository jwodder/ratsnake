mod app;
mod command;
mod consts;
mod game;
mod menu;
mod options;
mod util;
mod warning;
use crate::app::App;
use crossterm::{
    event::{DisableFocusChange, EnableFocusChange},
    execute,
};
use std::io::{self, ErrorKind, Write};
use std::process::ExitCode;
use thiserror::Error;

fn main() -> ExitCode {
    let terminal = match init_terminal() {
        Ok(term) => term,
        Err(e) => {
            return e.report();
        }
    };
    let r = App::new().run(terminal).map_err(MainError::App);
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

fn restore_terminal() -> Result<(), MainError> {
    execute!(io::stdout(), DisableFocusChange)
        .and(ratatui::try_restore())
        .map_err(MainError::Restore)
}

#[derive(Debug, Error)]
enum MainError {
    #[error("failed to set up terminal: {0}")]
    Init(io::Error),
    #[error(transparent)]
    App(io::Error),
    #[error("failed to clean up terminal: {0}")]
    Restore(io::Error),
}

impl MainError {
    fn report(&self) -> ExitCode {
        if matches!(self, MainError::App(e) if e.kind() == ErrorKind::BrokenPipe) {
            ExitCode::SUCCESS
        } else {
            let mut stderr = io::stderr().lock();
            if let Err(err) = writeln!(stderr, "ratsnake: {self}") {
                if err.kind() == ErrorKind::BrokenPipe {
                    return ExitCode::SUCCESS;
                }
            }
            ExitCode::from(2)
        }
    }
}
