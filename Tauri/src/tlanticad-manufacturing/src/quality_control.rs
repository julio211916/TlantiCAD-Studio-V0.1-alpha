//! S329-S333: Quality Control & Inspection
//!
//! Dimensional verification, surface analysis, fit testing, and batch QC.

use serde::{Deserialize, Serialize};

/// QC check result
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QcVerdict {
    Pass,
    ConditionalPass,
    Fail,
    NeedsRework,
}

/// Dimensional measurement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionalCheck {
    pub feature: String,
    pub nominal_mm: f64,
    pub actual_mm: f64,
    pub tolerance_mm: f64,
    pub deviation_mm: f64,
    pub in_tolerance: bool,
}

impl DimensionalCheck {
    pub fn new(feature: impl Into<String>, nominal: f64, actual: f64, tolerance: f64) -> Self {
        let dev = actual - nominal;
        Self {
            feature: feature.into(),
            nominal_mm: nominal,
            actual_mm: actual,
            tolerance_mm: tolerance,
            deviation_mm: dev,
            in_tolerance: dev.abs() <= tolerance,
        }
    }
}

/// Surface quality measurement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfaceQuality {
    pub roughness_ra_um: f64,
    pub roughness_rz_um: f64,
    pub max_defect_size_um: f64,
    pub defect_count: usize,
    pub acceptable: bool,
}

impl SurfaceQuality {
    pub fn evaluate(ra: f64, rz: f64, defects: usize, max_defect: f64) -> Self {
        // Dental standards: Ra < 0.8 µm for intaglio, < 0.2 for polished
        let acceptable = ra < 0.8 && defects < 3 && max_defect < 50.0;
        Self {
            roughness_ra_um: ra,
            roughness_rz_um: rz,
            max_defect_size_um: max_defect,
            defect_count: defects,
            acceptable,
        }
    }
}

/// Marginal fit measurement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginalFit {
    pub mean_gap_um: f64,
    pub max_gap_um: f64,
    pub min_gap_um: f64,
    pub measurement_points: usize,
    pub clinical_acceptable: bool,
}

impl MarginalFit {
    pub fn evaluate(gaps_um: &[f64]) -> Self {
        if gaps_um.is_empty() {
            return Self {
                mean_gap_um: 0.0, max_gap_um: 0.0, min_gap_um: 0.0,
                measurement_points: 0, clinical_acceptable: false,
            };
        }
        let mean = gaps_um.iter().sum::<f64>() / gaps_um.len() as f64;
        let max = gaps_um.iter().cloned().fold(0.0_f64, f64::max);
        let min = gaps_um.iter().cloned().fold(f64::INFINITY, f64::min);
        // Clinical standard: mean < 120µm, max < 200µm
        let acceptable = mean < 120.0 && max < 200.0;
        Self {
            mean_gap_um: mean, max_gap_um: max, min_gap_um: min,
            measurement_points: gaps_um.len(), clinical_acceptable: acceptable,
        }
    }
}

/// Internal fit measurement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalFit {
    pub mean_gap_um: f64,
    pub max_gap_um: f64,
    pub axial_gap_um: f64,
    pub occlusal_gap_um: f64,
    pub acceptable: bool,
}

impl InternalFit {
    pub fn evaluate(axial: f64, occlusal: f64) -> Self {
        let mean = (axial + occlusal) / 2.0;
        let max = axial.max(occlusal);
        // Axial gap < 100µm, occlusal < 200µm
        let acceptable = axial < 100.0 && occlusal < 200.0;
        Self {
            mean_gap_um: mean, max_gap_um: max,
            axial_gap_um: axial, occlusal_gap_um: occlusal, acceptable,
        }
    }
}

/// Occlusal contact verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcclusalContactQC {
    pub contacts_count: usize,
    pub premature_contacts: usize,
    pub max_high_spot_um: f64,
    pub holding_paper_thickness_um: f64,
    pub pass: bool,
}

/// Complete QC report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QcReport {
    pub part_id: String,
    pub dimensional_checks: Vec<DimensionalCheck>,
    pub surface: Option<SurfaceQuality>,
    pub marginal_fit: Option<MarginalFit>,
    pub internal_fit: Option<InternalFit>,
    pub occlusal_contact: Option<OcclusalContactQC>,
    pub overall_verdict: QcVerdict,
    pub notes: Vec<String>,
}

