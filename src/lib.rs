pub mod app;
pub mod config;
pub mod event;
pub mod llm;
pub mod utils;

pub mod ui;

pub use crate::app::{App, AppResult};
pub use crate::config::Config;
pub use crate::event::{handle_key_events, Event, EventHandler};
pub use crate::ui::{Formatter, Tui};

pub use crate::llm::{LLMAnswer, LLMModel, LLMRole, LLM};

use clap::Command;

pub fn cli() -> Command {
    Command::new("infini").about("TUI LLM Chat for InfiniLM ")
}
