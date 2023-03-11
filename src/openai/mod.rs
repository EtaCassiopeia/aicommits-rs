use serde::{Deserialize, Serialize};
use std::error::Error;
use surf::Config;
use surf::Url;
use thiserror::Error;

const PROMPT: &str = "Write an insightful but concise Git commit message in a complete sentence in present tense for the following diff without prefacing it with anything:";

struct BearerToken {
    token: String,
}

impl BearerToken {
    fn new(token: &str) -> Self {
        Self {
            token: String::from(token),
        }
    }
}

#[surf::utils::async_trait]
impl surf::middleware::Middleware for BearerToken {
    async fn handle(
        &self,
        mut req: surf::Request,
        client: surf::Client,
        next: surf::middleware::Next<'_>,
    ) -> surf::Result<surf::Response> {
        log::debug!("Request: {:?}", req);
        req.insert_header("Authorization", format!("Bearer {}", self.token));
        let response: surf::Response = next.run(req, client).await?;
        log::debug!("Response: {:?}", response);
        Ok(response)
    }
}

#[derive(Deserialize, Debug)]
pub(crate) struct ErrorWrapper {
    pub error: ErrorMessage,
}

#[derive(Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct ErrorMessage {
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: String,
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("API returned an Error: {}", .0.message)]
    APIError(ErrorMessage),
}

impl From<ErrorMessage> for AppError {
    fn from(e: ErrorMessage) -> Self {
        AppError::APIError(e)
    }
}

impl From<String> for ErrorMessage {
    fn from(e: String) -> Self {
        ErrorMessage {
            message: e,
            error_type: String::from(""),
        }
    }
}

impl From<String> for AppError {
    fn from(e: String) -> Self {
        AppError::APIError(ErrorMessage::from(e))
    }
}

pub fn async_client(token: &str, base_url: &str) -> Result<surf::Client, Box<dyn Error>> {
    let client: surf::Client = Config::new()
        .set_base_url(Url::parse(base_url).expect("Static string should parse"))
        .try_into()?;

    Ok(client.with(BearerToken::new(token)))
}

async fn post<B, R>(
    async_client: &surf::Client,
    endpoint: &str,
    body: B,
) -> Result<R, Box<dyn Error>>
where
    B: serde::ser::Serialize,
    R: serde::de::DeserializeOwned,
{
    let base_url = async_client
        .config()
        .base_url
        .as_ref()
        .ok_or_else(|| AppError::from(String::from("Base URL not set")))?;

    let mut response = async_client
        .post(&format!("{}{}", base_url, endpoint))
        .body(surf::Body::from_json(&body)?)
        .await?;
    match response.status() {
        surf::StatusCode::Ok => Ok(response.body_json::<R>().await?),
        _ => Err(Box::new(AppError::APIError(
            response
                .body_json::<ErrorWrapper>()
                .await
                .expect("The API has returned something funky")
                .error,
        ))),
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct CompletionRequest {
    model: String,
    prompt: String,
    temperature: f64,
    max_tokens: u64,
    top_p: f64,
    frequency_penalty: f64,
    presence_penalty: f64,
    stream: bool,
    n: u64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Choice {
    pub text: String,
    pub index: u64,
    pub finish_reason: String,
}

pub async fn complete_prompt(
    async_client: &surf::Client,
    completion_request: CompletionRequest,
) -> Result<CompletionResponse, Box<dyn Error>> {
    post(async_client, "completions", completion_request).await
}

async fn create_completion(
    async_client: &surf::Client,
    prompt: &str,
) -> Result<CompletionResponse, Box<dyn Error>> {
    let completion_request: CompletionRequest = CompletionRequest {
        model: String::from("text-davinci-003"),
        prompt: String::from(prompt),
        temperature: 0.7,
        max_tokens: 1500,
        top_p: 1.0,
        frequency_penalty: 0.0,
        presence_penalty: 0.0,
        stream: false,
        n: 1,
    };

    let completion: CompletionResponse = complete_prompt(async_client, completion_request).await?;
    Ok(completion)
}

fn sanitize_message(message: &str) -> String {
    message
        .trim()
        .replace(['\n', '\r'], "")
        .replace(r"\w\.$", "$1")
}

pub async fn generate_commit_message(
    async_client: &surf::Client,
    diff: &str,
) -> Result<String, Box<dyn Error>> {
    let prompt: &str = &format!("{}\n{}", PROMPT, diff);

    if prompt.len() > 4000 {
        return Err("The diff is too large for the OpenAI API. Try reducing the number of staged changes, or write your own commit message.".into());
    }

    let completion: CompletionResponse = create_completion(async_client, prompt).await?;

    let message: String = sanitize_message(&completion.choices[0].text);
    Ok(message)
}
