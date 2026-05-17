//! S381-S385: Lab–Dentist Communication
//!
//! Messaging, file exchange, and prescription protocols between dental labs and clinics.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Participant role in communication
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ParticipantRole { Dentist, LabTechnician, LabManager, Surgeon }

/// Message attachment kind
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub filename: String,
    pub mime_type: String,
    pub size_bytes: u64,
}

/// A single message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabMessage {
    pub id: String,
    pub case_id: String,
    pub sender_id: String,
    pub sender_role: ParticipantRole,
    pub body: String,
    pub attachments: Vec<Attachment>,
    pub sent_at: DateTime<Utc>,
    pub read: bool,
}

impl LabMessage {
    pub fn new(
        case_id: impl Into<String>,
        sender_id: impl Into<String>,
        role: ParticipantRole,
        body: impl Into<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            case_id: case_id.into(),
            sender_id: sender_id.into(),
            sender_role: role,
            body: body.into(),
            attachments: Vec::new(),
            sent_at: Utc::now(),
            read: false,
        }
    }

    pub fn attach(&mut self, filename: impl Into<String>, mime: impl Into<String>, size: u64) {
        self.attachments.push(Attachment { filename: filename.into(), mime_type: mime.into(), size_bytes: size });
    }
}

/// Digital dental prescription
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigitalRx {
    pub case_id: String,
    pub patient_name: String,
    pub doctor_name: String,
    pub tooth_numbers: Vec<u8>,
    pub restoration_type: String,
    pub material_preference: String,
    pub shade: String,
    pub special_instructions: String,
    pub created_at: DateTime<Utc>,
}

impl DigitalRx {
    pub fn summary(&self) -> String {
        format!(
            "Rx for {} – Teeth {:?} – {} in {} shade {}",
            self.patient_name, self.tooth_numbers,
            self.restoration_type, self.material_preference, self.shade,
        )
    }
}

/// Case communication thread
#[derive(Debug, Clone, Default)]
pub struct CaseThread {
    pub messages: Vec<LabMessage>,
}

impl CaseThread {
    pub fn new() -> Self { Self::default() }

    pub fn add(&mut self, msg: LabMessage) { self.messages.push(msg); }

    pub fn unread(&self) -> Vec<&LabMessage> {
        self.messages.iter().filter(|m| !m.read).collect()
    }

    pub fn by_role(&self, role: ParticipantRole) -> Vec<&LabMessage> {
        self.messages.iter().filter(|m| m.sender_role == role).collect()
    }

    pub fn mark_all_read(&mut self) {
        for m in &mut self.messages { m.read = true; }
    }

    pub fn len(&self) -> usize { self.messages.len() }
    pub fn is_empty(&self) -> bool { self.messages.is_empty() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lab_message() {
        let mut msg = LabMessage::new(
            "C-001", "doc-1", ParticipantRole::Dentist,
            "Please adjust the occlusal contacts on #19",
        );
        msg.attach("scan.stl", "application/sla", 2_500_000);
        assert_eq!(msg.attachments.len(), 1);
    }

    #[test]
    fn test_digital_rx() {
        let rx = DigitalRx {
            case_id: "C-001".into(),
            patient_name: "Patient A".into(),
            doctor_name: "Dr. Smith".into(),
            tooth_numbers: vec![14, 15],
            restoration_type: "Crown".into(),
            material_preference: "Zirconia".into(),
            shade: "A2".into(),
            special_instructions: "High-translucency for anteriors".into(),
            created_at: Utc::now(),
        };
        assert!(rx.summary().contains("Crown"));
    }

    #[test]
    fn test_case_thread() {
        let mut thread = CaseThread::new();
        thread.add(LabMessage::new("C-1", "doc-1", ParticipantRole::Dentist, "Hi"));
        thread.add(LabMessage::new("C-1", "tech-1", ParticipantRole::LabTechnician, "Hello"));
        assert_eq!(thread.len(), 2);
        assert_eq!(thread.unread().len(), 2);
        assert_eq!(thread.by_role(ParticipantRole::Dentist).len(), 1);
        thread.mark_all_read();
        assert_eq!(thread.unread().len(), 0);
    }
}
