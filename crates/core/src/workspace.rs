#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentStatus {
    Inbox,
    Processing,
    Done,
    Review,
    Archive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceTransition {
    Claim,
    Complete,
    Escalate,
    Archive,
}

pub fn next_status(current: DocumentStatus, transition: WorkspaceTransition) -> Option<DocumentStatus> {
    match (current, transition) {
        (DocumentStatus::Inbox, WorkspaceTransition::Claim) => Some(DocumentStatus::Processing),
        (DocumentStatus::Processing, WorkspaceTransition::Complete) => Some(DocumentStatus::Done),
        (DocumentStatus::Processing, WorkspaceTransition::Escalate) => Some(DocumentStatus::Review),
        (DocumentStatus::Review, WorkspaceTransition::Archive) => Some(DocumentStatus::Archive),
        (DocumentStatus::Done, WorkspaceTransition::Archive) => Some(DocumentStatus::Archive),
        _ => None,
    }
}
