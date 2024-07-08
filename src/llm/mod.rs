use crate::config::Config;
use crate::event::Event;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::atomic::AtomicBool;
use strum_macros::Display;
use strum_macros::EnumIter;
use tokio::sync::mpsc::UnboundedSender;

use dirs;
use dotenv::dotenv;
use reqwest::header::HeaderMap;
use serde_json::{json, Value};
use std;
use std::collections::HashMap;
use std::env;
use tracing::info;

pub mod chatglm;
pub mod chatgpt;
pub mod infinilm;
pub mod ollama;

use self::chatglm::ChatGLM;
use self::chatgpt::ChatGPT;
use self::infinilm::InfiniLM;
use self::ollama::Ollama;

use std::fmt::Debug;
use std::sync::Arc;

#[async_trait]
pub trait LLM: Send + Sync + Debug {
    async fn ask(
        &self,
        sender: UnboundedSender<Event>,
        terminate_response_signal: Arc<AtomicBool>,
    ) -> Result<(), Box<dyn std::error::Error>>;

    fn append_chat_msg(&mut self, msg: String, role: LLMRole);
    fn clear(&mut self);
}

#[derive(Clone, Debug)]
pub enum LLMAnswer {
    StartAnswer,
    Answer(String),
    EndAnswer,
}

#[derive(EnumIter, Display, Debug)]
#[strum(serialize_all = "lowercase")]
pub enum LLMRole {
    ASSISTANT,
    SYSTEM,
    USER,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum LLMBackend {
    ChatGPT,
    Ollama,
    ChatGLM,
    InfiniLM,
}

pub struct LLMModel;

impl LLMModel {
    pub async fn init(model: &LLMBackend, config: Arc<Config>) -> Box<dyn LLM> {
        match model {
            LLMBackend::ChatGPT => Box::new(ChatGPT::new(config.chatgpt.clone())),
            LLMBackend::Ollama => Box::new(Ollama::new(config.ollama.clone().unwrap())),
            LLMBackend::ChatGLM => Box::new(ChatGLM::new(config.chatglm.clone())),
            LLMBackend::InfiniLM => Box::new(InfiniLM::new(config.infinilm.clone().unwrap())),
        }
    }
}

pub fn read_default_prompts() -> Vec<HashMap<String, String>> {
    let prompts_path = dirs::config_dir()
        .unwrap()
        .join("infini")
        .join("prompt.toml");

    if !prompts_path.exists() {
        panic!("Error: 'prompts.toml' not found in the current working directory.");
    }

    read_messages_from_toml(&prompts_path.to_str().unwrap())
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ChatCompletion {
    pub model: String,
    pub messages: Vec<Message>,
    pub temperature: f32,
}

#[derive(Debug, Deserialize)]
pub struct APIMessage {
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct APIChoice {
    pub message: APIMessage,
}

#[derive(Debug, Deserialize)]
pub struct APIResponse {
    pub choices: Vec<APIChoice>,
}

#[derive(Deserialize)]
pub struct PromptsConfig {
    messages: Vec<Message>,
}

pub fn read_messages_from_toml(file_path: &str) -> Vec<HashMap<String, String>> {
    let contents = fs::read_to_string(file_path).expect("Failed to read TOML file");
    let config: PromptsConfig = toml::from_str(&contents).expect("Failed to parse TOML");

    config
        .messages
        .iter()
        .map(|msg| {
            let mut map = HashMap::new();
            map.insert("role".to_string(), msg.role.clone());
            map.insert("content".to_string(), msg.content.clone());
            map
        })
        .collect()
}
