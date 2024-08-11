use crate::ui::file_explore::FileExplorer;
use crate::ui::{Chat, Formatter, Help, History, Notification, Prompt, Spinner};
use std;
use std::sync::atomic::AtomicBool;

use crate::config::Config;
use arboard::Clipboard;
use crossterm::event::KeyCode;
use ratatui::text::Line;

use std::sync::Arc;

use crate::set_language;

pub type AppResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone, PartialEq)]
pub enum FocusedBlock {
    Prompt,
    Chat,
    History,
    Preview,
    Help,
    FileExplorer,
    FileExplorerPreview,
}

pub struct App<'a> {
    pub running: bool,
    pub prompt: Prompt<'a>,
    pub chat: Chat<'a>,
    pub focused_block: FocusedBlock,
    pub history: History<'a>,
    pub file_explorer: FileExplorer,
    pub notifications: Vec<Notification>,
    pub spinner: Spinner,
    pub terminate_response_signal: Arc<AtomicBool>,
    pub clipboard: Option<Clipboard>,
    pub help: Help,
    pub previous_key: KeyCode,
    pub config: Arc<Config>,
    pub formatter: &'a Formatter<'a>,
    // pub chat_scroll: usize,
}

impl<'a> App<'a> {
    pub fn new(config: Arc<Config>, formatter: &'a Formatter<'a>) -> Self {
        // 设置语言
        set_language(&config.language);

        Self {
            running: true,
            prompt: Prompt::default(),
            chat: Chat::new(),
            focused_block: FocusedBlock::Prompt,
            history: History::new(),
            file_explorer: FileExplorer::new(&config.file_explorer_path),
            notifications: Vec::new(),
            spinner: Spinner::default(),
            terminate_response_signal: Arc::new(AtomicBool::new(false)),
            clipboard: Clipboard::new().ok(),
            help: Help::new(),
            previous_key: KeyCode::Null,
            config,
            formatter,
        }
    }

    pub fn tick(&mut self) {
        self.notifications.retain(|n| n.ttl > 0);
        self.notifications.iter_mut().for_each(|n| n.ttl -= 1);

        if self.spinner.active {
            self.chat.formatted_chat.lines.pop();
            self.chat
                .formatted_chat
                .lines
                .push(Line::raw(format!("🤖: Waiting {}", self.spinner.draw())));
            self.spinner.update();
        }
    }
}
