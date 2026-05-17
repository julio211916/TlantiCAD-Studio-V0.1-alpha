//! Wax-up workflow state machine

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// State machine states for a wax-up session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WaxupState {
    NotStarted,
    ScanLoaded,
    AnatomySelected,
    Roughed,
    Detailed,
    Finalized,
}

/// A digital wax-up session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaxupSession {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub tooth_numbers: Vec<u8>,
    pub state: WaxupState,
    pub created_at: DateTime<Utc>,
    pub notes: String,
}

impl WaxupSession {
    /// Create a new wax-up session for the given patient and teeth
    pub fn new(patient_id: Uuid, teeth: Vec<u8>) -> Self {
        Self {
            id: Uuid::new_v4(),
            patient_id,
            tooth_numbers: teeth,
            state: WaxupState::NotStarted,
            created_at: Utc::now(),
            notes: String::new(),
        }
    }

    /// Advance to the next state in the workflow sequence.
    ///
    /// Returns `false` if already in the final state.
    pub fn advance_state(&mut self) -> bool {
        self.state = match self.state {
            WaxupState::NotStarted => WaxupState::ScanLoaded,
            WaxupState::ScanLoaded => WaxupState::AnatomySelected,
            WaxupState::AnatomySelected => WaxupState::Roughed,
            WaxupState::Roughed => WaxupState::Detailed,
            WaxupState::Detailed => WaxupState::Finalized,
            WaxupState::Finalized => return false,
        };
        true
    }

    /// Returns whether the session is in a state where export is permitted
    pub fn can_export(&self) -> bool {
        matches!(self.state, WaxupState::Detailed | WaxupState::Finalized)
    }
}
