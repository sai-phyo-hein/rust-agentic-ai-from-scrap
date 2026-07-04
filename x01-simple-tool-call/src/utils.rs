use std::path::PathBuf;
use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
use reqwest::Client;
use std::env;

use crate::tools::Tool;
use crate::messages::{Message, ToolCall};
use crate::prompts::get_system_prompt;

pub fn sandbox_root() -> Result<PathBuf> {
    Ok(std::env::current_dir()?.canonicalize()?)
}

#[derive(Debug, Serialize)]
struct Request<'a> {
    model: &'a str,
    max_tokens: usize,
    system: &'a str,
    tools: &'a [Tool],
    messages: &'a [Message]
}

#[derive(Debug, Deserialize)]
pub struct Response {
    pub choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub message: ResponseMessage,
    #[serde(default)]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ResponseMessage {
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub tool_calls: Option<Vec<ToolCall>>,
}

pub async fn send(
    tools: &[Tool],
    messages: &[Message]
) -> Result<Response> {

    dotenvy::dotenv().ok();
    let model = env::var("MODEL").context("MODEL not set")?;
    let max_tokens: usize = env::var("MAX_TOKEN")
        .context("MAX_TOKEN not set")?
        .parse()
        .context("MAX_TOKENS is not a valid number")?;
    let model_url = env::var("MODEL_URL").context("MODEL_URL not set")?;
    let api_key = env::var("OPENROUTER_API_KEY").context("OPENROUTER_API_KEY not set");

    let body = Request {
        model: &model,
        max_tokens,
        system: &get_system_prompt(),
        tools,
        messages,
    };

    let http = Client::new();

    let raw = http
        .post(&model_url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {:}", api_key.unwrap()))
        .json(&body)
        .send()
        .await
        .context("failed to send request")?;

    let status = raw.status();
    let text = raw.text().await.context("fail to read response body")?;

    if !status.is_success() {
        anyhow::bail!("API error {status}: {text}");
    }

    let response: Response = serde_json::from_str(&text).context("failed to parse response")?;
    
    Ok(response)

}