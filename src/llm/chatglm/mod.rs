use super::*;

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

use crate::config::ChatGLMConfig;
use regex::Regex;

use reqwest::header::HeaderValue;
use reqwest::Client;

pub mod api_operation;
pub mod custom_jwt;

#[derive(Clone, Debug)]
pub struct ChatGLM {
    client: reqwest::Client,
    chatglm_api_key: String,
    model: String,
    url: String,
    messages: Vec<HashMap<String, String>>,
}

impl ChatGLM {
    pub fn new(config: ChatGLMConfig) -> Self {
        let chatglm_api_key = match std::env::var("CHATML_API_KEY") {
            Ok(key) => key,
            Err(_) => config
                .chatglm_api_key
                .ok_or_else(|| {
                    eprintln!(
                        r#"Can not find the ChatGLM api key
You need to define one wether in the configuration file or as an environment variable"#
                    );

                    std::process::exit(1);
                })
                .unwrap(),
        };

        Self {
            client: reqwest::Client::new(),
            chatglm_api_key,
            model: config.model,
            url: config.url,
            messages: Vec::new(),
        }
    }

    pub fn sign_token(&self) -> String {
        let api_key_instance = api_operation::APIKeys::get_instance(&self.chatglm_api_key);
        let jwt_creator = custom_jwt::CustomJwt::new(
            api_key_instance.get_user_id(),
            api_key_instance.get_user_secret(),
        );
        let jwt = jwt_creator.create_jwt();

        let jwt_to_verify = jwt.clone();
        let is_valid = jwt_creator.verify_jwt(&jwt_to_verify);

        if is_valid {
            return jwt;
        } else {
            panic!("JWT is not valid");
        }
    }
}

#[async_trait]
impl LLM for ChatGLM {
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
        let jwt = self.sign_token();
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse()?);
        headers.insert("Authorization", format!("Bearer {}", jwt).parse()?);

        // let mut messages: Vec<HashMap<String, String>> = vec![
        //     (HashMap::from([
        //         ("role".to_string(), "system".to_string()),
        //         (
        //             "content".to_string(),
        //             "You are a helpful assistant.".to_string(),
        //         ),
        //     ])),
        // ];

        let mut messages: Vec<HashMap<String, String>> = super::read_default_prompts();

        messages.extend(self.messages.clone());

        let body: Value = json!({
            "model": self.model,
            "messages": messages,
            "stream": true,
        });

        // FIXME: support sse

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
                    // info!("ChatGLM response chunk: {}", chunk);

                    let re = Regex::new(r"data:\s(.*)")?;

                    for captures in re.captures_iter(&chunk) {
                        if let Some(data_json) = captures.get(1) {
                            // info!("ChatGLM response data json: {:?} ", data_json);

                            if terminate_response_signal.load(Ordering::Relaxed) {
                                sender.send(Event::LLMEvent(LLMAnswer::EndAnswer))?;
                                return Ok(());
                            }

                            if data_json.as_str() == "[DONE]" {
                                sender.send(Event::LLMEvent(LLMAnswer::EndAnswer))?;
                                return Ok(());
                            }

                            // FIX EOF
                            let answer: Value = match serde_json::from_str(data_json.as_str()) {
                                Ok(answer) => answer,
                                Err(_e) => {
                                    last_string = data_json.as_str().to_string();
                                    continue;
                                }
                            };

                            let msg = answer["choices"][0]["delta"]["content"].as_str();
                            let mut queue_result = String::new();
                            let mut char_queue = VecDeque::new();

                            if let Some(content) = msg {
                                let get_message = convert_unicode_emojis(content)
                                    .replace("\"", "")
                                    .replace("\\n\\n", "\n")
                                    .replace("\\nn", "\n")
                                    .replace("\\\\n", "\n")
                                    .replace("\\\\nn", "\n")
                                    .replace("\\", "");

                                for c in get_message.chars() {
                                    char_queue.push_back(c);
                                }
                            } else {
                                println!("Invalid JSON format");
                            }

                            queue_result.extend(char_queue);

                            if queue_result != "null" {
                                sender.send(Event::LLMEvent(LLMAnswer::Answer(queue_result)))?;
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

lazy_static::lazy_static! {
    static ref UNICODE_REGEX: regex::Regex = regex::Regex::new(r"\\u[0-9a-fA-F]{4}").unwrap();
}

fn convert_unicode_emojis(input: &str) -> String {
    UNICODE_REGEX
        .replace_all(input, |caps: &regex::Captures| {
            let emoji = char::from_u32(
                u32::from_str_radix(&caps[0][2..], 16).expect("Failed to parse Unicode escape"),
            )
            .expect("Invalid Unicode escape");
            emoji.to_string()
        })
        .to_string()
}

/// use default_josn to avoid EOF error
#[allow(dead_code)]
fn default_json() -> Value {
    json!(
        {
            "choices": [
              {
                "delta": {
                  "content": "!",
                  "role": "assistant"
                },
                "index": 0
              }
            ],
            "created": 1711946953,
            "id": "8530388547035158662",
            "model": "glm-4"
          }
    )
}

// // for test
// pub async fn call_glm(messages: Vec<Message>)-> Result<String, Box<dyn std::error::Error + Send>> {

//     dotenv().ok();

//     // Extract API Key information
//     let api_key: String =
//         env::var("CHATGLM_AI_KEY").expect("OPEN_AI_KEY not found in enviornment variables");
//     // let api_org: String =
//     // env::var("OPEN_AI_ORG").expect("OPEN_AI_ORG not found in enviornment variables");

//     // Confirm endpoint
//     let url: &str = "https://open.bigmodel.cn/api/paas/v4/chat/completions";

//     // Create headers
//     let mut headers: HeaderMap = HeaderMap::new();

//     // Create api key header
//     headers.insert(
//         "authorization",
//         HeaderValue::from_str(&format!("Bearer {}", api_key))
//             .map_err(|e| -> Box<dyn std::error::Error + Send> { Box::new(e) })?,
//     );

//     let client: Client = Client::builder()
//         .default_headers(headers)
//         .build()
//         .map_err(|e| -> Box<dyn std::error::Error + Send> { Box::new(e) })?;

//     let chat_completion: ChatCompletion = ChatCompletion {
//         model:  "glm-4".to_string(),
//         messages,
//         temperature: 0.1,
//     };

//     // Extract API Response
//     let res: APIResponse = client
//         .post(url)
//         .json(&chat_completion)
//         .send()
//         .await
//         .map_err(|e| -> Box<dyn std::error::Error + Send> { Box::new(e) })?
//         .json()
//         .await
//         .map_err(|e| -> Box<dyn std::error::Error + Send> { Box::new(e) })?;

//     // Send Response
//     Ok(res.choices[0].message.content.clone())
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[tokio::test]
//     async fn tests_call_to_chatglm() {

//         let message: Message = Message {
//             role: "user".to_string(),
//             content: "Hi there, this is a test. Give me a short reponse.".to_string(),
//         };

//         let messages: Vec<Message> = vec![message];

//         let res: Result<String, Box<dyn std::error::Error + Send>> = call_glm(messages).await;
//         match res {
//             Ok(res_str) => {
//                 dbg!(res_str);
//                 assert!(true);
//             }
//             Err(e) => {
//                 dbg!(e);
//                 assert!(false);
//             }
//         }
//     }
// }
