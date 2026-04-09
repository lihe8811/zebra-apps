use anyhow::{anyhow, Context, Result};
use base64::Engine;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use yup_oauth2::{read_application_secret, InstalledFlowAuthenticator, InstalledFlowReturnMethod};

const GMAIL_SCOPE: &str = "https://www.googleapis.com/auth/gmail.modify";
const GMAIL_BASE_URL: &str = "https://gmail.googleapis.com/gmail/v1/users";

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct GmailHeader {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct GmailMessageBody {
    pub data: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct GmailMessagePart {
    #[serde(rename = "mimeType")]
    pub mime_type: Option<String>,
    pub filename: Option<String>,
    #[serde(default)]
    pub headers: Vec<GmailHeader>,
    pub body: Option<GmailMessageBody>,
    #[serde(default)]
    pub parts: Vec<GmailMessagePart>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct GmailMessagePayload {
    #[serde(rename = "mimeType")]
    pub mime_type: Option<String>,
    pub filename: Option<String>,
    #[serde(default)]
    pub headers: Vec<GmailHeader>,
    pub body: Option<GmailMessageBody>,
    #[serde(default)]
    pub parts: Vec<GmailMessagePart>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct GmailMessage {
    pub id: String,
    #[serde(rename = "threadId")]
    pub thread_id: String,
    #[serde(rename = "internalDate")]
    pub internal_date: Option<String>,
    #[serde(default)]
    pub snippet: String,
    pub payload: Option<GmailMessagePayload>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct GmailMessageRef {
    pub id: String,
    #[serde(rename = "threadId")]
    pub thread_id: String,
}

#[derive(Debug, Clone, Deserialize)]
struct GmailListResponse {
    #[serde(default)]
    messages: Vec<GmailMessageRef>,
}

#[derive(Debug, Serialize)]
struct GmailModifyRequest<'a> {
    #[serde(rename = "removeLabelIds")]
    remove_label_ids: &'a [&'a str],
}

pub struct GmailApiClient {
    http: Client,
    bearer_token: String,
    user_id: String,
}

impl GmailApiClient {
    pub async fn from_secret_file(
        secret_path: &std::path::Path,
        token_cache_path: &std::path::Path,
        user_id: String,
    ) -> Result<Self> {
        let secret = read_application_secret(secret_path)
            .await
            .with_context(|| format!("failed to read OAuth secret {}", secret_path.display()))?;
        let auth = InstalledFlowAuthenticator::builder(secret, InstalledFlowReturnMethod::HTTPRedirect)
            .persist_tokens_to_disk(token_cache_path)
            .build()
            .await
            .context("failed to build Gmail OAuth authenticator")?;
        let token = auth
            .token(&[GMAIL_SCOPE])
            .await
            .context("failed to obtain Gmail OAuth token")?;
        let bearer_token = token
            .token()
            .map(ToOwned::to_owned)
            .ok_or_else(|| anyhow!("gmail oauth did not return an access token"))?;

        Ok(Self {
            http: Client::new(),
            bearer_token,
            user_id,
        })
    }

    pub async fn list_messages(&self, query: &str) -> Result<Vec<GmailMessageRef>> {
        let response = self
            .authorized_get(&format!("{GMAIL_BASE_URL}/{}/messages", self.user_id), &[("q", query)])
            .await?;

        let payload = response
            .json::<GmailListResponse>()
            .await
            .context("failed to parse Gmail list response")?;

        Ok(payload.messages)
    }

    pub async fn get_message(&self, message_id: &str) -> Result<GmailMessage> {
        let response = self
            .authorized_get(
                &format!("{GMAIL_BASE_URL}/{}/messages/{}", self.user_id, message_id),
                &[("format", "full")],
            )
            .await?;

        response
            .json::<GmailMessage>()
            .await
            .context("failed to parse Gmail message response")
    }

    pub async fn archive_message(&self, message_id: &str) -> Result<()> {
        let response = self
            .http
            .post(format!(
                "{GMAIL_BASE_URL}/{}/messages/{}/modify",
                self.user_id, message_id
            ))
            .header(AUTHORIZATION, format!("Bearer {}", self.bearer_token))
            .header(CONTENT_TYPE, "application/json")
            .json(&GmailModifyRequest {
                remove_label_ids: &["INBOX"],
            })
            .send()
            .await
            .context("failed to modify Gmail labels")?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow!("gmail archive failed with status {}", response.status()))
        }
    }

    async fn authorized_get(&self, url: &str, query: &[(&str, &str)]) -> Result<reqwest::Response> {
        let response = self
            .http
            .get(url)
            .query(query)
            .header(AUTHORIZATION, format!("Bearer {}", self.bearer_token))
            .send()
            .await
            .with_context(|| format!("failed to GET {url}"))?;

        if response.status().is_success() {
            Ok(response)
        } else {
            Err(anyhow!("gmail request failed with status {}", response.status()))
        }
    }
}

pub fn extract_message_text(message: &GmailMessage) -> Result<String> {
    let payload = message
        .payload
        .as_ref()
        .ok_or_else(|| anyhow!("gmail message payload missing"))?;

    extract_payload_text(payload)
        .or_else(|| extract_payload_html(payload).map(strip_html))
        .ok_or_else(|| anyhow!("no readable email body found"))
}

pub fn header_value<'a>(headers: &'a [GmailHeader], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|header| header.name.eq_ignore_ascii_case(name))
        .map(|header| header.value.as_str())
}

fn extract_payload_text(payload: &GmailMessagePayload) -> Option<String> {
    for part in &payload.parts {
        if let Some(text) = extract_part_text(part) {
            return Some(text);
        }
    }

    payload
        .body
        .as_ref()
        .and_then(|body| body.data.as_deref())
        .and_then(decode_gmail_body)
}

fn extract_payload_html(payload: &GmailMessagePayload) -> Option<String> {
    for part in &payload.parts {
        if let Some(text) = extract_part_html(part) {
            return Some(text);
        }
    }

    None
}

fn extract_part_text(part: &GmailMessagePart) -> Option<String> {
    if matches!(part.mime_type.as_deref(), Some("text/plain")) {
        if let Some(text) = part
            .body
            .as_ref()
            .and_then(|body| body.data.as_deref())
            .and_then(decode_gmail_body)
        {
            return Some(text);
        }
    }

    for child in &part.parts {
        if let Some(text) = extract_part_text(child) {
            return Some(text);
        }
    }

    None
}

fn extract_part_html(part: &GmailMessagePart) -> Option<String> {
    if matches!(part.mime_type.as_deref(), Some("text/html")) {
        if let Some(text) = part
            .body
            .as_ref()
            .and_then(|body| body.data.as_deref())
            .and_then(decode_gmail_body)
        {
            return Some(text);
        }
    }

    for child in &part.parts {
        if let Some(text) = extract_part_html(child) {
            return Some(text);
        }
    }

    None
}

fn decode_gmail_body(data: &str) -> Option<String> {
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(data)
        .or_else(|_| base64::engine::general_purpose::URL_SAFE.decode(data))
        .ok()?;

    String::from_utf8(bytes).ok().map(|text| text.trim().to_string())
}

fn strip_html(html: String) -> String {
    let mut output = String::with_capacity(html.len());
    let mut in_tag = false;

    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => output.push(ch),
            _ => {}
        }
    }

    output
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .trim()
        .to_string()
}
