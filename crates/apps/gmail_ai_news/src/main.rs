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

    let sources = load_processing_sources(&config)?;
    println!(
        "{}",
        format_step_log(
            "config",
            &format!("loaded {} processing sources", sources.len())
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

    let mut processed_messages = 0usize;
    for source in &sources {
        println!(
            "{}",
            format_step_log(
                "gmail_query",
                &format!("source {} searching Gmail with query {}", source.name, source.gmail_query)
            )
        );
        let messages = gmail.list_messages(&source.gmail_query).await?;
        println!(
            "{}",
            format_step_log(
                "gmail_query",
                &format!(
                    "source {} found {} matching messages",
                    source.name,
                    messages.len()
                )
            )
        );

        let total = messages.len();
        for (index, message_ref) in messages.into_iter().enumerate() {
            println!(
                "{}",
                format_step_log(
                    "gmail_fetch",
                    &format!(
                        "source {} fetching message {} ({}/{})",
                        source.name,
                        message_ref.id,
                        index + 1,
                        total
                    )
                )
            );
            let message = gmail.get_message(&message_ref.id).await?;
            let body = extract_message_text(&message)?;
            let subject = message
                .payload
                .as_ref()
                .and_then(|payload| header_value(&payload.headers, "Subject"))
                .unwrap_or("Untitled Email");
            println!(
                "{}",
                format_message_progress(index + 1, total, &message.id, subject, &source.name)
            );
            println!(
                "{}",
                format_step_log(
                    "gmail_fetch",
                    &format!(
                        "source {} extracted body for {} ({} chars)",
                        source.name,
                        message.id,
                        body.chars().count()
                    )
                )
            );

            println!(
                "{}",
                format_step_log(
                    "summarize",
                    &format!("source {} requesting summary for {}", source.name, message.id)
                )
            );
            let summary = llm
                .execute(&ProviderRequest {
                    model: config.model.clone(),
                    instructions: source.prompt.clone(),
                    input: body.clone(),
                })
                .await?;
            println!(
                "{}",
                format_step_log(
                    "summarize",
                    &format!(
                        "source {} received summary for {} ({} chars)",
                        source.name,
                        message.id,
                        summary.content.chars().count()
                    )
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
            processed_messages += 1;
        }
    }

    println!(
        "{}",
        format_step_log("complete", &format!("processed {} messages", processed_messages))
    );

    Ok(())
}

struct ProcessingSource {
    name: String,
    gmail_query: String,
    prompt: String,
}

fn load_processing_sources(config: &GmailAiNewsConfig) -> Result<Vec<ProcessingSource>> {
    config
        .sources
        .iter()
        .map(|source| {
            let prompt = config.load_prompt(source)?;
            println!(
                "{}",
                format_step_log(
                    "prompt",
                    &format!(
                        "loaded source {} prompt from {} ({} chars)",
                        source.name,
                        source.prompt_file.display(),
                        prompt.chars().count()
                    )
                )
            );
            Ok(ProcessingSource {
                name: source.name.clone(),
                gmail_query: source.gmail_query.clone(),
                prompt,
            })
        })
        .collect()
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

fn format_message_progress(
    index: usize,
    total: usize,
    message_id: &str,
    subject: &str,
    source_name: &str,
) -> String {
    format!("[gmail_ai_news] source {source_name} message {index}/{total} ({message_id}): {subject}")
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
    use super::{
        format_message_progress, format_output_filename, format_step_log,
        load_processing_sources,
    };
    use std::path::PathBuf;
    use zebra_core::config::{GmailAiNewsConfig, GmailSourceConfig};

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
            format_message_progress(2, 5, "abc123", "Daily AI News", "ai_news"),
            "[gmail_ai_news] source ai_news message 2/5 (abc123): Daily AI News"
        );
    }

    #[test]
    fn formats_output_filename_from_received_date() {
        assert_eq!(
            format_output_filename("2026-04-08", "19d6a7dfd7177ba0"),
            "2026-04-08-19d6a7dfd7177ba0.md"
        );
    }

    #[test]
    fn loads_multiple_processing_sources_with_prompts() {
        let temp_dir = std::env::temp_dir().join(format!(
            "gmail-ai-news-source-test-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(temp_dir.join("config/prompts"))
            .expect("prompt dir should be created");
        std::fs::write(
            temp_dir.join("config/prompts/gmail_ai_news_summary.md"),
            "newsletter prompt",
        )
        .expect("newsletter prompt should be written");
        std::fs::write(
            temp_dir.join("config/prompts/gmail_podcast_transcript.md"),
            "transcript prompt",
        )
        .expect("transcript prompt should be written");

        let mut config = GmailAiNewsConfig {
            app_name: "gmail_ai_news".to_string(),
            workspace_root: "workspace".into(),
            done_dir: "workspace/done/gmail_ai_news".into(),
            sources: vec![
                GmailSourceConfig {
                    name: "ai_news".to_string(),
                    gmail_query: "in:inbox from:swyx+ainews@substack.com".to_string(),
                    prompt_file: PathBuf::from("config/prompts/gmail_ai_news_summary.md"),
                },
                GmailSourceConfig {
                    name: "podcast_transcript".to_string(),
                    gmail_query: "in:inbox from:swyx@substack.com".to_string(),
                    prompt_file: PathBuf::from("config/prompts/gmail_podcast_transcript.md"),
                },
            ],
            provider: "openai".to_string(),
            model: "gpt-5-mini".to_string(),
            oauth_client_secret_file: "config/gmail/oauth_client_secret.json".into(),
            oauth_token_cache_file: "config/gmail/oauth_tokens.json".into(),
            gmail_user_id: "me".to_string(),
        };
        config.resolve_relative_paths(&temp_dir);

        let sources = load_processing_sources(&config).expect("processing sources should load");
        std::fs::remove_dir_all(&temp_dir).expect("temp dir should be removed");

        assert_eq!(sources.len(), 2);
        assert_eq!(sources[0].name, "ai_news");
        assert_eq!(sources[0].prompt, "newsletter prompt");
        assert_eq!(sources[1].name, "podcast_transcript");
        assert_eq!(sources[1].prompt, "transcript prompt");
    }
}
