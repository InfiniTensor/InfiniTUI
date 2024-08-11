use std;

use ratatui::prelude::*;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use rust_i18n::t;

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
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // 标题栏
            Constraint::Min(1),     // 聊天区域
            Constraint::Length(3),  // 输入框
            Constraint::Length(1),  // 状态栏
        ])
        .split(frame.size());

    // 渲染标题栏
    render_title_bar(frame, chunks[0]);

    // 渲染聊天区域
    // 渲染聊天区域
        app.chat.render(frame, chunks[1]);

    // 渲染输入框
    app.prompt.render(frame, chunks[2]);

    // 渲染状态栏
    render_status_bar(frame, chunks[3]);

    // 渲染其他弹出窗口 (如果需要)
    render_popups(app, frame);
}

fn render_title_bar(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new(t!("ai_chat_title"))
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center);
    frame.render_widget(title, area);
}

fn render_status_bar(frame: &mut Frame, area: Rect) {
    let status = Paragraph::new(t!("status_area"))
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left);
    frame.render_widget(status, area);
}

fn render_popups(app: &mut App, frame: &mut Frame) {
    let frame_size = frame.size();
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
