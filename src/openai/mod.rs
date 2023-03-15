use std::error::Error;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use surf::middleware::Middleware;
use surf::middleware::Next;
use surf::utils::async_trait;
use surf::Client;
use surf::Config;
use surf::Request;
use surf::Response;
use surf::StatusCode;
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

#[async_trait]
impl Middleware for BearerToken {
    async fn handle(
        &self,
        mut req: Request,
        client: Client,
        next: Next<'_>,
    ) -> surf::Result<Response> {
        log::debug!("Request: {:?}", req);
        req.insert_header("Authorization", format!("Bearer {}", self.token));
        let response: Response = next.run(req, client).await?;
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
    #[error("Base URL not set")]
    BaseUrlNotSet,
    #[error("The diff is too large for the OpenAI API. Try reducing the number of staged changes, or write your own commit message.")]
    DiffTooLarge,
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

pub struct OpenAiClient {
    client: Client,
}

impl OpenAiClient {
    pub fn new(token: &str, base_url: &str) -> Result<Self, Box<dyn Error>> {
        let client: Client = Config::new()
            .set_base_url(Url::parse(base_url).unwrap())
            .try_into()?;
        Ok(Self {
            client: client.with(BearerToken::new(token)),
        })
    }

    async fn post<B, R>(&self, endpoint: &str, body: B) -> Result<R, Box<dyn Error>>
    where
        B: Serialize,
        R: DeserializeOwned,
    {
        let base_url = self
            .client
            .config()
            .base_url
            .as_ref()
            .ok_or(AppError::BaseUrlNotSet)?;

        let mut response = self
            .client
            .post(&format!("{}{}", base_url, endpoint))
            .body(surf::Body::from_json(&body)?)
            .await?;
        match response.status() {
            StatusCode::Ok => Ok(response.body_json::<R>().await?),
            _ => Err(Box::new(AppError::APIError(
                response
                    .body_json::<ErrorWrapper>()
                    .await
                    .expect("The API has returned something funky")
                    .error,
            ))),
        }
    }

    async fn complete_prompt(
        &self,
        completion_request: CompletionRequest,
    ) -> Result<CompletionResponse, Box<dyn Error>> {
        self.post("completions", completion_request).await
    }

    async fn create_completion(&self, prompt: &str) -> Result<CompletionResponse, Box<dyn Error>> {
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

        let completion: CompletionResponse = self.complete_prompt(completion_request).await?;
        Ok(completion)
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

fn sanitize_message(message: &str) -> String {
    message
        .trim()
        .replace(['\n', '\r'], "")
        .replace(r"\w\.$", "$1")
}

pub struct GitCommitMessageGenerator {
    openai_client: OpenAiClient,
}

impl GitCommitMessageGenerator {
    pub fn new(openai_client: OpenAiClient) -> Self {
        Self { openai_client }
    }

    pub async fn generate_commit_message(&self, diff: &str) -> Result<String, Box<dyn Error>> {
        let prompt: &str = &format!("{}\n{}", PROMPT, diff);

        if prompt.len() > 4000 {
            return Err(AppError::DiffTooLarge.into());
        }

        let completion: CompletionResponse = self.openai_client.create_completion(prompt).await?;

        let message: String = sanitize_message(&completion.choices[0].text);
        Ok(message)
    }
}
