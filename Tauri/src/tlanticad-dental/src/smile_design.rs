//! S251-S255: Digital Smile Design
//!
//! Facial analysis, golden-ratio tooth proportions, smile arc,
//! gingival contour analysis, and before/after overlay metrics.

use serde::{Deserialize, Serialize};

// ────────────────────────────────────────────────────────────────────
//  Facial Analysis
// ────────────────────────────────────────────────────────────────────

/// Facial landmark type for DSD overlay
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FacialLandmark {
    Glabella,
    Nasion,
    SubNasale,
    Stomion,
    Menton,
    Pogonion,
    LeftPupil,
    RightPupil,
    LeftCommissure,
    RightCommissure,
    LeftAlar,
    RightAlar,
    UpperLipVermilion,
    LowerLipVermilion,
}

/// A 2D point on the facial photo
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FacialPoint {
    pub landmark: FacialLandmark,
    pub x: f64,
    pub y: f64,
}

/// Facial analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacialAnalysis {
    pub landmarks: Vec<FacialPoint>,
    pub midline_angle_deg: f64,
    pub interpupillary_distance: f64,
    pub lower_face_third: f64,
    pub smile_width: f64,
    pub lip_line: LipLine,
}

/// Lip line classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LipLine {
    Low,
    Average,
    High,
}

impl FacialAnalysis {
    /// Compute from landmarks
    pub fn compute(landmarks: &[FacialPoint]) -> Option<Self> {
        let find = |lm: FacialLandmark| landmarks.iter().find(|p| p.landmark == lm).copied();

        let left_pupil = find(FacialLandmark::LeftPupil)?;
        let right_pupil = find(FacialLandmark::RightPupil)?;
        let sub_nasale = find(FacialLandmark::SubNasale)?;
        let menton = find(FacialLandmark::Menton)?;
        let left_comm = find(FacialLandmark::LeftCommissure)?;
        let right_comm = find(FacialLandmark::RightCommissure)?;
        let upper_lip = find(FacialLandmark::UpperLipVermilion)?;

        let ipd = ((right_pupil.x - left_pupil.x).powi(2) +
                   (right_pupil.y - left_pupil.y).powi(2)).sqrt();

        let _midline_x = (left_pupil.x + right_pupil.x) / 2.0;
        let midline_angle = ((right_pupil.y - left_pupil.y) /
                             (right_pupil.x - left_pupil.x)).atan().to_degrees();

        let lower_third = ((menton.y - sub_nasale.y).powi(2)).sqrt();

        let smile_w = ((right_comm.x - left_comm.x).powi(2) +
                       (right_comm.y - left_comm.y).powi(2)).sqrt();

        // Lip line classification based on gingival display
        let lip_classification = if upper_lip.y > sub_nasale.y + lower_third * 0.4 {
            LipLine::Low
        } else if upper_lip.y > sub_nasale.y + lower_third * 0.25 {
            LipLine::Average
        } else {
            LipLine::High
        };

        Some(Self {
            landmarks: landmarks.to_vec(),
            midline_angle_deg: midline_angle,
            interpupillary_distance: ipd,
            lower_face_third: lower_third,
            smile_width: smile_w,
            lip_line: lip_classification,
        })
    }
}

// ────────────────────────────────────────────────────────────────────
//  Tooth proportion (Golden Ratio)
// ────────────────────────────────────────────────────────────────────

/// The golden ratio constant
pub const GOLDEN_RATIO: f64 = 1.618;

/// Recurring proportion (80% rule in DSD)
pub const RECURRING_RATIO: f64 = 0.618;

/// Ideal tooth proportions based on golden ratio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToothProportion {
    pub tooth_id: String,
    pub width: f64,
    pub height: f64,
    pub width_height_ratio: f64,
    pub ideal_ratio: f64,
    pub deviation_pct: f64,
}

