//! S351-S355: Dental Case Management
//!
//! Patient cases, restoration tracking, and status management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Case status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CaseStatus {
    Created,
    ScanReceived,
    DesignInProgress,
    DesignApproved,
    Manufacturing,
    QualityCheck,
    Shipped,
    Delivered,
    Completed,
    OnHold,
    Cancelled,
}

impl CaseStatus {
    pub fn is_active(&self) -> bool {
        !matches!(self, Self::Completed | Self::Cancelled)
    }

    pub fn can_transition_to(&self, next: CaseStatus) -> bool {
        use CaseStatus::*;
        matches!(
            (self, next),
            (Created, ScanReceived) |
            (ScanReceived, DesignInProgress) |
            (DesignInProgress, DesignApproved) | (DesignInProgress, OnHold) |
            (DesignApproved, Manufacturing) |
            (Manufacturing, QualityCheck) |
            (QualityCheck, Shipped) | (QualityCheck, Manufacturing) |
            (Shipped, Delivered) |
            (Delivered, Completed) |
            (OnHold, DesignInProgress) |
            (_, Cancelled)
        )
    }
}

/// Restoration type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RestorationType {
    SingleCrown,
    Bridge,
    Inlay,
    Onlay,
    Veneer,
    ImplantAbutment,
    ImplantCrown,
    FullArchFixed,
    FullArchRemovable,
    PartialDenture,
    CompleteDenture,
    SurgicalGuide,
    OcclusalSplint,
    Temporary,
}

/// Single restoration item in a case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseItem {
    pub id: String,
    pub restoration_type: RestorationType,
    pub tooth_positions: Vec<u8>,
    pub material: String,
    pub shade: Option<String>,
    pub notes: String,
}

/// Dental case (work order)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DentalCase {
    pub id: String,
    pub patient_name: String,
    pub dentist: String,
    pub lab: Option<String>,
    pub status: CaseStatus,
    pub items: Vec<CaseItem>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub due_date: Option<DateTime<Utc>>,
    pub priority: CasePriority,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CasePriority { Urgent, High, Normal, Low }

impl DentalCase {
    pub fn new(patient: impl Into<String>, dentist: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            patient_name: patient.into(),
            dentist: dentist.into(),
            lab: None,
            status: CaseStatus::Created,
            items: Vec::new(),
            created_at: now,
            updated_at: now,
            due_date: None,
            priority: CasePriority::Normal,
            tags: Vec::new(),
        }
    }

    pub fn add_item(&mut self, item: CaseItem) {
        self.items.push(item);
        self.updated_at = Utc::now();
    }

    pub fn transition(&mut self, new_status: CaseStatus) -> Result<(), String> {
        if self.status.can_transition_to(new_status) {
            self.status = new_status;
            self.updated_at = Utc::now();
            Ok(())
        } else {
            Err(format!("Cannot transition from {:?} to {:?}", self.status, new_status))
        }
    }

    pub fn total_teeth(&self) -> usize {
        self.items.iter().map(|i| i.tooth_positions.len()).sum()
    }

    pub fn is_overdue(&self) -> bool {
        self.due_date.map_or(false, |d| d < Utc::now() && self.status.is_active())
    }
}

/// Case manager (in-memory store)
#[derive(Debug, Clone, Default)]
pub struct CaseManager {
    pub cases: Vec<DentalCase>,
}

impl CaseManager {
    pub fn new() -> Self { Self { cases: Vec::new() } }

    pub fn create_case(&mut self, patient: impl Into<String>, dentist: impl Into<String>) -> &DentalCase {
        let case = DentalCase::new(patient, dentist);
        self.cases.push(case);
        self.cases.last().unwrap()
    }

    pub fn find_by_id(&self, id: &str) -> Option<&DentalCase> {
        self.cases.iter().find(|c| c.id == id)
    }

    pub fn active_cases(&self) -> Vec<&DentalCase> {
        self.cases.iter().filter(|c| c.status.is_active()).collect()
    }

    pub fn cases_by_status(&self, status: CaseStatus) -> Vec<&DentalCase> {
        self.cases.iter().filter(|c| c.status == status).collect()
    }

    pub fn total(&self) -> usize { self.cases.len() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_creation() {
        let case = DentalCase::new("John Doe", "Dr. Smith");
        assert_eq!(case.status, CaseStatus::Created);
        assert!(case.status.is_active());
    }

    #[test]
    fn test_case_transition() {
        let mut case = DentalCase::new("Jane", "Dr. A");
        assert!(case.transition(CaseStatus::ScanReceived).is_ok());
        assert_eq!(case.status, CaseStatus::ScanReceived);
        // Invalid transition
        assert!(case.transition(CaseStatus::Shipped).is_err());
    }

    #[test]
    fn test_case_cancel() {
        let mut case = DentalCase::new("P", "D");
        case.transition(CaseStatus::ScanReceived).unwrap();
        // Can always cancel
        assert!(case.transition(CaseStatus::Cancelled).is_ok());
        assert!(!case.status.is_active());
    }

    #[test]
    fn test_case_items() {
        let mut case = DentalCase::new("P", "D");
        case.add_item(CaseItem {
            id: "i1".into(),
            restoration_type: RestorationType::SingleCrown,
            tooth_positions: vec![14, 15],
            material: "Zirconia".into(),
            shade: Some("A2".into()),
            notes: String::new(),
        });
        assert_eq!(case.total_teeth(), 2);
    }

    #[test]
    fn test_case_manager() {
        let mut mgr = CaseManager::new();
        mgr.create_case("P1", "D1");
        mgr.create_case("P2", "D2");
        assert_eq!(mgr.total(), 2);
        assert_eq!(mgr.active_cases().len(), 2);
    }

    #[test]
    fn test_priority() {
        let mut case = DentalCase::new("P", "D");
        case.priority = CasePriority::Urgent;
        assert_eq!(case.priority, CasePriority::Urgent);
    }
}
