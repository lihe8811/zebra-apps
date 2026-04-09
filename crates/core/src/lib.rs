pub mod config;
pub mod document;
pub mod gmail;
pub mod provider;
pub mod runlog;
pub mod workspace;

#[cfg(test)]
mod tests {
    use crate::config::{find_workspace_root, load_shared_env_file, GmailAiNewsConfig};
    use crate::document::{DocumentFrontMatter, WorkflowDocument};
    use crate::gmail::{
        extract_message_text, GmailHeader, GmailMessage, GmailMessageBody, GmailMessagePart,
        GmailMessagePayload,
    };
    use std::path::PathBuf;
    use crate::workspace::DocumentStatus;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn sample_document() -> WorkflowDocument {
        WorkflowDocument {
            front_matter: DocumentFrontMatter {
                id: "doc-1".to_string(),
                doc_type: "note".to_string(),
                status: DocumentStatus::Inbox,
                source_app: "capture".to_string(),
                target_app: "summarize".to_string(),
                created_at: "2026-03-25T00:00:00Z".to_string(),
                updated_at: "2026-03-25T00:00:00Z".to_string(),
                model: "gpt-placeholder".to_string(),
                run_id: "run-1".to_string(),
            },
            body: "# title".to_string(),
        }
    }

    #[test]
    fn validates_document_with_required_front_matter() {
        let document = sample_document();

        assert!(document.validate().is_ok());
    }

    #[test]
    fn rejects_document_with_missing_id() {
        let mut document = sample_document();
        document.front_matter.id.clear();

        let error = document.validate().expect_err("missing id should fail");

        assert_eq!(error, "missing required field: id");
    }

    #[test]
    fn extracts_plain_text_from_nested_gmail_parts() {
        let message = GmailMessage {
            id: "gmail-1".to_string(),
            thread_id: "thread-1".to_string(),
            internal_date: Some("1712536013000".to_string()),
            snippet: "snippet".to_string(),
            payload: Some(GmailMessagePayload {
                mime_type: Some("multipart/alternative".to_string()),
                filename: None,
                headers: vec![GmailHeader {
                    name: "Subject".to_string(),
                    value: "AI News".to_string(),
                }],
                body: None,
                parts: vec![
                    GmailMessagePart {
                        mime_type: Some("text/html".to_string()),
                        filename: None,
                        headers: Vec::new(),
                        body: Some(GmailMessageBody {
                            data: Some("PGRpdj5IVE1MIHZlcnNpb248L2Rpdj4".to_string()),
                        }),
                        parts: Vec::new(),
                    },
                    GmailMessagePart {
                        mime_type: Some("text/plain".to_string()),
                        filename: None,
                        headers: Vec::new(),
                        body: Some(GmailMessageBody {
                            data: Some("RnVsbCBuZXdzbGV0dGVyIHRleHQ".to_string()),
                        }),
                        parts: Vec::new(),
                    },
                ],
            }),
        };

        let text = extract_message_text(&message).expect("plain text should be extracted");

        assert_eq!(text, "Full newsletter text");
    }

    #[test]
    fn renders_summary_document_with_front_matter_and_sections() {
        let markdown = WorkflowDocument::summary_output(
            "gmail_ai_news",
            "message-123",
            "AI News Subject",
            "gpt-5-mini",
            "Summary body",
            "Original full email body",
        );

        assert!(markdown.contains("source_app: gmail_ai_news"));
        assert!(markdown.contains("status: done"));
        assert!(markdown.contains("# AI News Subject"));
        assert!(markdown.contains("## Summary"));
        assert!(markdown.contains("Summary body"));
        assert!(markdown.contains("## Source"));
        assert!(markdown.contains("Original full email body"));
    }

    #[test]
    fn loads_openai_api_key_from_shared_env_file() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("zebra_env_test_{unique}"));
        let env_path = dir.join(".env");
        let key_name = format!("ZEBRA_TEST_OPENAI_KEY_{unique}");

        fs::create_dir_all(&dir).expect("temp dir should be created");
        fs::write(&env_path, format!("{key_name}=loaded-from-dotenv\n"))
            .expect(".env file should be written");

        // This test owns a unique environment variable name, so mutating it is isolated.
        unsafe { std::env::remove_var(&key_name) };
        let loaded = load_shared_env_file(&env_path).expect(".env should load");

        assert!(loaded);
        assert_eq!(std::env::var(&key_name).as_deref(), Ok("loaded-from-dotenv"));
    }

    #[test]
    fn finds_workspace_root_from_nested_config_path() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("zebra_root_test_{unique}"));
        let nested = root.join("config/apps/gmail_ai_news.toml");

        fs::create_dir_all(nested.parent().expect("nested config parent"))
            .expect("nested dirs should be created");
        fs::write(root.join("Cargo.toml"), "[workspace]\n").expect("workspace manifest should exist");

        let found = find_workspace_root(&nested).expect("workspace root should be found");

        assert_eq!(found, root);
    }

    #[test]
    fn resolves_gmail_config_paths_relative_to_workspace_root() {
        let mut config = GmailAiNewsConfig {
            app_name: "gmail_ai_news".to_string(),
            workspace_root: "workspace".into(),
            done_dir: "workspace/done/gmail_ai_news".into(),
            gmail_query: "in:inbox".to_string(),
            prompt_file: "config/prompts/gmail_ai_news_summary.md".into(),
            provider: "openai".to_string(),
            model: "gpt-5-mini".to_string(),
            oauth_client_secret_file: "config/gmail/oauth_client_secret.json".into(),
            oauth_token_cache_file: "config/gmail/oauth_tokens.json".into(),
            gmail_user_id: "me".to_string(),
        };

        let root = PathBuf::from("/tmp/zebra-workspace-root");
        config.resolve_relative_paths(&root);

        assert_eq!(config.workspace_root, root.join("workspace"));
        assert_eq!(config.done_dir, root.join("workspace/done/gmail_ai_news"));
        assert_eq!(
            config.prompt_file,
            root.join("config/prompts/gmail_ai_news_summary.md")
        );
        assert_eq!(
            config.oauth_client_secret_file,
            root.join("config/gmail/oauth_client_secret.json")
        );
    }
}
