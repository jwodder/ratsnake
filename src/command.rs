use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// An enum of input commands, (mostly) abstracted away from the key codes that
/// produce them
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Command {
    /// Quit the program (Ctrl-C)
    Quit,
    /// Move up (Up, `w`, `k`, `8`)
    Up,
    /// Move down (Down, `s`, `j`, `2`)
    Down,
    /// Move left (Left, `a`, `h`, `4`)
    Left,
    /// Move right (Right, `d`, `l`, `6`)
    Right,
    /// Select/activate the current button or menu item
    Enter,
    /// User pressed the spacebar
    Space,
    /// Go to the first item in the list (Home)
    Home,
    /// Go to the last item in the list (End)
    End,
    /// Go to the next item, circling around at the end (Tab)
    Next,
    /// Go to the previous item, circling around at the beginning (Shift+Tab)
    Prev,
    /// User pressed the Escape key
    Esc,
    /// User pressed the `m` key
    M,
    /// User pressed the `p` key
    P,
    /// User pressed the `q` key
    Q,
    /// User pressed the `r` key
    R,
}

impl Command {
    /// Return the `Command`, if any, for the given key event
    pub(crate) fn from_key_event(ev: KeyEvent) -> Option<Command> {
        match (ev.modifiers, ev.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('c')) => Some(Command::Quit),
            (KeyModifiers::NONE, KeyCode::Char('w' | 'k' | '8') | KeyCode::Up) => Some(Command::Up),
            (KeyModifiers::NONE, KeyCode::Char('s' | 'j' | '2') | KeyCode::Down) => {
                Some(Command::Down)
            }
            (KeyModifiers::NONE, KeyCode::Char('a' | 'h' | '4') | KeyCode::Left) => {
                Some(Command::Left)
            }
            (KeyModifiers::NONE, KeyCode::Char('d' | 'l' | '6') | KeyCode::Right) => {
                Some(Command::Right)
            }
            (_, KeyCode::Enter) => Some(Command::Enter),
            (KeyModifiers::NONE, KeyCode::Char(' ')) => Some(Command::Space),
            (_, KeyCode::Home) => Some(Command::Home),
            (_, KeyCode::End) => Some(Command::End),
            (_, KeyCode::Tab) => Some(Command::Next),
            (_, KeyCode::BackTab) => Some(Command::Prev),
            (_, KeyCode::Esc) => Some(Command::Esc),
            (KeyModifiers::NONE, KeyCode::Char('m')) => Some(Command::M),
            (KeyModifiers::NONE, KeyCode::Char('p')) => Some(Command::P),
            (KeyModifiers::NONE, KeyCode::Char('q')) => Some(Command::Q),
            (KeyModifiers::NONE, KeyCode::Char('r')) => Some(Command::R),
            _ => None,
        }
    }
}
