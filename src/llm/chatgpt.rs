use super::*;

use reqwest::header::HeaderValue;
use reqwest::Client;

// use async_openai::{Client, types::{CreateChatCompletionResponse, CreateChatCompletionRequest, ChatCompletionRequestMessage, Role, CreateEmbeddingRequest, EmbeddingInput}};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

use crate::config::ChatGPTConfig;
use regex::Regex;

#[derive(Clone, Debug)]
pub struct ChatGPT {
    client: reqwest::Client,
    openai_api_key: String,
    model: String,
    url: String,
    messages: Vec<HashMap<String, String>>,
}

impl ChatGPT {
    pub fn new(config: ChatGPTConfig) -> Self {
        let openai_api_key = match std::env::var("OPENAI_API_KEY") {
            Ok(key) => key,
            Err(_) => config
                .openai_api_key
                .ok_or_else(|| {
                    eprintln!(
                        r#"Can not find the openai api key
You need to define one wether in the configuration file or as an environment variable"#
                    );

                    std::process::exit(1);
                })
                .unwrap(),
        };

        Self {
            client: reqwest::Client::new(),
            openai_api_key,
            model: config.model,
            url: config.url,
            messages: Vec::new(),
        }
    }
}

#[async_trait]
impl LLM for ChatGPT {
    fn clear(&mut self) {
        self.messages = Vec::new();
    }

    fn append_chat_msg(&mut self, msg: String, role: LLMRole) {
        let mut conv: HashMap<String, String> = HashMap::new();
        conv.insert("role".to_string(), role.to_string());
        conv.insert("content".to_string(), msg);
        self.messages.push(conv);
    }

    // For UI
    async fn ask(
        &self,
        sender: UnboundedSender<Event>,
        terminate_response_signal: Arc<AtomicBool>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse()?);
        headers.insert(
            "Authorization",
            format!("Bearer {}", self.openai_api_key).parse()?,
        );

        let mut messages: Vec<HashMap<String, String>> = super::read_default_prompts();

        messages.extend(self.messages.clone());

        let body: Value = json!({
            "model": self.model,
            "messages": messages,
            "stream": true,
            "temperature": 0.1,
        });

        let response = self
            .client
            .post(&self.url)
            .headers(headers)
            .json(&body)
            .send()
            .await?;

        match response.error_for_status() {
            Ok(mut res) => {
                let mut last_string = String::new();

                sender.send(Event::LLMEvent(LLMAnswer::StartAnswer))?;
                while let Some(chunk) = res.chunk().await? {
                    let mut chunk = std::str::from_utf8(&chunk)?.to_owned();
                    if !last_string.is_empty() {
                        chunk = last_string + &chunk;
                        last_string = "".into();
                    }

                    let re = Regex::new(r"data:\s(.*)")?;

                    for captures in re.captures_iter(&chunk) {
                        if let Some(data_json) = captures.get(1) {
                            if terminate_response_signal.load(Ordering::Relaxed) {
                                sender.send(Event::LLMEvent(LLMAnswer::EndAnswer))?;
                                return Ok(());
                            }

                            if data_json.as_str() == "[DONE]" {
                                sender.send(Event::LLMEvent(LLMAnswer::EndAnswer))?;
                                return Ok(());
                            }

                            // fix EOF
                            let answer: Value = match serde_json::from_str(data_json.as_str()) {
                                Ok(answer) => answer,
                                Err(_e) => {
                                    last_string = data_json.as_str().to_string();
                                    continue;
                                }
                            };

                            let msg = answer["choices"][0]["delta"]["content"]
                                .as_str()
                                .unwrap_or("\n");

                            if msg != "null" {
                                sender.send(Event::LLMEvent(LLMAnswer::Answer(msg.to_string())))?;
                            }

                            sleep(Duration::from_millis(100)).await;
                        }
                    }
                }
            }
            Err(e) => return Err(Box::new(e)),
        }

        Ok(())
    }
}

// for test
pub async fn call_gpt(messages: Vec<Message>) -> Result<String, Box<dyn std::error::Error + Send>> {
    dotenv().ok();

    // Extract API Key information
    let api_key: String =
        env::var("OPEN_AI_KEY").expect("OPEN_AI_KEY not found in enviornment variables");
    // let api_org: String =
    // env::var("OPEN_AI_ORG").expect("OPEN_AI_ORG not found in enviornment variables");

    // Confirm endpoint
    let url: &str = "https://api.openai.com/v1/chat/completions";

    // Create headers
    let mut headers: HeaderMap = HeaderMap::new();

    // Create api key header
    headers.insert(
        "authorization",
        HeaderValue::from_str(&format!("Bearer {}", api_key))
            .map_err(|e| -> Box<dyn std::error::Error + Send> { Box::new(e) })?,
    );

    let client: Client = Client::builder()
        .default_headers(headers)
        .build()
        .map_err(|e| -> Box<dyn std::error::Error + Send> { Box::new(e) })?;

    let chat_completion: ChatCompletion = ChatCompletion {
        model: "gpt-3.5-turbo".to_string(),
        messages,
        temperature: 0.1,
    };

    // Extract API Response
    let res: APIResponse = client
        .post(url)
        .json(&chat_completion)
        .send()
        .await
        .map_err(|e| -> Box<dyn std::error::Error + Send> { Box::new(e) })?
        .json()
        .await
        .map_err(|e| -> Box<dyn std::error::Error + Send> { Box::new(e) })?;

    // Send Response
    Ok(res.choices[0].message.content.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn tests_call_to_openai() {
        let message: Message = Message {
            role: "user".to_string(),
            content: "Hi there, this is a test. Give me a short reponse.".to_string(),
        };

        let messages: Vec<Message> = vec![message];

        let res: Result<String, Box<dyn std::error::Error + Send>> = call_gpt(messages).await;
        match res {
            Ok(res_str) => {
                dbg!(res_str);
                assert!(true);
            }
            Err(_) => {
                assert!(false);
            }
        }
    }
}
