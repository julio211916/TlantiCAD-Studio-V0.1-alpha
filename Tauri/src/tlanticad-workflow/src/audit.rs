//! S361-S365: Audit Trail & Change Tracking
//!
//! Record all changes to cases, designs, and manufacturing data.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Audit event type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuditAction {
    CaseCreated,
    CaseUpdated,
    StatusChanged,
    DesignModified,
    DesignApproved,
    DesignRejected,
    FileUploaded,
    FileDeleted,
    ManufacturingStarted,
    ManufacturingCompleted,
    QcPassed,
    QcFailed,
    Shipped,
    CommentAdded,
    UserLogin,
    UserLogout,
}

/// Single audit entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub action: AuditAction,
    pub entity_type: String,
    pub entity_id: String,
    pub user_id: String,
    pub details: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}

impl AuditEntry {
    pub fn new(
        action: AuditAction,
        entity_type: impl Into<String>,
        entity_id: impl Into<String>,
        user_id: impl Into<String>,
        details: impl Into<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            action,
            entity_type: entity_type.into(),
            entity_id: entity_id.into(),
            user_id: user_id.into(),
            details: details.into(),
            old_value: None,
            new_value: None,
        }
    }

    pub fn with_change(mut self, old: impl Into<String>, new: impl Into<String>) -> Self {
        self.old_value = Some(old.into());
        self.new_value = Some(new.into());
        self
    }
}

/// Audit log store
#[derive(Debug, Clone, Default)]
pub struct AuditLog {
    entries: Vec<AuditEntry>,
}

impl AuditLog {
    pub fn new() -> Self { Self { entries: Vec::new() } }

    pub fn record(&mut self, entry: AuditEntry) {
        self.entries.push(entry);
    }

    pub fn entries_for_entity(&self, entity_id: &str) -> Vec<&AuditEntry> {
        self.entries.iter().filter(|e| e.entity_id == entity_id).collect()
    }

    pub fn entries_by_user(&self, user_id: &str) -> Vec<&AuditEntry> {
        self.entries.iter().filter(|e| e.user_id == user_id).collect()
    }

    pub fn entries_by_action(&self, action: AuditAction) -> Vec<&AuditEntry> {
        self.entries.iter().filter(|e| e.action == action).collect()
    }

    pub fn recent(&self, count: usize) -> Vec<&AuditEntry> {
        self.entries.iter().rev().take(count).collect()
    }

    pub fn total(&self) -> usize { self.entries.len() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_entry() {
        let entry = AuditEntry::new(
            AuditAction::CaseCreated, "Case", "C-001", "U-001", "New case created",
        );
        assert_eq!(entry.action, AuditAction::CaseCreated);
    }

    #[test]
    fn test_audit_with_change() {
        let entry = AuditEntry::new(
            AuditAction::StatusChanged, "Case", "C-001", "U-001", "Status update",
        ).with_change("Created", "ScanReceived");
        assert_eq!(entry.old_value.as_deref(), Some("Created"));
        assert_eq!(entry.new_value.as_deref(), Some("ScanReceived"));
    }

    #[test]
    fn test_audit_log_queries() {
        let mut log = AuditLog::new();
        log.record(AuditEntry::new(AuditAction::CaseCreated, "Case", "C-1", "U-1", ""));
        log.record(AuditEntry::new(AuditAction::DesignModified, "Design", "D-1", "U-2", ""));
        log.record(AuditEntry::new(AuditAction::CaseUpdated, "Case", "C-1", "U-1", ""));

        assert_eq!(log.total(), 3);
        assert_eq!(log.entries_for_entity("C-1").len(), 2);
        assert_eq!(log.entries_by_user("U-2").len(), 1);
        assert_eq!(log.entries_by_action(AuditAction::CaseCreated).len(), 1);
    }

    #[test]
    fn test_audit_recent() {
        let mut log = AuditLog::new();
        for i in 0..5 {
            log.record(AuditEntry::new(
                AuditAction::CommentAdded, "Case", &format!("C-{}", i), "U-1", "",
            ));
        }
        assert_eq!(log.recent(3).len(), 3);
    }
}
