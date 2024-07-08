use std;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, FocusedBlock};

pub mod chat;
pub mod file_explore;
pub mod formatter;
pub mod help;
pub mod history;
pub mod notification;
pub mod prompt;
pub mod spinner;
pub mod tui;

pub use chat::Chat;
pub use formatter::Formatter;
pub use help::Help;
pub use history::{History, Preview};
pub use notification::{Notification, NotificationLevel};
pub use prompt::Prompt;
pub use spinner::Spinner;
pub use tui::Tui;

pub type AppResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn notification_rect(offset: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(1 + 5 * offset),
                Constraint::Length(5),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(74),
                Constraint::Percentage(25),
                Constraint::Percentage(1),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

pub fn help_rect(r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(35),
                Constraint::Min(10),
                Constraint::Percentage(35),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Length((r.width - 80) / 2),
                Constraint::Min(80),
                Constraint::Length((r.width - 80) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

pub fn render(app: &mut App, frame: &mut Frame) {
    let frame_size = frame.size();

    let prompt_block_height = app.prompt.height(&frame_size) + 3;

    let (chat_block, prompt_block) = {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(prompt_block_height)].as_ref())
            .split(frame.size());
        (chunks[0], chunks[1])
    };

    // Chat
    app.chat.render(frame, chat_block);

    // Prompt
    app.prompt.render(frame, prompt_block);

    // History
    if let FocusedBlock::History | FocusedBlock::Preview = app.focused_block {
        let area = centered_rect(80, 80, frame_size);
        app.history.render(frame, area, app.focused_block.clone());
    }

    // File explorer
    if let FocusedBlock::FileExplorer | FocusedBlock::FileExplorerPreview = app.focused_block {
        let area = centered_rect(80, 80, frame_size);
        app.file_explorer
            .render(frame, area, app.focused_block.clone(), &app.formatter);
    }

    // Help
    if let FocusedBlock::Help = app.focused_block {
        app.prompt.update(&FocusedBlock::Help);
        let area = help_rect(frame_size);
        app.help.render(frame, area);
    }

    // Notifications
    for (i, notif) in app.notifications.iter_mut().enumerate() {
        let area = notification_rect(i as u16, frame_size);
        notif.render(frame, area);
    }
}
