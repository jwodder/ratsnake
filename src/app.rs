use crate::game::Game;
use crate::menu::MainMenu;
use crate::options::Options;
use ratatui::{backend::Backend, Terminal};

#[derive(Clone, Debug)]
pub(crate) struct App {
    screen: Screen,
}

impl App {
    pub(crate) fn new(opts: Options) -> App {
        let screen = Screen::Main(MainMenu::new(opts));
        App { screen }
    }

    pub(crate) fn run<B: Backend>(mut self, mut terminal: Terminal<B>) -> std::io::Result<()> {
        while !self.quitting() {
            self.draw(&mut terminal)?;
            self.process_input()?;
        }
        Ok(())
    }

    fn draw<B: Backend>(&self, terminal: &mut Terminal<B>) -> std::io::Result<()> {
        match self.screen {
            Screen::Main(ref menu) => {
                terminal.draw(|frame| menu.draw(frame))?;
            }
            Screen::Game(ref game) => {
                terminal.draw(|frame| game.draw(frame))?;
            }
            Screen::Quit => (),
        }
        Ok(())
    }

    fn process_input(&mut self) -> std::io::Result<()> {
        match self.screen {
            Screen::Main(ref mut menu) => {
                if let Some(screen) = menu.process_input()? {
                    self.screen = screen;
                }
            }
            Screen::Game(ref mut game) => {
                if let Some(screen) = game.process_input()? {
                    self.screen = screen;
                }
            }
            Screen::Quit => (),
        }
        Ok(())
    }

    fn quitting(&self) -> bool {
        matches!(self.screen, Screen::Quit)
    }
}

#[derive(Clone, Debug)]
pub(crate) enum Screen {
    Main(MainMenu),
    Game(Game),
    Quit,
}
