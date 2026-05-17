//! S191-S195: Motor Automation — workflow orchestration and batch processing.
//!
//! Multi-case batch pipelines, template-based design automation,
//! parameter presets, and quality gate enforcement.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Workflow step identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkflowStep {
    Import,
    Segmentation,
    MarginDetection,
    InsertionAxis,
    DesignGeneration,
    OcclusionCheck,
    ManufacturingValidation,
    Export,
    Custom(String),
}

/// Quality gate — a pass/fail check at a workflow stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGate {
    pub step: WorkflowStep,
    pub name: String,
    pub passed: bool,
    pub message: String,
}

/// A complete workflow template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTemplate {
    pub name: String,
    pub steps: Vec<WorkflowStep>,
    pub parameter_overrides: HashMap<String, String>,
    pub quality_gates: Vec<String>,
}

/// Parameter preset for design automation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignPreset {
    pub name: String,
    pub category: String,
    pub parameters: HashMap<String, f64>,
    pub description: String,
}

/// Result of a single batch case.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCaseResult {
    pub case_id: String,
    pub status: CaseStatus,
    pub gates: Vec<QualityGate>,
    pub elapsed_ms: u64,
    pub errors: Vec<String>,
}

/// Status of a batch case.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CaseStatus {
    Pending,
    Running,
    Completed,
    Failed,
    GateFailed,
}

/// Batch processing result.
#[derive(Debug, Clone)]
pub struct BatchResult {
    pub cases: Vec<BatchCaseResult>,
    pub total_ms: u64,
    pub success_count: usize,
    pub failure_count: usize,
}

// ---------------------------------------------------------------------------
// Workflow definition (S191-S192)
// ---------------------------------------------------------------------------

/// Create a standard crown workflow template.
pub fn crown_workflow() -> WorkflowTemplate {
    WorkflowTemplate {
        name: "Standard Crown".into(),
        steps: vec![
            WorkflowStep::Import,
            WorkflowStep::Segmentation,
            WorkflowStep::MarginDetection,
            WorkflowStep::InsertionAxis,
            WorkflowStep::DesignGeneration,
            WorkflowStep::OcclusionCheck,
            WorkflowStep::ManufacturingValidation,
            WorkflowStep::Export,
        ],
        parameter_overrides: HashMap::new(),
        quality_gates: vec![
            "margin_gap < 100".into(),
            "min_thickness > 0.5".into(),
            "occlusal_clearance > 0.3".into(),
        ],
    }
}

/// Create a bridge workflow template.
pub fn bridge_workflow() -> WorkflowTemplate {
    WorkflowTemplate {
        name: "Bridge Design".into(),
        steps: vec![
            WorkflowStep::Import,
            WorkflowStep::Segmentation,
            WorkflowStep::MarginDetection,
            WorkflowStep::InsertionAxis,
            WorkflowStep::DesignGeneration,
            WorkflowStep::Custom("connector_sizing".into()),
            WorkflowStep::Custom("stress_analysis".into()),
            WorkflowStep::ManufacturingValidation,
            WorkflowStep::Export,
        ],
        parameter_overrides: HashMap::new(),
        quality_gates: vec![
            "connector_area > 6.0".into(),
            "span_deflection < 0.5".into(),
        ],
    }
}

// ---------------------------------------------------------------------------
// Design presets (S193)
// ---------------------------------------------------------------------------

/// Standard set of design parameter presets.
pub fn standard_presets() -> Vec<DesignPreset> {
    vec![
        DesignPreset {
            name: "Standard Crown".into(),
            category: "crown".into(),
            parameters: {
                let mut m = HashMap::new();
                m.insert("cement_gap".into(), 0.04);
                m.insert("extra_gap".into(), 0.02);
                m.insert("occlusal_thickness".into(), 1.5);
                m.insert("margin_thickness".into(), 0.5);
                m
            },
            description: "Default crown parameters for zirconia".into(),
        },
        DesignPreset {
            name: "Thin Veneer".into(),
            category: "veneer".into(),
            parameters: {
                let mut m = HashMap::new();
                m.insert("thickness".into(), 0.5);
                m.insert("cement_gap".into(), 0.03);
                m
            },
            description: "Minimal feldspathic veneer".into(),
        },
        DesignPreset {
            name: "Night Guard".into(),
            category: "splint".into(),
            parameters: {
                let mut m = HashMap::new();
                m.insert("occlusal_thickness".into(), 2.0);
                m.insert("wall_thickness".into(), 1.5);
                m
            },
            description: "Standard hard acrylic night guard".into(),
        },
    ]
}

/// Look up a preset by name.
pub fn find_preset(name: &str) -> Option<DesignPreset> {
    standard_presets().into_iter().find(|p| p.name.eq_ignore_ascii_case(name))
}