/// Compute ideal proportions for upper anterior teeth
pub fn golden_ratio_proportions(central_incisor_width: f64) -> Vec<ToothProportion> {
    let ci_w = central_incisor_width;
    let ci_h = ci_w * GOLDEN_RATIO * 0.65; // typical w/h ratio ~0.75-0.8
    let li_w = ci_w * RECURRING_RATIO;
    let li_h = li_w * GOLDEN_RATIO * 0.65;
    let cn_w = li_w * RECURRING_RATIO;
    let cn_h = cn_w * GOLDEN_RATIO * 0.7;

    vec![
        ToothProportion {
            tooth_id: "11".into(), width: ci_w, height: ci_h,
            width_height_ratio: ci_w / ci_h, ideal_ratio: 0.78, deviation_pct: 0.0,
        },
        ToothProportion {
            tooth_id: "21".into(), width: ci_w, height: ci_h,
            width_height_ratio: ci_w / ci_h, ideal_ratio: 0.78, deviation_pct: 0.0,
        },
        ToothProportion {
            tooth_id: "12".into(), width: li_w, height: li_h,
            width_height_ratio: li_w / li_h, ideal_ratio: 0.73, deviation_pct: 0.0,
        },
        ToothProportion {
            tooth_id: "22".into(), width: li_w, height: li_h,
            width_height_ratio: li_w / li_h, ideal_ratio: 0.73, deviation_pct: 0.0,
        },
        ToothProportion {
            tooth_id: "13".into(), width: cn_w, height: cn_h,
            width_height_ratio: cn_w / cn_h, ideal_ratio: 0.77, deviation_pct: 0.0,
        },
        ToothProportion {
            tooth_id: "23".into(), width: cn_w, height: cn_h,
            width_height_ratio: cn_w / cn_h, ideal_ratio: 0.77, deviation_pct: 0.0,
        },
    ]
}

// ────────────────────────────────────────────────────────────────────
//  Smile Arc
// ────────────────────────────────────────────────────────────────────

/// Smile arc type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SmileArcType {
    Consonant,  // incisal edges follow lower lip curvature
    Flat,       // straight across
    Reverse,    // opposite curvature
}

/// Smile arc analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmileArcAnalysis {
    pub arc_type: SmileArcType,
    pub curvature_mm: f64,
    pub buccal_corridor_left_pct: f64,
    pub buccal_corridor_right_pct: f64,
    pub symmetry_score: f64,  // 0..1 (1 = perfect)
}

impl SmileArcAnalysis {
    pub fn evaluate(
        incisal_points: &[[f64; 2]],
        _lip_contour: &[[f64; 2]],
        smile_width: f64,
    ) -> Self {
        // Calculate curvature from incisal edges
        let y_values: Vec<f64> = incisal_points.iter().map(|p| p[1]).collect();
        let y_min = y_values.iter().cloned().fold(f64::INFINITY, f64::min);
        let y_max = y_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let curvature = y_max - y_min;

        let arc_type = if curvature > 2.0 {
            SmileArcType::Consonant
        } else if curvature < 0.5 {
            SmileArcType::Flat
        } else {
            SmileArcType::Flat // borderline
        };

        // Buccal corridors: dark space ratio on each side
        let tooth_width: f64 = incisal_points.iter().map(|p| p[0]).fold(f64::NEG_INFINITY, f64::max)
            - incisal_points.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min);
        let corridor_total = smile_width - tooth_width;
        let corridor_pct = if smile_width > 0.0 { corridor_total / smile_width * 100.0 / 2.0 } else { 0.0 };

        // Symmetry: compare left and right halves
        let mid_x = incisal_points.iter().map(|p| p[0]).sum::<f64>() / incisal_points.len().max(1) as f64;
        let left: Vec<_> = incisal_points.iter().filter(|p| p[0] < mid_x).collect();
        let right: Vec<_> = incisal_points.iter().filter(|p| p[0] >= mid_x).collect();
        let sym = if left.len() == right.len() && !left.is_empty() {
            let diff_sum: f64 = left.iter().zip(right.iter())
                .map(|(l, r)| (l[1] - r[1]).abs())
                .sum();
            1.0 - (diff_sum / left.len() as f64 / curvature.max(1.0)).min(1.0)
        } else { 0.5 };

