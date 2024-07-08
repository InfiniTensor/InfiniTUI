use crossterm::event::{KeyCode, KeyEvent};

/// Input enum to represent the fours different actions available inside a [`FileExplorer`](crate::FileExplorer).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Input {
    Up,
    Down,
    Left,
    Right,
    None,
}

impl From<&KeyEvent> for Input {
    /// Convert crossterm [`Event`](https://docs.rs/crossterm/latest/crossterm/event/enum.Event.html) to [`Input`].
    ///
    /// **Note:** This implementation is only available when the `crossterm` feature is enabled.
    fn from(value: &KeyEvent) -> Self {
        if matches!(
            value.kind,
            crossterm::event::KeyEventKind::Press | crossterm::event::KeyEventKind::Repeat
        ) {
            let input = match value.code {
                KeyCode::Char('j') | KeyCode::Down => Input::Down,
                KeyCode::Char('k') | KeyCode::Up => Input::Up,
                KeyCode::Char('h') | KeyCode::Left | KeyCode::Backspace => Input::Left,
                KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => Input::Right,
                _ => Input::None,
            };

            return input;
        }

        Input::None
    }
}
