//! S291-S295: Quality & Validation
//!
//! Design rules engine, fit analysis, contact analysis,
//! FEA stress estimation, and manufacturing feasibility.

use serde::{Deserialize, Serialize};

/// Severity of a validation finding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// A single validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub severity: Severity,
    pub check: RuleCheck,
}

/// The check type for a validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleCheck {
    MinThickness(f64),
    MaxThickness(f64),
    MinConnectorArea(f64),
    MinMarginWidth(f64),
    MaxOcclusalDeviation(f64),
    MinWallAngle(f64),
    Custom(String),
}

/// Result of a single rule check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleResult {
    pub rule_id: String,
    pub passed: bool,
    pub actual_value: f64,
    pub expected_value: f64,
    pub severity: Severity,
    pub message: String,
}

/// Fit analysis between restoration and preparation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FitAnalysis {
    pub tooth_id: String,
    pub marginal_gap_avg_um: f64,
    pub marginal_gap_max_um: f64,
    pub internal_gap_avg_um: f64,
    pub internal_gap_max_um: f64,
    pub cement_space_um: f64,
    pub seating_complete: bool,
}

impl FitAnalysis {
    /// Check if fit meets clinical thresholds
    pub fn is_acceptable(&self) -> bool {
        self.marginal_gap_avg_um <= 120.0 && self.internal_gap_max_um <= 300.0
    }
}

/// Contact analysis between restoration and adjacent/opposing teeth
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactAnalysis {
    pub tooth_id: String,
    pub mesial_contact_force_n: f64,
    pub distal_contact_force_n: f64,
    pub occlusal_contact_points: u32,
    pub has_premature_contact: bool,
    pub max_occlusal_discrepancy_um: f64,
}

impl ContactAnalysis {
    pub fn proximal_contacts_ok(&self) -> bool {
        self.mesial_contact_force_n >= 0.5 && self.distal_contact_force_n >= 0.5
    }
}

/// Simplified FEA stress result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressEstimate {
    pub tooth_id: String,
    pub max_von_mises_mpa: f64,
    pub max_principal_stress_mpa: f64,
    pub max_displacement_um: f64,
    pub material_yield_mpa: f64,
    pub safety_factor: f64,
}

impl StressEstimate {
    pub fn is_safe(&self) -> bool {
        self.safety_factor >= 1.5
    }

    /// Compute safety factor from yield and max stress
    pub fn compute(
        tooth_id: impl Into<String>,
        max_vm: f64,
        max_principal: f64,
        max_disp: f64,
        yield_strength: f64,
    ) -> Self {
        let sf = if max_vm > 0.0 { yield_strength / max_vm } else { 10.0 };
        Self {
            tooth_id: tooth_id.into(),
            max_von_mises_mpa: max_vm,
            max_principal_stress_mpa: max_principal,
            max_displacement_um: max_disp,
            material_yield_mpa: yield_strength,
            safety_factor: sf,
        }
    }
}

/// Manufacturing feasibility check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeasibilityReport {
    pub method: ManufacturingMethod,
    pub feasible: bool,
    pub issues: Vec<String>,
    pub min_feature_size_mm: f64,
    pub estimated_cost_factor: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ManufacturingMethod {
    Milling5Axis,
    Milling3Axis,
    SLS,    // Selective Laser Sintering
    SLM,    // Selective Laser Melting
    SLA,    // Stereolithography (resin)
    DLP,    // Digital Light Processing
    Pressing,
    Casting,
}

