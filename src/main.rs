mod app;
mod consts;
mod util;
use crate::app::App;
use std::io::{self, ErrorKind};
use std::process::ExitCode;

fn main() -> ExitCode {
    let terminal = ratatui::init();
    let r = App::new(rand::rng()).run(terminal);
    ratatui::restore();
    io_exit(r)
}

fn io_exit(r: io::Result<()>) -> ExitCode {
    match r {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) if e.kind() == ErrorKind::BrokenPipe => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(2)
        }
    }
}
