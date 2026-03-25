use crate::workspace::DocumentStatus;

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
}
