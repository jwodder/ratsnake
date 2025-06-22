mod app;
mod command;
mod consts;
mod game;
mod logo;
mod menu;
mod options;
mod util;
use crate::app::App;
use std::io::{ErrorKind, Write};
use std::process::ExitCode;

macro_rules! error {
    ($($arg:tt)*) => {
        {
            let mut stderr = std::io::stderr().lock();
            if let Err(err) = writeln!(stderr, "ratsnake: {}", format_args!($($arg)*)) {
                if err.kind() == ErrorKind::BrokenPipe {
                    return ExitCode::SUCCESS;
                } else {
                    return ExitCode::from(2);
                }
            }
        }
    }
}

fn main() -> ExitCode {
    let terminal = match ratatui::try_init() {
        Ok(term) => term,
        Err(e) => {
            error!("failed to set up terminal: {e}");
            return ExitCode::from(2);
        }
    };
    let r = App::new().run(terminal);
    if let Err(e) = ratatui::try_restore() {
        error!("failed to clean up terminal: {e}");
    }
    match r {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) if e.kind() == ErrorKind::BrokenPipe => ExitCode::SUCCESS,
        Err(e) => {
            error!("{e}");
            ExitCode::from(2)
        }
    }
}
