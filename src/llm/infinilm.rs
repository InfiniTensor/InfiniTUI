use super::*;

use crate::config::InfiniLMConfig;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use regex::Regex;

#[derive(Clone, Debug)]
pub struct InfiniLM {
    client: reqwest::Client,
    url: String,
    messages: Vec<HashMap<String, String>>,
}

impl InfiniLM {
    pub fn new(config: InfiniLMConfig) -> Self {
        Self {
            client: reqwest::Client::new(),
            url: config.url,
            messages: Vec::new(),
        }
    }
}

#[async_trait]
impl LLM for InfiniLM {
    fn clear(&mut self) {
        self.messages = Vec::new();
    }

    fn append_chat_msg(&mut self, msg: String, role: LLMRole) {
        let mut conv: HashMap<String, String> = HashMap::new();
        conv.insert("role".to_string(), role.to_string());
        conv.insert("content".to_string(), msg);
        self.messages.push(conv);
    }

    async fn ask(
        &self,
        sender: UnboundedSender<Event>,
        terminate_response_signal: Arc<AtomicBool>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse()?);

        let mut messages: Vec<HashMap<String, String>> = super::read_default_prompts();

        messages.extend(self.messages.clone());

        // "inputs": [{"role": "user", "content": "用rust写个topk"}]
        let body: Value = json!({
            "inputs":messages,
            "encoding": "text",
            "temperature": 0.9,
            "top-k": 100,
            "top-p": 0.9,
            "stream": true,
        });

        info!("InfiniLM body data json: {:?} ", body);

        let response = reqwest::ClientBuilder::new()
                            .no_proxy() // 本地模型不需要走代理
                            .build()
                            .unwrap()
                            .post(&self.url)
                            .headers(headers)
                            .json(&body)
                            .send()
                            .await?;

        match response.error_for_status() {
            Ok(mut res) => {
                sender.send(Event::LLMEvent(LLMAnswer::StartAnswer))?;
                while let Some(chunk) = res.chunk().await? {
                    let chunk = std::str::from_utf8(&chunk)?.to_owned();

                    info!("InfiniLM chunk data json: {:?} ", chunk);

                    if terminate_response_signal.load(Ordering::Relaxed) {
                        sender.send(Event::LLMEvent(LLMAnswer::EndAnswer))?;
                        return Ok(());
                    }
                    sender.send(Event::LLMEvent(LLMAnswer::Answer(chunk)))?;
                }
            }
            Err(e) => return Err(Box::new(e)),
        }

        sender.send(Event::LLMEvent(LLMAnswer::EndAnswer))?;

        Ok(())
    }
}