/// Run full validation suite on a design
pub fn run_validation(rules: &[ValidationRule], values: &[(String, f64)]) -> Vec<RuleResult> {
    let val_map: std::collections::HashMap<&str, f64> =
        values.iter().map(|(k, v)| (k.as_str(), *v)).collect();

    rules.iter().map(|rule| {
        let (expected, actual, passed) = match &rule.check {
            RuleCheck::MinThickness(min) => {
                let act = val_map.get("thickness").copied().unwrap_or(0.0);
                (*min, act, act >= *min)
            }
            RuleCheck::MaxThickness(max) => {
                let act = val_map.get("thickness").copied().unwrap_or(0.0);
                (*max, act, act <= *max)
            }
            RuleCheck::MinConnectorArea(min) => {
                let act = val_map.get("connector_area").copied().unwrap_or(0.0);
                (*min, act, act >= *min)
            }
            RuleCheck::MinMarginWidth(min) => {
                let act = val_map.get("margin_width").copied().unwrap_or(0.0);
                (*min, act, act >= *min)
            }
            RuleCheck::MaxOcclusalDeviation(max) => {
                let act = val_map.get("occlusal_deviation").copied().unwrap_or(0.0);
                (*max, act, act <= *max)
            }
            RuleCheck::MinWallAngle(min) => {
                let act = val_map.get("wall_angle").copied().unwrap_or(0.0);
                (*min, act, act >= *min)
            }
            RuleCheck::Custom(_) => (0.0, 0.0, true),
        };

        RuleResult {
            rule_id: rule.id.clone(),
            passed,
            actual_value: actual,
            expected_value: expected,
            severity: rule.severity,
            message: if passed {
                format!("{}: OK", rule.name)
            } else {
                format!("{}: expected {:.2}, got {:.2}", rule.name, expected, actual)
            },
        }
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fit_acceptable() {
        let fit = FitAnalysis {
            tooth_id: "16".into(),
            marginal_gap_avg_um: 80.0,
            marginal_gap_max_um: 150.0,
            internal_gap_avg_um: 100.0,
            internal_gap_max_um: 200.0,
            cement_space_um: 30.0,
            seating_complete: true,
        };
        assert!(fit.is_acceptable());
    }

    #[test]
    fn test_fit_not_acceptable() {
        let fit = FitAnalysis {
            tooth_id: "26".into(),
            marginal_gap_avg_um: 150.0,
            marginal_gap_max_um: 250.0,
            internal_gap_avg_um: 200.0,
            internal_gap_max_um: 350.0,
            cement_space_um: 50.0,
            seating_complete: false,
        };
        assert!(!fit.is_acceptable());
    }

    #[test]
    fn test_stress_safe() {
        let stress = StressEstimate::compute("16", 100.0, 120.0, 5.0, 400.0);
        assert!(stress.is_safe());
        assert!((stress.safety_factor - 4.0).abs() < 0.001);
    }

    #[test]
    fn test_stress_unsafe() {
        let stress = StressEstimate::compute("36", 400.0, 450.0, 20.0, 400.0);
        assert!(!stress.is_safe());
    }

    #[test]
    fn test_validation_rules() {
        let rules = vec![
            ValidationRule {
                id: "R1".into(), name: "Min Thickness".into(),
                description: "Check minimum wall thickness".into(),
                severity: Severity::Error,
                check: RuleCheck::MinThickness(0.5),
            },
        ];
        let values = vec![("thickness".into(), 0.3)];
        let results = run_validation(&rules, &values);
        assert_eq!(results.len(), 1);
        assert!(!results[0].passed);
    }

    #[test]
    fn test_contact_analysis() {
        let ca = ContactAnalysis {
            tooth_id: "16".into(),
            mesial_contact_force_n: 1.0,
            distal_contact_force_n: 0.8,
            occlusal_contact_points: 3,
            has_premature_contact: false,
            max_occlusal_discrepancy_um: 30.0,
        };
        assert!(ca.proximal_contacts_ok());
    }
}

// ── S275-S280 Iteration: Extended quality validation ──

/// Batch validation — run full QC on multiple restorations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchQcResult {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub pass_rate: f64,
    pub common_failures: Vec<String>,
}

pub fn batch_validate(
    items: &[(String, Vec<(String, f64)>)],
    rules: &[ValidationRule],
) -> BatchQcResult {
    let mut passed = 0usize;
    let mut failures: Vec<String> = Vec::new();

    for (_, values) in items {
        let results = run_validation(rules, values);
        if results.iter().all(|r| r.passed) {
            passed += 1;
        } else {
            for r in &results {
                if !r.passed { failures.push(r.rule_id.clone()); }
            }
        }
    }

    let total = items.len();
    let pass_rate = if total > 0 { passed as f64 / total as f64 * 100.0 } else { 0.0 };

    // Count most common failure
    let mut failure_counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for f in &failures { *failure_counts.entry(f.as_str()).or_default() += 1; }
    let mut common: Vec<String> = failure_counts.into_iter()
        .map(|(k, _)| k.to_string())
        .collect();
    common.sort();
    common.dedup();

    BatchQcResult { total, passed, failed: total - passed, pass_rate, common_failures: common }
}

/// ISO 6872 compliance check for dental ceramics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Iso6872Check {
    pub material: String,
    pub flexural_strength_mpa: f64,
    pub required_mpa: f64,
    pub compliant: bool,
}

pub fn check_iso6872(material: impl Into<String>, strength: f64, required: f64) -> Iso6872Check {
    Iso6872Check {
        material: material.into(),
        flexural_strength_mpa: strength,
        required_mpa: required,
        compliant: strength >= required,
    }
}

#[cfg(test)]
mod tests_extended {
    use super::*;

    #[test]
    fn test_batch_validate_all_pass() {
        let rules = vec![
            ValidationRule {
                id: "R1".into(), name: "Thickness".into(),
                description: "".into(), severity: Severity::Error,
                check: RuleCheck::MinThickness(0.5),
            },
        ];
        let items = vec![
            ("C1".into(), vec![("thickness".into(), 0.8)]),
            ("C2".into(), vec![("thickness".into(), 1.0)]),
        ];
        let result = batch_validate(&items, &rules);
        assert_eq!(result.passed, 2);
        assert!((result.pass_rate - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_batch_validate_some_fail() {
        let rules = vec![
            ValidationRule {
                id: "R1".into(), name: "Thickness".into(),
                description: "".into(), severity: Severity::Error,
                check: RuleCheck::MinThickness(0.5),
            },
        ];
        let items = vec![
            ("C1".into(), vec![("thickness".into(), 0.3)]),
            ("C2".into(), vec![("thickness".into(), 0.8)]),
        ];
        let result = batch_validate(&items, &rules);
        assert_eq!(result.failed, 1);
        assert!(result.pass_rate < 100.0);
    }

    #[test]
    fn test_iso6872_compliant() {
        let c = check_iso6872("Zirconia", 1200.0, 800.0);
        assert!(c.compliant);
    }

    #[test]
    fn test_iso6872_non_compliant() {
        let c = check_iso6872("Feldspathic", 80.0, 100.0);
        assert!(!c.compliant);
    }
}
