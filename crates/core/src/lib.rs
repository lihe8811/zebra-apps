pub mod config;
pub mod document;
pub mod provider;
pub mod runlog;
pub mod workspace;

#[cfg(test)]
mod tests {
    use crate::document::{DocumentFrontMatter, WorkflowDocument};
    use crate::workspace::DocumentStatus;

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
}
