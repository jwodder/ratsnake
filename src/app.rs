use crate::game::Game;
use crate::options::Options;
use crate::startup::StartupScreen;
use ratatui::{backend::Backend, Terminal};
use std::io;

#[derive(Clone, Debug)]
pub(crate) struct App {
    state: AppState,
}

impl App {
    pub(crate) fn new() -> App {
        let state = AppState::Startup(StartupScreen::new(Options::default()));
        App { state }
    }

    pub(crate) fn run<B: Backend>(mut self, mut terminal: Terminal<B>) -> io::Result<()> {
        while !self.quitting() {
            self.draw(&mut terminal)?;
            self.process_input()?;
        }
        Ok(())
    }

    fn draw<B: Backend>(&self, terminal: &mut Terminal<B>) -> io::Result<()> {
        match self.state {
            AppState::Startup(ref startup) => {
                terminal.draw(|frame| startup.draw(frame))?;
            }
            AppState::Game(ref game) => {
                terminal.draw(|frame| game.draw(frame))?;
            }
            AppState::Quit => (),
        }
        Ok(())
    }

    fn process_input(&mut self) -> io::Result<()> {
        match self.state {
            AppState::Startup(ref mut startup) => {
                if let Some(state) = startup.process_input()? {
                    self.state = state;
                }
            }
            AppState::Game(ref mut game) => {
                if let Some(state) = game.process_input()? {
                    self.state = state;
                }
            }
            AppState::Quit => (),
        }
        Ok(())
    }

    fn quitting(&self) -> bool {
        matches!(self.state, AppState::Quit)
    }
}

#[derive(Clone, Debug)]
pub(crate) enum AppState {
    Startup(StartupScreen),
    Game(Game),
    Quit,
}
