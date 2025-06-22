use crate::game::Game;
use crate::mainmenu::MainMenu;
use crate::options::Options;
use ratatui::{backend::Backend, Terminal};
use std::io;

#[derive(Clone, Debug)]
pub(crate) struct App {
    state: AppState,
}

impl App {
    pub(crate) fn new() -> App {
        let state = AppState::Main(MainMenu::new(Options::default()));
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
            AppState::Main(ref menu) => {
                terminal.draw(|frame| menu.draw(frame))?;
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
            AppState::Main(ref mut menu) => {
                if let Some(state) = menu.process_input()? {
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
    Main(MainMenu),
    Game(Game),
    Quit,
}