impl QcReport {
    pub fn evaluate(
        part_id: impl Into<String>,
        dims: Vec<DimensionalCheck>,
        surface: Option<SurfaceQuality>,
        marginal: Option<MarginalFit>,
        internal: Option<InternalFit>,
    ) -> Self {
        let mut notes = Vec::new();
        let dims_ok = dims.iter().all(|d| d.in_tolerance);
        if !dims_ok { notes.push("Dimensional out of tolerance".into()); }

        let surface_ok = surface.as_ref().map_or(true, |s| s.acceptable);
        if !surface_ok { notes.push("Surface quality below threshold".into()); }

        let marginal_ok = marginal.as_ref().map_or(true, |m| m.clinical_acceptable);
        if !marginal_ok { notes.push("Marginal fit exceeds clinical limit".into()); }

        let internal_ok = internal.as_ref().map_or(true, |i| i.acceptable);
        if !internal_ok { notes.push("Internal fit exceeds limit".into()); }

        let verdict = if dims_ok && surface_ok && marginal_ok && internal_ok {
            QcVerdict::Pass
        } else if !marginal_ok || !internal_ok {
            QcVerdict::Fail
        } else {
            QcVerdict::NeedsRework
        };

        Self {
            part_id: part_id.into(),
            dimensional_checks: dims,
            surface, marginal_fit: marginal, internal_fit: internal,
            occlusal_contact: None,
            overall_verdict: verdict, notes,
        }
    }
}

/// Batch QC summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchQcSummary {
    pub total_parts: usize,
    pub passed: usize,
    pub failed: usize,
    pub rework: usize,
    pub pass_rate_pct: f64,
}

impl BatchQcSummary {
    pub fn from_reports(reports: &[QcReport]) -> Self {
        let passed = reports.iter().filter(|r| r.overall_verdict == QcVerdict::Pass).count();
        let failed = reports.iter().filter(|r| r.overall_verdict == QcVerdict::Fail).count();
        let rework = reports.iter().filter(|r| r.overall_verdict == QcVerdict::NeedsRework).count();
        let total = reports.len();
        let rate = if total > 0 { passed as f64 / total as f64 * 100.0 } else { 0.0 };
        Self { total_parts: total, passed, failed, rework, pass_rate_pct: rate }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dimensional_check_pass() {
        let check = DimensionalCheck::new("width", 10.0, 10.05, 0.1);
        assert!(check.in_tolerance);
    }

    #[test]
    fn test_dimensional_check_fail() {
        let check = DimensionalCheck::new("height", 8.0, 8.3, 0.1);
        assert!(!check.in_tolerance);
    }

    #[test]
    fn test_surface_quality() {
        let sq = SurfaceQuality::evaluate(0.5, 3.2, 1, 30.0);
        assert!(sq.acceptable);
    }

    #[test]
    fn test_marginal_fit_good() {
        let mf = MarginalFit::evaluate(&[80.0, 90.0, 100.0, 85.0, 95.0]);
        assert!(mf.clinical_acceptable);
        assert!(mf.mean_gap_um < 120.0);
    }

    #[test]
    fn test_marginal_fit_bad() {
        let mf = MarginalFit::evaluate(&[150.0, 180.0, 220.0]);
        assert!(!mf.clinical_acceptable);
    }

    #[test]
    fn test_internal_fit() {
        let fit = InternalFit::evaluate(70.0, 150.0);
        assert!(fit.acceptable);
    }

    #[test]
    fn test_qc_report_pass() {
        let dims = vec![DimensionalCheck::new("w", 10.0, 10.02, 0.1)];
        let surface = SurfaceQuality::evaluate(0.3, 2.0, 0, 0.0);
        let marginal = MarginalFit::evaluate(&[80.0, 90.0]);
        let report = QcReport::evaluate("part-1", dims, Some(surface), Some(marginal), None);
        assert_eq!(report.overall_verdict, QcVerdict::Pass);
    }

    #[test]
    fn test_qc_report_fail() {
        let dims = vec![DimensionalCheck::new("w", 10.0, 10.02, 0.1)];
        let marginal = MarginalFit::evaluate(&[250.0, 300.0]);
        let report = QcReport::evaluate("part-2", dims, None, Some(marginal), None);
        assert_eq!(report.overall_verdict, QcVerdict::Fail);
    }

    #[test]
    fn test_batch_summary() {
        let r1 = QcReport::evaluate("p1", vec![DimensionalCheck::new("w", 10.0, 10.0, 0.1)], None, None, None);
        let r2 = QcReport::evaluate("p2", vec![DimensionalCheck::new("w", 10.0, 10.5, 0.1)], None, None, None);
        let summary = BatchQcSummary::from_reports(&[r1, r2]);
        assert_eq!(summary.total_parts, 2);
        assert_eq!(summary.passed, 1);
    }

    #[test]
    fn test_dimensional_in_tolerance() {
        let d = DimensionalCheck::new("length", 20.0, 20.03, 0.05);
        assert!(d.in_tolerance);
    }

    #[test]
    fn test_dimensional_out_of_tolerance() {
        let d = DimensionalCheck::new("length", 20.0, 20.2, 0.05);
        assert!(!d.in_tolerance);
    }

    #[test]
    fn test_internal_fit_reject() {
        let fit = InternalFit::evaluate(200.0, 500.0);
        assert!(!fit.acceptable);
    }
}
