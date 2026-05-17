//! S391-S395: Approval Workflows
//!
//! Design review and approval chains for dental restorations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Approval status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
    RevisionRequested,
    Expired,
}

/// Approval request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub id: String,
    pub case_id: String,
    pub stage: String,
    pub requested_by: String,
    pub assigned_to: String,
    pub status: ApprovalStatus,
    pub created_at: DateTime<Utc>,
    pub decided_at: Option<DateTime<Utc>>,
    pub comments: String,
    pub attachments: Vec<String>,
}

impl ApprovalRequest {
    pub fn new(
        case_id: impl Into<String>,
        stage: impl Into<String>,
        requested_by: impl Into<String>,
        assigned_to: impl Into<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            case_id: case_id.into(),
            stage: stage.into(),
            requested_by: requested_by.into(),
            assigned_to: assigned_to.into(),
            status: ApprovalStatus::Pending,
            created_at: Utc::now(),
            decided_at: None,
            comments: String::new(),
            attachments: Vec::new(),
        }
    }

    pub fn approve(&mut self, comment: impl Into<String>) {
        self.status = ApprovalStatus::Approved;
        self.decided_at = Some(Utc::now());
        self.comments = comment.into();
    }

    pub fn reject(&mut self, reason: impl Into<String>) {
        self.status = ApprovalStatus::Rejected;
        self.decided_at = Some(Utc::now());
        self.comments = reason.into();
    }

    pub fn request_revision(&mut self, instructions: impl Into<String>) {
        self.status = ApprovalStatus::RevisionRequested;
        self.decided_at = Some(Utc::now());
        self.comments = instructions.into();
    }

    pub fn is_decided(&self) -> bool {
        self.status != ApprovalStatus::Pending
    }

    pub fn turnaround_hours(&self) -> Option<f64> {
        self.decided_at.map(|d| {
            d.signed_duration_since(self.created_at).num_minutes() as f64 / 60.0
        })
    }
}

/// Approval chain for multi-level review
#[derive(Debug, Clone, Default)]
pub struct ApprovalChain {
    pub requests: Vec<ApprovalRequest>,
}

impl ApprovalChain {
    pub fn new() -> Self { Self::default() }

    pub fn add(&mut self, req: ApprovalRequest) { self.requests.push(req); }

    pub fn pending(&self) -> Vec<&ApprovalRequest> {
        self.requests.iter().filter(|r| r.status == ApprovalStatus::Pending).collect()
    }

    pub fn for_case(&self, case_id: &str) -> Vec<&ApprovalRequest> {
        self.requests.iter().filter(|r| r.case_id == case_id).collect()
    }

    pub fn for_reviewer(&self, reviewer: &str) -> Vec<&ApprovalRequest> {
        self.requests.iter()
            .filter(|r| r.assigned_to == reviewer && r.status == ApprovalStatus::Pending)
            .collect()
    }

    pub fn approval_rate(&self) -> f64 {
        let decided: Vec<_> = self.requests.iter().filter(|r| r.is_decided()).collect();
        if decided.is_empty() { return 0.0; }
        let approved = decided.iter().filter(|r| r.status == ApprovalStatus::Approved).count();
        approved as f64 / decided.len() as f64 * 100.0
    }

    pub fn avg_turnaround_hours(&self) -> f64 {
        let times: Vec<f64> = self.requests.iter()
            .filter_map(|r| r.turnaround_hours())
            .collect();
        if times.is_empty() { 0.0 } else { times.iter().sum::<f64>() / times.len() as f64 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_approval_create() {
        let req = ApprovalRequest::new("C-001", "Design", "tech-1", "doc-1");
        assert_eq!(req.status, ApprovalStatus::Pending);
        assert!(!req.is_decided());
    }

    #[test]
    fn test_approval_approve() {
        let mut req = ApprovalRequest::new("C-001", "Design", "tech-1", "doc-1");
        req.approve("Looks great");
        assert_eq!(req.status, ApprovalStatus::Approved);
        assert!(req.is_decided());
        assert!(req.turnaround_hours().is_some());
    }

    #[test]
    fn test_approval_reject() {
        let mut req = ApprovalRequest::new("C-001", "Design", "tech-1", "doc-1");
        req.reject("Contacts too heavy");
        assert_eq!(req.status, ApprovalStatus::Rejected);
    }

    #[test]
    fn test_approval_chain() {
        let mut chain = ApprovalChain::new();
        let mut r1 = ApprovalRequest::new("C-1", "Design", "t1", "d1");
        r1.approve("OK");
        chain.add(r1);
        chain.add(ApprovalRequest::new("C-1", "QC", "t1", "qc1"));
        assert_eq!(chain.pending().len(), 1);
        assert_eq!(chain.for_case("C-1").len(), 2);
        assert!((chain.approval_rate() - 100.0).abs() < 0.1);
    }

    #[test]
    fn test_reviewer_queue() {
        let mut chain = ApprovalChain::new();
        chain.add(ApprovalRequest::new("C-1", "Design", "t1", "doc-1"));
        chain.add(ApprovalRequest::new("C-2", "Design", "t2", "doc-1"));
        chain.add(ApprovalRequest::new("C-3", "Design", "t3", "doc-2"));
        assert_eq!(chain.for_reviewer("doc-1").len(), 2);
    }
}