// ---------------------------------------------------------------------------
// Quality gates (S194)
// ---------------------------------------------------------------------------

/// Evaluate a quality gate expression against a set of metrics.
pub fn evaluate_gate(
    expression: &str,
    metrics: &HashMap<String, f64>,
) -> QualityGate {
    // Parse simple "key op value" expressions
    let parts: Vec<&str> = expression.split_whitespace().collect();
    if parts.len() != 3 {
        return QualityGate {
            step: WorkflowStep::Custom("gate".into()),
            name: expression.to_string(),
            passed: false,
            message: format!("Cannot parse gate expression: {}", expression),
        };
    }

    let key = parts[0];
    let op = parts[1];
    let threshold: f64 = match parts[2].parse() {
        Ok(v) => v,
        Err(_) => {
            return QualityGate {
                step: WorkflowStep::Custom("gate".into()),
                name: expression.to_string(),
                passed: false,
                message: format!("Invalid threshold: {}", parts[2]),
            };
        }
    };

    let value = metrics.get(key).copied().unwrap_or(0.0);
    let passed = match op {
        "<" => value < threshold,
        ">" => value > threshold,
        "<=" => value <= threshold,
        ">=" => value >= threshold,
        "==" => (value - threshold).abs() < 1e-9,
        _ => false,
    };

    QualityGate {
        step: WorkflowStep::Custom("gate".into()),
        name: expression.to_string(),
        passed,
        message: if passed {
            format!("{} = {:.3} — OK", key, value)
        } else {
            format!("{} = {:.3} — FAILED ({} {})", key, value, op, threshold)
        },
    }
}

// ---------------------------------------------------------------------------
// Batch processing (S195)
// ---------------------------------------------------------------------------

/// Run a batch of cases through a workflow template.
pub fn run_batch(
    case_ids: &[String],
    template: &WorkflowTemplate,
    case_metrics: &HashMap<String, HashMap<String, f64>>,
) -> BatchResult {
    let mut cases = Vec::with_capacity(case_ids.len());
    let mut success = 0usize;
    let mut failure = 0usize;

    for case_id in case_ids {
        let metrics = case_metrics.get(case_id).cloned().unwrap_or_default();
        let gates: Vec<QualityGate> = template
            .quality_gates
            .iter()
            .map(|expr| evaluate_gate(expr, &metrics))
            .collect();

        let all_passed = gates.iter().all(|g| g.passed);
        let status = if all_passed {
            success += 1;
            CaseStatus::Completed
        } else {
            failure += 1;
            CaseStatus::GateFailed
        };

        cases.push(BatchCaseResult {
            case_id: case_id.clone(),
            status,
            gates,
            elapsed_ms: 0,
            errors: vec![],
        });
    }

    BatchResult {
        cases,
        total_ms: 0,
        success_count: success,
        failure_count: failure,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crown_workflow_has_all_steps() {
        let wf = crown_workflow();
        assert_eq!(wf.steps.len(), 8);
        assert!(wf.steps.contains(&WorkflowStep::Export));
    }

    #[test]
    fn preset_lookup() {
        assert!(find_preset("Standard Crown").is_some());
        assert!(find_preset("nonexistent").is_none());
    }

    #[test]
    fn gate_pass() {
        let mut metrics = HashMap::new();
        metrics.insert("margin_gap".into(), 50.0);
        let gate = evaluate_gate("margin_gap < 100", &metrics);
        assert!(gate.passed);
    }

    #[test]
    fn gate_fail() {
        let mut metrics = HashMap::new();
        metrics.insert("min_thickness".into(), 0.3);
        let gate = evaluate_gate("min_thickness > 0.5", &metrics);
        assert!(!gate.passed);
    }

    #[test]
    fn batch_mixed_results() {
        let template = crown_workflow();
        let ids = vec!["case1".into(), "case2".into()];
        let mut all_metrics = HashMap::new();

        let mut m1 = HashMap::new();
        m1.insert("margin_gap".into(), 50.0);
        m1.insert("min_thickness".into(), 0.8);
        m1.insert("occlusal_clearance".into(), 0.5);
        all_metrics.insert("case1".into(), m1);

        let mut m2 = HashMap::new();
        m2.insert("margin_gap".into(), 200.0); // fails
        m2.insert("min_thickness".into(), 0.2); // fails
        m2.insert("occlusal_clearance".into(), 0.1); // fails
        all_metrics.insert("case2".into(), m2);

        let result = run_batch(&ids, &template, &all_metrics);
        assert_eq!(result.success_count, 1);
        assert_eq!(result.failure_count, 1);
    }
}
