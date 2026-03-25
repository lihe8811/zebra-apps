#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderRequest {
    pub model: String,
    pub prompt: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderResponse {
    pub content: String,
}

pub trait LlmProvider {
    fn execute(&self, request: &ProviderRequest) -> Result<ProviderResponse, String>;
}
