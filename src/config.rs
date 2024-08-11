use crate::llm::LLMBackend;
use toml;

use dirs;
use serde::Deserialize;

use std::env;
use std::path::PathBuf;
use tracing::info;

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default = "default_archive_file_name")]
    pub archive_file_name: String,

    #[serde(default)]
    pub key_bindings: KeyBindings,

    #[serde(default = "default_llm_backend")]
    pub llm: LLMBackend,

    #[serde(default)]
    pub chatgpt: ChatGPTConfig,

    #[serde(default)]
    pub chatglm: ChatGLMConfig,

    // pub llamacpp: Option<LLamacppConfig>,
    pub ollama: Option<OllamaConfig>,

    pub infinilm: Option<InfiniLMConfig>,

    // pub file_explore_path
    #[serde(default = "default_file_root")]
    pub file_explorer_path: String,

    #[serde(default = "default_language")]
    pub language: String,
}

pub fn default_language() -> String {
    "zh-CN".to_string()
}

pub fn default_archive_file_name() -> String {
    String::from("infini.archive.md")
}

pub fn default_llm_backend() -> LLMBackend {
    LLMBackend::ChatGLM
}

fn default_file_root() -> String {
    "./files/".to_string()
}

// ChatGLM
#[derive(Deserialize, Debug, Clone)]
pub struct ChatGLMConfig {
    pub chatglm_api_key: Option<String>,

    #[serde(default = "ChatGLMConfig::default_model")]
    pub model: String,

    #[serde(default = "ChatGLMConfig::default_url")]
    pub url: String,
}

impl Default for ChatGLMConfig {
    fn default() -> Self {
        Self {
            chatglm_api_key: None,
            model: Self::default_model(),
            url: Self::default_url(),
        }
    }
}

impl ChatGLMConfig {
    pub fn default_model() -> String {
        String::from("glm-4")
    }

    pub fn default_url() -> String {
        String::from("https://open.bigmodel.cn/api/paas/v4/chat/completions")
    }
}

// ChatGPT
#[derive(Deserialize, Debug, Clone)]
pub struct ChatGPTConfig {
    pub openai_api_key: Option<String>,

    #[serde(default = "ChatGPTConfig::default_model")]
    pub model: String,

    #[serde(default = "ChatGPTConfig::default_url")]
    pub url: String,
}

impl Default for ChatGPTConfig {
    fn default() -> Self {
        Self {
            openai_api_key: None,
            model: Self::default_model(),
            url: Self::default_url(),
        }
    }
}

impl ChatGPTConfig {
    pub fn default_model() -> String {
        String::from("gpt-3.5-turbo")
    }

    pub fn default_url() -> String {
        String::from("https://api.openai.com/v1/chat/completions")
    }
}

// Ollama

#[derive(Deserialize, Debug, Clone)]
pub struct OllamaConfig {
    pub url: String,
    pub model: String,
}

// InfiniLM
#[derive(Deserialize, Debug, Clone)]
pub struct InfiniLMConfig {
    pub url: String,
    // pub model: String,
}

// Key Bindings

#[derive(Deserialize, Debug)]
pub struct KeyBindings {
    #[serde(default = "KeyBindings::default_show_help")]
    pub show_help: char,

    #[serde(default = "KeyBindings::default_show_history")]
    pub show_history: char,

    #[serde(default = "KeyBindings::default_new_chat")]
    pub new_chat: char,

    #[serde(default = "KeyBindings::default_save_chat")]
    pub save_chat: char,

    #[serde(default = "KeyBindings::default_stop_stream")]
    pub stop_stream: char,

    #[serde(default = "KeyBindings::default_show_file_explorer")]
    pub show_file_explorer: char,

    #[serde(default = "KeyBindings::default_code_to_prompt")]
    pub code_to_prompt: char,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            show_help: '?',
            show_history: 'h',
            new_chat: 'n',
            save_chat: 's',
            stop_stream: 't',
            show_file_explorer: 'f',
            code_to_prompt: 'p',
        }
    }
}

impl KeyBindings {
    fn default_show_help() -> char {
        '?'
    }

    fn default_show_history() -> char {
        'h'
    }

    fn default_new_chat() -> char {
        'n'
    }

    fn default_save_chat() -> char {
        's'
    }

    fn default_stop_stream() -> char {
        't'
    }

    fn default_show_file_explorer() -> char {
        'f'
    }

    fn default_code_to_prompt() -> char {
        'p'
    }
}

impl Config {
    pub fn load() -> Self {
        let conf_dir = match env::var("CONFIG_DIR") {
            Ok(dir) => {
                info!("Using custom config directory: {}", dir);
                PathBuf::from(dir)
            }
            Err(env::VarError::NotPresent) => {
                let default_dir = dirs::config_dir().unwrap().join("infini");
                info!("Using default config directory: {:?}", default_dir);
                default_dir
            }
            Err(e) => {
                panic!("Failed to read CONFIG_DIR: {}", e);
            }
        };

        // 构造完整的配置文件路径
        let conf_path = conf_dir.join("config.toml");

        let config = std::fs::read_to_string(&conf_path)
            .unwrap_or_else(|_| panic!("Failed to read config file: {:?}", conf_path));
        let mut app_config: Config = toml::from_str(&config)
            .unwrap_or_else(|_| panic!("Failed to parse config file: {:?}", conf_path));

        if app_config.llm == LLMBackend::Ollama && app_config.ollama.is_none() {
            eprintln!("Config for Ollama is not provided");
            std::process::exit(1);
        }

        eprintln!("app_config.language {:?}", app_config.language);

        app_config
    }
}
