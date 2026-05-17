//! S356-S360: Workflow Pipeline & State Machine
//!
//! Configurable multi-stage pipelines for dental lab workflows.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Pipeline stage definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageDefinition {
    pub name: String,
    pub required: bool,
    pub auto_advance: bool,
    pub timeout_hours: Option<f64>,
    pub assignee_role: Option<String>,
}

/// Pipeline definition (template)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineDefinition {
    pub name: String,
    pub stages: Vec<StageDefinition>,
}

impl PipelineDefinition {
    pub fn crown_workflow() -> Self {
        Self {
            name: "Crown Manufacturing".into(),
            stages: vec![
                StageDefinition { name: "Scan Import".into(), required: true, auto_advance: true, timeout_hours: None, assignee_role: Some("technician".into()) },
                StageDefinition { name: "Design".into(), required: true, auto_advance: false, timeout_hours: Some(24.0), assignee_role: Some("designer".into()) },
                StageDefinition { name: "Design Review".into(), required: true, auto_advance: false, timeout_hours: Some(4.0), assignee_role: Some("reviewer".into()) },
                StageDefinition { name: "CAM Prep".into(), required: true, auto_advance: true, timeout_hours: None, assignee_role: Some("cam_operator".into()) },
                StageDefinition { name: "Manufacturing".into(), required: true, auto_advance: false, timeout_hours: Some(8.0), assignee_role: Some("machinist".into()) },
                StageDefinition { name: "Post-Processing".into(), required: true, auto_advance: false, timeout_hours: Some(12.0), assignee_role: Some("technician".into()) },
                StageDefinition { name: "QC".into(), required: true, auto_advance: false, timeout_hours: Some(2.0), assignee_role: Some("qc_inspector".into()) },
                StageDefinition { name: "Shipping".into(), required: true, auto_advance: false, timeout_hours: Some(1.0), assignee_role: Some("shipping".into()) },
            ],
        }
    }

    pub fn surgical_guide_workflow() -> Self {
        Self {
            name: "Surgical Guide".into(),
            stages: vec![
                StageDefinition { name: "CBCT Import".into(), required: true, auto_advance: true, timeout_hours: None, assignee_role: None },
                StageDefinition { name: "Implant Planning".into(), required: true, auto_advance: false, timeout_hours: Some(48.0), assignee_role: Some("surgeon".into()) },
                StageDefinition { name: "Guide Design".into(), required: true, auto_advance: false, timeout_hours: Some(24.0), assignee_role: Some("designer".into()) },
                StageDefinition { name: "3D Printing".into(), required: true, auto_advance: false, timeout_hours: Some(4.0), assignee_role: Some("printer_op".into()) },
                StageDefinition { name: "QC".into(), required: true, auto_advance: false, timeout_hours: Some(1.0), assignee_role: Some("qc_inspector".into()) },
            ],
        }
    }

    pub fn stage_count(&self) -> usize { self.stages.len() }
}

/// Instance of a pipeline for a specific case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineInstance {
    pub id: String,
    pub case_id: String,
    pub definition_name: String,
    pub current_stage: usize,
    pub total_stages: usize,
    pub stage_history: Vec<StageRecord>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageRecord {
    pub stage_name: String,
    pub entered_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub assignee: Option<String>,
    pub notes: String,
}

impl PipelineInstance {
    pub fn start(case_id: impl Into<String>, definition: &PipelineDefinition) -> Self {
        let now = Utc::now();
        let first_stage = definition.stages.first()
            .map(|s| s.name.clone())
            .unwrap_or_default();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            case_id: case_id.into(),
            definition_name: definition.name.clone(),
            current_stage: 0,
            total_stages: definition.stages.len(),
            stage_history: vec![StageRecord {
                stage_name: first_stage,
                entered_at: now,
                completed_at: None,
                assignee: None,
                notes: String::new(),
            }],
            started_at: now,
            completed_at: None,
        }
    }

    pub fn advance(&mut self, stage_names: &[String]) -> bool {
        if self.current_stage + 1 >= self.total_stages {
            self.completed_at = Some(Utc::now());
            return false;
        }
        // Close current stage
        if let Some(record) = self.stage_history.last_mut() {
            record.completed_at = Some(Utc::now());
        }
        self.current_stage += 1;
        let name = stage_names.get(self.current_stage)
            .cloned()
            .unwrap_or_else(|| format!("Stage {}", self.current_stage));
        self.stage_history.push(StageRecord {
            stage_name: name,
            entered_at: Utc::now(),
            completed_at: None,
            assignee: None,
            notes: String::new(),
        });
        true
    }

    pub fn is_complete(&self) -> bool {
        self.completed_at.is_some()
    }

    pub fn progress_pct(&self) -> f64 {
        if self.total_stages == 0 { return 0.0; }
        (self.current_stage as f64 / self.total_stages as f64 * 100.0).min(100.0)
    }

    pub fn current_stage_name(&self) -> &str {
        self.stage_history.last()
            .map(|r| r.stage_name.as_str())
            .unwrap_or("Unknown")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crown_pipeline_create() {
        let def = PipelineDefinition::crown_workflow();
        assert_eq!(def.stage_count(), 8);
    }

    #[test]
    fn test_pipeline_instance() {
        let def = PipelineDefinition::crown_workflow();
        let inst = PipelineInstance::start("case-1", &def);
        assert_eq!(inst.current_stage, 0);
        assert!(!inst.is_complete());
        assert!(inst.progress_pct() < 1.0);
    }

    #[test]
    fn test_pipeline_advance() {
        let def = PipelineDefinition::crown_workflow();
        let names: Vec<String> = def.stages.iter().map(|s| s.name.clone()).collect();
        let mut inst = PipelineInstance::start("case-1", &def);
        assert!(inst.advance(&names));
        assert_eq!(inst.current_stage, 1);
        assert_eq!(inst.current_stage_name(), "Design");
    }

    #[test]
    fn test_pipeline_complete() {
        let def = PipelineDefinition::surgical_guide_workflow();
        let names: Vec<String> = def.stages.iter().map(|s| s.name.clone()).collect();
        let mut inst = PipelineInstance::start("case-2", &def);
        for _ in 0..10 { inst.advance(&names); }
        assert!(inst.is_complete() || inst.current_stage == def.stage_count() - 1);
    }

    #[test]
    fn test_stage_history() {
        let def = PipelineDefinition::crown_workflow();
        let names: Vec<String> = def.stages.iter().map(|s| s.name.clone()).collect();
        let mut inst = PipelineInstance::start("c", &def);
        inst.advance(&names);
        inst.advance(&names);
        assert_eq!(inst.stage_history.len(), 3);
    }
}
