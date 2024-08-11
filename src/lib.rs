#[macro_use]
extern crate rust_i18n;

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

use clap::{Arg, Command};

use rust_i18n::{i18n, locale};

i18n!("locales", fallback="en");

pub fn set_language(lang: &str) {
    rust_i18n::set_locale(lang);
    // let locale = rust_i18n::locale();
    // println!("Language set to: {:?}", &*locale);
}

pub fn cli() -> Command {
    Command::new("infini")
            .about("TUI LLM Chat for InfiniLM")
            .arg(Arg::new("lang")
                .short('l')
                .long("lang")
                .value_name("LANGUAGE")
                .help("Sets the display language (e.g., en, zh-CN)"))
}
