use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use zebra_core::config::{find_workspace_root, load_shared_env_file, GmailAiNewsConfig};
use zebra_core::document::WorkflowDocument;
use zebra_core::gmail::{extract_message_text, header_value, GmailApiClient};
use zebra_core::provider::{OpenAiResponsesClient, ProviderRequest};

const DEFAULT_CONFIG_PATH: &str = "config/apps/gmail_ai_news.toml";

#[tokio::main]
async fn main() -> Result<()> {
    if matches!(env::args().nth(1).as_deref(), Some("--help") | Some("-h")) {
        print_help();
        return Ok(());
    }

    let config_path = env::var("GMAIL_AI_NEWS_CONFIG").unwrap_or_else(|_| DEFAULT_CONFIG_PATH.to_string());
    run(Path::new(&config_path)).await
}

async fn run(config_path: &Path) -> Result<()> {
    println!("{}", format_step_log("startup", "initializing gmail_ai_news run"));
    let resolved_config_path = resolve_config_path(config_path)?;
    println!(
        "{}",
        format_step_log(
            "config",
            &format!("using config {}", resolved_config_path.display())
        )
    );
    let workspace_root = find_workspace_root(&resolved_config_path).with_context(|| {
        format!(
            "failed to find workspace root from config path {}",
            resolved_config_path.display()
        )
    })?;
    let _ = load_shared_env_file(workspace_root.join(".env"))?;
    let mut config = GmailAiNewsConfig::load_from_file(&resolved_config_path)?;
    config.resolve_relative_paths(&workspace_root);
    println!(
        "{}",
        format_step_log(
            "config",
            &format!("resolved workspace root {}", workspace_root.display())
        )
    );
    fs::create_dir_all(&config.done_dir).with_context(|| {
        format!(
            "failed to create output directory {}",
            config.done_dir.as_path().display()
        )
    })?;
    println!(
        "{}",
        format_step_log(
            "output",
            &format!("writing summaries to {}", config.done_dir.display())
        )
    );

    let prompt = config.load_prompt()?;
    println!(
        "{}",
        format_step_log(
            "prompt",
            &format!(
                "loaded prompt from {} ({} chars)",
                config.prompt_file.display(),
                prompt.chars().count()
            )
        )
    );
    println!("{}", format_step_log("gmail_auth", "requesting Gmail access token"));
    let gmail = GmailApiClient::from_secret_file(
        &config.oauth_client_secret_file,
        &config.oauth_token_cache_file,
        config.gmail_user_id.clone(),
    )
    .await?;
    println!(
        "{}",
        format_step_log(
            "gmail_auth",
            &format!(
                "authenticated Gmail user {} with token cache {}",
                config.gmail_user_id,
                config.oauth_token_cache_file.display()
            )
        )
    );
    let llm = OpenAiResponsesClient::new();
    println!(
        "{}",
        format_step_log("openai", &format!("initialized OpenAI client with model {}", config.model))
    );

    println!(
        "{}",
        format_step_log("gmail_query", &format!("searching Gmail with query {}", config.gmail_query))
    );
    let messages = gmail.list_messages(&config.gmail_query).await?;
    println!(
        "{}",
        format_step_log(
            "gmail_query",
            &format!("found {} matching messages", messages.len())
        )
    );

    if messages.is_empty() {
        println!(
            "{}",
            format_step_log("complete", "no matching messages to process")
        );
        return Ok(());
    }

    let total = messages.len();
    for (index, message_ref) in messages.into_iter().enumerate() {
        println!(
            "{}",
            format_step_log(
                "gmail_fetch",
                &format!("fetching message {} ({}/{})", message_ref.id, index + 1, total)
            )
        );
        let message = gmail.get_message(&message_ref.id).await?;
        let body = extract_message_text(&message)?;
        let subject = message
            .payload
            .as_ref()
            .and_then(|payload| header_value(&payload.headers, "Subject"))
            .unwrap_or("Untitled Email");
        println!("{}", format_message_progress(index + 1, total, &message.id, subject));
        println!(
            "{}",
            format_step_log(
                "gmail_fetch",
                &format!("extracted body for {} ({} chars)", message.id, body.chars().count())
            )
        );

        println!(
            "{}",
            format_step_log("summarize", &format!("requesting summary for {}", message.id))
        );
        let summary = llm
            .execute(&ProviderRequest {
                model: config.model.clone(),
                instructions: prompt.clone(),
                input: body.clone(),
            })
            .await?;
        println!(
            "{}",
            format_step_log(
                "summarize",
                &format!("received summary for {} ({} chars)", message.id, summary.content.chars().count())
            )
        );

        let output = WorkflowDocument::summary_output(
            &config.app_name,
            &message.id,
            subject,
            &config.model,
            &summary.content,
            &body,
        );
        let output_path = output_path(&config.done_dir, &message)?;
        println!(
            "{}",
            format_step_log("write", &format!("writing {}", output_path.display()))
        );
        fs::write(&output_path, output)
            .with_context(|| format!("failed to write {}", output_path.display()))?;
        println!(
            "{}",
            format_step_log("archive", &format!("archiving Gmail message {}", message.id))
        );
        gmail.archive_message(&message.id).await?;
        println!(
            "{}",
            format_step_log("archive", &format!("archived Gmail message {}", message.id))
        );
    }

    println!(
        "{}",
        format_step_log("complete", &format!("processed {} messages", total))
    );

    Ok(())
}

