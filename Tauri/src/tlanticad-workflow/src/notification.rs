//! S366-S370: Notification System
//!
//! Alerts, reminders, and real-time notifications for workflow events.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Notification channel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NotifChannel {
    InApp,
    Email,
    Sms,
    Push,
    Webhook,
}

/// Notification priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NotifPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub channel: NotifChannel,
    pub priority: NotifPriority,
    pub recipient: String,
    pub title: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub read: bool,
    pub link: Option<String>,
}

impl Notification {
    pub fn new(
        channel: NotifChannel,
        recipient: impl Into<String>,
        title: impl Into<String>,
        body: impl Into<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            channel,
            priority: NotifPriority::Normal,
            recipient: recipient.into(),
            title: title.into(),
            body: body.into(),
            created_at: Utc::now(),
            read: false,
            link: None,
        }
    }

    pub fn with_priority(mut self, p: NotifPriority) -> Self { self.priority = p; self }
    pub fn with_link(mut self, link: impl Into<String>) -> Self { self.link = Some(link.into()); self }

    pub fn mark_read(&mut self) { self.read = true; }
}

/// Notification rule (trigger → notification)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifRule {
    pub name: String,
    pub trigger_event: String,
    pub channel: NotifChannel,
    pub recipient_role: String,
    pub template_title: String,
    pub template_body: String,
    pub enabled: bool,
}

/// Notification store
#[derive(Debug, Clone, Default)]
pub struct NotifStore {
    notifications: Vec<Notification>,
    rules: Vec<NotifRule>,
}

impl NotifStore {
    pub fn new() -> Self { Self::default() }

    pub fn add_rule(&mut self, rule: NotifRule) { self.rules.push(rule); }

    pub fn send(&mut self, notif: Notification) { self.notifications.push(notif); }

    pub fn unread_for(&self, recipient: &str) -> Vec<&Notification> {
        self.notifications.iter()
            .filter(|n| n.recipient == recipient && !n.read)
            .collect()
    }

    pub fn all_for(&self, recipient: &str) -> Vec<&Notification> {
        self.notifications.iter()
            .filter(|n| n.recipient == recipient)
            .collect()
    }

    pub fn mark_all_read(&mut self, recipient: &str) {
        for n in &mut self.notifications {
            if n.recipient == recipient { n.read = true; }
        }
    }

    pub fn total(&self) -> usize { self.notifications.len() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_create() {
        let n = Notification::new(NotifChannel::InApp, "user-1", "New Case", "Case C-001 assigned")
            .with_priority(NotifPriority::High)
            .with_link("/cases/C-001");
        assert!(!n.read);
        assert_eq!(n.priority, NotifPriority::High);
        assert!(n.link.is_some());
    }

    #[test]
    fn test_notif_store() {
        let mut store = NotifStore::new();
        store.send(Notification::new(NotifChannel::InApp, "u1", "T1", "B1"));
        store.send(Notification::new(NotifChannel::Email, "u2", "T2", "B2"));
        store.send(Notification::new(NotifChannel::InApp, "u1", "T3", "B3"));
        assert_eq!(store.total(), 3);
        assert_eq!(store.unread_for("u1").len(), 2);
    }

    #[test]
    fn test_mark_read() {
        let mut store = NotifStore::new();
        store.send(Notification::new(NotifChannel::InApp, "u1", "T", "B"));
        store.mark_all_read("u1");
        assert_eq!(store.unread_for("u1").len(), 0);
    }

    #[test]
    fn test_notif_rule() {
        let rule = NotifRule {
            name: "QC Fail Alert".into(),
            trigger_event: "qc_failed".into(),
            channel: NotifChannel::Push,
            recipient_role: "supervisor".into(),
            template_title: "QC Failed".into(),
            template_body: "Case {case_id} failed QC".into(),
            enabled: true,
        };
        assert!(rule.enabled);
    }
}
