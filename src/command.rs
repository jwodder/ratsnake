use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Command {
    Quit,
    Up,
    Down,
    Left,
    Right,
    Enter,
    Space,
    Home,
    End,
    Next,
    Prev,
    P,
    Q,
}

impl Command {
    pub(crate) fn from_key_event(ev: KeyEvent) -> Option<Command> {
        match (ev.modifiers, ev.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('c')) => Some(Command::Quit),
            (KeyModifiers::NONE, KeyCode::Char('w' | 'k') | KeyCode::Up) => Some(Command::Up),
            (KeyModifiers::NONE, KeyCode::Char('s' | 'j') | KeyCode::Down) => Some(Command::Down),
            (KeyModifiers::NONE, KeyCode::Char('a' | 'h') | KeyCode::Left) => Some(Command::Left),
            (KeyModifiers::NONE, KeyCode::Char('d' | 'l') | KeyCode::Right) => Some(Command::Right),
            (_, KeyCode::Enter) => Some(Command::Enter),
            (KeyModifiers::NONE, KeyCode::Char(' ')) => Some(Command::Space),
            (_, KeyCode::Home) => Some(Command::Home),
            (_, KeyCode::End) => Some(Command::End),
            (_, KeyCode::Tab) => Some(Command::Next),
            (_, KeyCode::BackTab) => Some(Command::Prev),
            (KeyModifiers::NONE, KeyCode::Char('p')) => Some(Command::P),
            (KeyModifiers::NONE, KeyCode::Char('q')) => Some(Command::Q),
            _ => None,
        }
    }
}