fn output_path(done_dir: &Path, message: &zebra_core::gmail::GmailMessage) -> Result<PathBuf> {
    let received_at = received_at(message)?;
    Ok(done_dir.join(format_output_filename(&received_at, &message.id)))
}

fn resolve_config_path(config_path: &Path) -> Result<PathBuf> {
    if config_path.is_absolute() {
        return Ok(config_path.to_path_buf());
    }

    if config_path.exists() {
        return Ok(config_path.to_path_buf());
    }

    if let Some(workspace_root) = find_workspace_root(std::env::current_dir()?) {
        let candidate = workspace_root.join(config_path);
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    Ok(config_path.to_path_buf())
}

fn print_help() {
    println!("gmail_ai_news");
    println!("Reads Gmail inbox mail from a configured sender, summarizes it, writes markdown output, and archives the message.");
    println!("Set GMAIL_AI_NEWS_CONFIG to override the default config path.");
}

fn format_step_log(step: &str, message: &str) -> String {
    format!("[gmail_ai_news] {step}: {message}")
}

fn format_message_progress(index: usize, total: usize, message_id: &str, subject: &str) -> String {
    format!("[gmail_ai_news] message {index}/{total} ({message_id}): {subject}")
}

fn format_output_filename(received_at: &str, message_id: &str) -> String {
    let date = received_at.split('T').next().unwrap_or(received_at);
    format!("{date}-{message_id}.md")
}

fn received_at(message: &zebra_core::gmail::GmailMessage) -> Result<String> {
    let internal_date = message
        .internal_date
        .as_deref()
        .context("gmail message missing internalDate")?;
    let millis = internal_date
        .parse::<i64>()
        .with_context(|| format!("invalid gmail internalDate {internal_date}"))?;
    let date_time: DateTime<Utc> = DateTime::from_timestamp_millis(millis)
        .context("gmail internalDate was out of range")?;
    Ok(date_time.date_naive().format("%Y-%m-%d").to_string())
}

#[cfg(test)]
mod tests {
    use super::{format_message_progress, format_output_filename, format_step_log};

    #[test]
    fn formats_step_logs_with_app_prefix() {
        assert_eq!(
            format_step_log("gmail_query", "found 0 matching messages"),
            "[gmail_ai_news] gmail_query: found 0 matching messages"
        );
    }

    #[test]
    fn formats_message_progress_with_subject() {
        assert_eq!(
            format_message_progress(2, 5, "abc123", "Daily AI News"),
            "[gmail_ai_news] message 2/5 (abc123): Daily AI News"
        );
    }

    #[test]
    fn formats_output_filename_from_received_date() {
        assert_eq!(
            format_output_filename("2026-04-08", "19d6a7dfd7177ba0"),
            "2026-04-08-19d6a7dfd7177ba0.md"
        );
    }
}