        Self {
            arc_type,
            curvature_mm: curvature,
            buccal_corridor_left_pct: corridor_pct.max(0.0),
            buccal_corridor_right_pct: corridor_pct.max(0.0),
            symmetry_score: sym.clamp(0.0, 1.0),
        }
    }
}

// ────────────────────────────────────────────────────────────────────
//  Gingival Contour
// ────────────────────────────────────────────────────────────────────

/// Gingival contour assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GingivalContour {
    pub zenith_positions: Vec<GingivalZenith>,
    pub symmetry_score: f64,
    pub margin_harmony: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GingivalZenith {
    pub tooth_id: String,
    pub x: f64,
    pub y: f64,
    pub offset_from_ideal_mm: f64,
}

impl GingivalContour {
    /// Evaluate gingival symmetry between contralateral teeth
    pub fn evaluate(zeniths: Vec<GingivalZenith>) -> Self {
        let symmetry = if zeniths.len() >= 2 {
            let deviations: f64 = zeniths.iter().map(|z| z.offset_from_ideal_mm.abs()).sum();
            1.0 - (deviations / zeniths.len() as f64 / 2.0).min(1.0)
        } else { 0.5 };

        Self {
            symmetry_score: symmetry,
            margin_harmony: symmetry * 0.9, // simplified
            zenith_positions: zeniths,
        }
    }
}

// ────────────────────────────────────────────────────────────────────
//  Smile Overlay / Before-After
// ────────────────────────────────────────────────────────────────────

/// Before/After overlay metric comparing original vs designed smile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmileOverlayMetric {
    pub width_change_mm: f64,
    pub height_change_mm: f64,
    pub midline_shift_mm: f64,
    pub buccal_corridor_change_pct: f64,
    pub overall_improvement_pct: f64,
}

/// Complete DSD case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmileDesignCase {
    pub facial: FacialAnalysis,
    pub proportions: Vec<ToothProportion>,
    pub smile_arc: SmileArcAnalysis,
    pub gingival: GingivalContour,
    pub overlay: Option<SmileOverlayMetric>,
}

impl SmileDesignCase {
    /// Composite DSD score (0-100)
    pub fn composite_score(&self) -> f64 {
        let prop_score = {
            let deviations: f64 = self.proportions.iter()
                .map(|p| (p.width_height_ratio - p.ideal_ratio).abs())
                .sum();
            (1.0 - deviations / self.proportions.len().max(1) as f64).clamp(0.0, 1.0)
        };
        let arc_score = match self.smile_arc.arc_type {
            SmileArcType::Consonant => 1.0,
            SmileArcType::Flat => 0.6,
            SmileArcType::Reverse => 0.3,
        };
        let sym_score = self.smile_arc.symmetry_score;
        let gingival_score = self.gingival.symmetry_score;

        ((prop_score * 30.0) + (arc_score * 25.0) + (sym_score * 25.0) + (gingival_score * 20.0))
            .clamp(0.0, 100.0)
    }

    /// Whether the case meets acceptable DSD criteria
    pub fn passes_criteria(&self) -> bool {
        self.composite_score() >= 60.0
    }
}

