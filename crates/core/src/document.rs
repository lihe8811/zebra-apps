use crate::workspace::DocumentStatus;
use chrono::Utc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentFrontMatter {
    pub id: String,
    pub doc_type: String,
    pub status: DocumentStatus,
    pub source_app: String,
    pub target_app: String,
    pub created_at: String,
    pub updated_at: String,
    pub model: String,
    pub run_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowDocument {
    pub front_matter: DocumentFrontMatter,
    pub body: String,
}

impl WorkflowDocument {
    pub fn validate(&self) -> Result<(), String> {
        for (field, value) in [
            ("id", self.front_matter.id.as_str()),
            ("type", self.front_matter.doc_type.as_str()),
            ("source_app", self.front_matter.source_app.as_str()),
            ("target_app", self.front_matter.target_app.as_str()),
            ("created_at", self.front_matter.created_at.as_str()),
            ("updated_at", self.front_matter.updated_at.as_str()),
            ("model", self.front_matter.model.as_str()),
            ("run_id", self.front_matter.run_id.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(format!("missing required field: {field}"));
            }
        }

        Ok(())
    }

    pub fn summary_output(
        source_app: &str,
        message_id: &str,
        subject: &str,
        model: &str,
        summary: &str,
        source_body: &str,
    ) -> String {
        let timestamp = Utc::now().to_rfc3339();
        let escaped_subject = if subject.trim().is_empty() {
            "Untitled Email"
        } else {
            subject.trim()
        };

        format!(
            concat!(
                "---\n",
                "id: {message_id}\n",
                "type: gmail_summary\n",
                "status: done\n",
                "source_app: {source_app}\n",
                "target_app: none\n",
                "created_at: {timestamp}\n",
                "updated_at: {timestamp}\n",
                "model: {model}\n",
                "run_id: {message_id}\n",
                "---\n\n",
                "# {subject}\n\n",
                "## Summary\n\n",
                "{summary}\n\n",
                "## Source\n\n",
                "{source_body}\n"
            ),
            message_id = message_id,
            source_app = source_app,
            timestamp = timestamp,
            model = model,
            subject = escaped_subject,
            summary = summary.trim(),
            source_body = source_body.trim(),
        )
    }
}
