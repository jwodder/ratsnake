use crate::game::Game;
use crate::menu::MainMenu;
use ratatui::{backend::Backend, Terminal};

/// The `ratsnake` application, the top-level struct for the program
#[derive(Clone, Debug)]
pub(crate) struct App {
    /// What screen are we currently displaying?
    screen: Screen,
}

impl App {
    /// Create a new `App` from the given [`Globals`][crate::util::Globals]
    /// that shows the main menu.
    pub(crate) fn new(globals: crate::util::Globals) -> App {
        let screen = Screen::Main(MainMenu::new(globals));
        App { screen }
    }

    /// Run the application on the given terminal
    pub(crate) fn run<B: Backend>(mut self, mut terminal: Terminal<B>) -> std::io::Result<()> {
        while !self.quitting() {
            self.draw(&mut terminal)?;
            self.process_input()?;
        }
        Ok(())
    }

    /// Draw the current screen on the terminal
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

    /// Receive & handle the next input event or lack thereof
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

    /// Should the application terminate?
    fn quitting(&self) -> bool {
        matches!(self.screen, Screen::Quit)
    }
}

/// An enum of the application's top-level screens.
///
/// Screen values implement the following *de facto* trait:
///
/// ```compile_fail
/// trait Screen {
///     /// Draw the screen on the given frame
///     fn draw(&self, frame: &mut Frame<'_>);
///
///     /// Handle the next input event or lack thereof.  Return `Some(screen)`
///     /// if the application should switch to a new screen.
///     fn process_input(&mut self) -> std::io::Result<Option<Screen>>;
/// }
/// ```
#[derive(Clone, Debug)]
pub(crate) enum Screen {
    /// The main menu screen
    Main(MainMenu),

    /// The gameplay screen
    Game(Game),

    /// A pseudo-screen used to indicate that the application should terminate
    Quit,
}