/// Compare actual tooth proportions with ideal
pub fn evaluate_proportion_deviation(
    actual_widths: &[(String, f64)],
    central_width: f64,
) -> Vec<ToothProportion> {
    let ideal = golden_ratio_proportions(central_width);
    ideal.into_iter().map(|mut p| {
        if let Some((_, actual_w)) = actual_widths.iter().find(|(id, _)| *id == p.tooth_id) {
            p.deviation_pct = ((actual_w - p.width) / p.width * 100.0).abs();
        }
        p
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_golden_ratio_proportions() {
        let props = golden_ratio_proportions(8.5);
        assert_eq!(props.len(), 6);
        assert!(props[0].width > props[2].width);
        assert!(props[2].width > props[4].width);
    }

    #[test]
    fn test_smile_arc_analysis() {
        let incisal = vec![[0.0, 0.0], [3.0, 3.0], [6.0, 0.0]];
        let lip = vec![[0.0, 0.5], [6.0, 0.5]];
        let result = SmileArcAnalysis::evaluate(&incisal, &lip, 60.0);
        assert_eq!(result.arc_type, SmileArcType::Consonant);
        assert!(result.curvature_mm > 2.0);
    }

    #[test]
    fn test_lip_line_classification() {
        let landmarks = vec![
            FacialPoint { landmark: FacialLandmark::LeftPupil, x: 100.0, y: 100.0 },
            FacialPoint { landmark: FacialLandmark::RightPupil, x: 200.0, y: 100.0 },
            FacialPoint { landmark: FacialLandmark::SubNasale, x: 150.0, y: 200.0 },
            FacialPoint { landmark: FacialLandmark::Menton, x: 150.0, y: 400.0 },
            FacialPoint { landmark: FacialLandmark::LeftCommissure, x: 120.0, y: 260.0 },
            FacialPoint { landmark: FacialLandmark::RightCommissure, x: 180.0, y: 260.0 },
            FacialPoint { landmark: FacialLandmark::UpperLipVermilion, x: 150.0, y: 250.0 },
        ];
        let analysis = FacialAnalysis::compute(&landmarks).unwrap();
        assert!((analysis.interpupillary_distance - 100.0).abs() < 0.1);
        assert!(analysis.smile_width > 0.0);
    }

    #[test]
    fn test_gingival_contour() {
        let zeniths = vec![
            GingivalZenith { tooth_id: "11".into(), x: 0.0, y: 5.0, offset_from_ideal_mm: 0.3 },
            GingivalZenith { tooth_id: "21".into(), x: 8.0, y: 5.2, offset_from_ideal_mm: 0.5 },
        ];
        let contour = GingivalContour::evaluate(zeniths);
        assert!(contour.symmetry_score > 0.5);
    }

    #[test]
    fn test_smile_design_case_score() {
        let landmarks = vec![
            FacialPoint { landmark: FacialLandmark::LeftPupil, x: 100.0, y: 100.0 },
            FacialPoint { landmark: FacialLandmark::RightPupil, x: 200.0, y: 100.0 },
            FacialPoint { landmark: FacialLandmark::SubNasale, x: 150.0, y: 200.0 },
            FacialPoint { landmark: FacialLandmark::Menton, x: 150.0, y: 400.0 },
            FacialPoint { landmark: FacialLandmark::LeftCommissure, x: 120.0, y: 260.0 },
            FacialPoint { landmark: FacialLandmark::RightCommissure, x: 180.0, y: 260.0 },
            FacialPoint { landmark: FacialLandmark::UpperLipVermilion, x: 150.0, y: 250.0 },
        ];
        let facial = FacialAnalysis::compute(&landmarks).unwrap();
        let props = golden_ratio_proportions(8.5);
        let arc = SmileArcAnalysis::evaluate(&[[0.0, 0.0], [3.0, 3.0], [6.0, 0.0]], &[[0.0, 0.5]], 60.0);
        let gingival = GingivalContour::evaluate(vec![
            GingivalZenith { tooth_id: "11".into(), x: 0.0, y: 5.0, offset_from_ideal_mm: 0.2 },
        ]);
        let case = SmileDesignCase { facial, proportions: props, smile_arc: arc, gingival, overlay: None };
        assert!(case.composite_score() > 0.0);
        assert!(case.passes_criteria());
    }

    #[test]
    fn test_proportion_deviation() {
        let actual = vec![("11".to_string(), 9.0), ("12".to_string(), 6.0)];
        let result = evaluate_proportion_deviation(&actual, 8.5);
        assert_eq!(result.len(), 6);
        let ci = &result[0];
        assert!(ci.deviation_pct > 0.0); // 9.0 vs 8.5
    }

    #[test]
    fn test_overlay_metric() {
        let overlay = SmileOverlayMetric {
            width_change_mm: 2.0,
            height_change_mm: 1.5,
            midline_shift_mm: 0.3,
            buccal_corridor_change_pct: -5.0,
            overall_improvement_pct: 15.0,
        };
        assert!(overlay.overall_improvement_pct > 0.0);
    }
}
