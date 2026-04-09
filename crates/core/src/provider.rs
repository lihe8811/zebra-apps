use anyhow::{anyhow, Context, Result};
use async_openai::config::OpenAIConfig;
use async_openai::types::responses::{CreateResponseArgs, Response};
use async_openai::Client;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderRequest {
    pub model: String,
    pub instructions: String,
    pub input: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderResponse {
    pub content: String,
}

pub struct OpenAiResponsesClient {
    client: Client<OpenAIConfig>,
}

impl OpenAiResponsesClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn execute(&self, request: &ProviderRequest) -> Result<ProviderResponse> {
        let response = self
            .client
            .responses()
            .create(
                CreateResponseArgs::default()
                    .model(request.model.clone())
                    .instructions(request.instructions.clone())
                    .input(request.input.clone())
                    .build()
                    .context("failed to build OpenAI response request")?,
            )
            .await
            .context("OpenAI response request failed")?;

        extract_response_text(response)
    }
}

fn extract_response_text(response: Response) -> Result<ProviderResponse> {
    let content = response
        .output_text()
        .filter(|text| !text.trim().is_empty())
        .ok_or_else(|| anyhow!("OpenAI response did not contain output text"))?;

    Ok(ProviderResponse { content })
}
