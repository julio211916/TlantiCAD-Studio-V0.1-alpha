//! S156-S160: Motor Bridge — automated bridge generation pipeline.
//!
//! Pontic selection, connector sizing, framework optimization,
//! multi-span bridge generation, and stress analysis.

use nalgebra::Point3;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Pontic shape preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PonticType {
    /// Bullet-shaped (hygienic)
    Bullet,
    /// Modified ridge lap
    ModifiedRidgeLap,
    /// Ovate (depresses into ridge)
    Ovate,
    /// Conical
    Conical,
}

/// Connector constraint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorSpec {
    pub min_area_mm2: f64,
    pub min_height: f64,
    pub min_width: f64,
    pub radius_at_gingiva: f64,
}

impl Default for ConnectorSpec {
    fn default() -> Self {
        Self {
            min_area_mm2: 9.0, // 3×3 mm for posterior
            min_height: 3.0,
            min_width: 3.0,
            radius_at_gingiva: 0.5,
        }
    }
}

/// Single span in a bridge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeSpan {
    pub abutment_fdi: Vec<u8>,
    pub pontic_fdi: Vec<u8>,
    pub pontic_type: PonticType,
    pub connector_spec: ConnectorSpec,
}

/// Motor bridge parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotorBridgeParams {
    pub spans: Vec<BridgeSpan>,
    pub framework_thickness: f64,
    pub cement_gap_um: f64,
    pub smooth_iterations: u32,
}

impl Default for MotorBridgeParams {
    fn default() -> Self {
        Self {
            spans: vec![BridgeSpan {
                abutment_fdi: vec![14, 16],
                pontic_fdi: vec![15],
                pontic_type: PonticType::ModifiedRidgeLap,
                connector_spec: ConnectorSpec::default(),
            }],
            framework_thickness: 0.5,
            cement_gap_um: 50.0,
            smooth_iterations: 2,
        }
    }
}

/// Result of bridge generation.
#[derive(Debug, Clone)]
pub struct MotorBridgeResult {
    pub framework_vertices: Vec<Point3<f64>>,
    pub framework_indices: Vec<[u32; 3]>,
    pub pontic_vertices: Vec<Vec<Point3<f64>>>,
    pub connector_areas: Vec<f64>,
    pub metrics: BridgeMetrics,
}

/// Bridge quality metrics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BridgeMetrics {
    pub total_span_length: f64,
    pub min_connector_area: f64,
    pub max_deflection_est: f64,
    pub framework_volume: f64,
}

// ---------------------------------------------------------------------------
// Bridge generation (S156-S158)
// ---------------------------------------------------------------------------

/// Generate a bridge framework between abutments.
pub fn generate_bridge(
    abutment_positions: &[Point3<f64>],
    ridge_crest: &[Point3<f64>],
    params: &MotorBridgeParams,
) -> MotorBridgeResult {
    if abutment_positions.len() < 2 {
        return MotorBridgeResult {
            framework_vertices: vec![],
            framework_indices: vec![],
            pontic_vertices: vec![],
            connector_areas: vec![],
            metrics: BridgeMetrics::default(),
        };
    }

    // Compute span length
    let mut total_span = 0.0;
    for i in 1..abutment_positions.len() {
        total_span += (abutment_positions[i] - abutment_positions[i - 1]).norm();
    }

    // Generate pontic positions along the span
    let mut pontic_verts = Vec::new();
    let mut connector_areas = Vec::new();
    for span in &params.spans {
        let n_pontics = span.pontic_fdi.len();
        for p_idx in 0..n_pontics {
            let t = (p_idx as f64 + 1.0) / (n_pontics as f64 + 1.0);
            let pos = interpolate_along_points(abutment_positions, t);
            let pontic = generate_pontic(&pos, &span.pontic_type, ridge_crest);
            pontic_verts.push(pontic);
            connector_areas.push(span.connector_spec.min_area_mm2);
        }
    }

    // Framework: connect abutments through pontics
    let framework_vertices = generate_framework_path(abutment_positions, params.framework_thickness);
    let min_conn = connector_areas.iter().cloned().fold(f64::MAX, f64::min);
    let deflection = estimate_deflection(total_span, min_conn, params.framework_thickness);

    MotorBridgeResult {
        framework_vertices,
        framework_indices: vec![],
        pontic_vertices: pontic_verts,
        metrics: BridgeMetrics {
            total_span_length: total_span,
            min_connector_area: if connector_areas.is_empty() { 0.0 } else { min_conn },
            max_deflection_est: deflection,
            framework_volume: total_span * params.framework_thickness * params.framework_thickness,
        },
        connector_areas,
    }
}

// ---------------------------------------------------------------------------
// Connector analysis (S159)
// ---------------------------------------------------------------------------

/// Check if connector areas are sufficient for the span.
pub fn validate_connectors(result: &MotorBridgeResult, params: &MotorBridgeParams) -> Vec<String> {
    let mut issues = Vec::new();
    for (i, &area) in result.connector_areas.iter().enumerate() {
        for span in &params.spans {
            if area < span.connector_spec.min_area_mm2 {
                issues.push(format!(
                    "Connector {} area {:.1} mm² below minimum {:.1} mm²",
                    i, area, span.connector_spec.min_area_mm2
                ));
            }
        }
    }
    if result.metrics.max_deflection_est > 0.1 {
        issues.push(format!(
            "Estimated deflection {:.3} mm exceeds 0.1 mm limit",
            result.metrics.max_deflection_est
        ));
    }
    issues
}

// ---------------------------------------------------------------------------
// Stress estimation (S160)
// ---------------------------------------------------------------------------

/// Simple beam deflection estimate for bridge span.
pub fn estimate_deflection(span_length: f64, connector_area: f64, thickness: f64) -> f64 {
    if connector_area <= 0.0 || thickness <= 0.0 {
        return 0.0;
    }
    // Simplified: δ ≈ F·L³ / (48·E·I), assume F=400N, E=210 GPa (zirconia)
    let force = 400.0; // Newtons (max bite force)
    let e = 210_000.0; // MPa (zirconia modulus)
    let i = connector_area * thickness * thickness / 12.0; // mm⁴
    let l = span_length; // mm
    if i <= 0.0 { return 0.0; }
    force * l.powi(3) / (48.0 * e * i)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn interpolate_along_points(points: &[Point3<f64>], t: f64) -> Point3<f64> {
    if points.len() < 2 {
        return points.first().copied().unwrap_or(Point3::origin());
    }
    let n = points.len() - 1;
    let segment = (t * n as f64).floor() as usize;
    let seg = segment.min(n - 1);
    let local_t = t * n as f64 - seg as f64;
    Point3::from(
        points[seg].coords * (1.0 - local_t) + points[seg + 1].coords * local_t,
    )
}

fn generate_pontic(
    center: &Point3<f64>,
    pontic_type: &PonticType,
    _ridge: &[Point3<f64>],
) -> Vec<Point3<f64>> {
    let r = match pontic_type {
        PonticType::Bullet => 2.0,
        PonticType::ModifiedRidgeLap => 2.5,
        PonticType::Ovate => 2.0,
        PonticType::Conical => 1.5,
    };
    // Generate simple circular cross-section
    (0..8)
        .map(|i| {
            let angle = std::f64::consts::TAU * i as f64 / 8.0;
            Point3::new(
                center.x + r * angle.cos(),
                center.y + r * angle.sin(),
                center.z,
            )
        })
        .collect()
}

fn generate_framework_path(
    abutments: &[Point3<f64>],
    _thickness: f64,
) -> Vec<Point3<f64>> {
    // Simple: interpolate between abutments
    let mut path = Vec::new();
    for i in 0..abutments.len().saturating_sub(1) {
        for s in 0..10 {
            let t = s as f64 / 10.0;
            path.push(Point3::from(
                abutments[i].coords * (1.0 - t) + abutments[i + 1].coords * t,
            ));
        }
    }
    if let Some(last) = abutments.last() {
        path.push(*last);
    }
    path
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_bridge_basic() {
        let abutments = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(10.0, 0.0, 0.0),
        ];
        let ridge = vec![Point3::new(5.0, 0.0, -2.0)];
        let params = MotorBridgeParams::default();
        let result = generate_bridge(&abutments, &ridge, &params);
        assert!(result.metrics.total_span_length > 0.0);
        assert!(!result.framework_vertices.is_empty());
    }

    #[test]
    fn generate_bridge_single_abutment() {
        let abutments = vec![Point3::new(0.0, 0.0, 0.0)];
        let result = generate_bridge(&abutments, &[], &MotorBridgeParams::default());
        assert!(result.framework_vertices.is_empty());
    }

    #[test]
    fn deflection_estimate_reasonable() {
        let d = estimate_deflection(10.0, 9.0, 3.0);
        assert!(d > 0.0);
        assert!(d < 1.0, "Deflection should be sub-mm for short spans");
    }

    #[test]
    fn validate_connectors_flags_small() {
        let result = MotorBridgeResult {
            framework_vertices: vec![],
            framework_indices: vec![],
            pontic_vertices: vec![],
            connector_areas: vec![4.0], // below 9.0 minimum
            metrics: BridgeMetrics {
                total_span_length: 10.0,
                min_connector_area: 4.0,
                max_deflection_est: 0.01,
                framework_volume: 50.0,
            },
        };
        let issues = validate_connectors(&result, &MotorBridgeParams::default());
        assert!(!issues.is_empty());
    }

    #[test]
    fn pontic_types_different_sizes() {
        let center = Point3::new(0.0, 0.0, 0.0);
        let bullet = generate_pontic(&center, &PonticType::Bullet, &[]);
        let conical = generate_pontic(&center, &PonticType::Conical, &[]);
        // Bullet is wider than conical
        let bullet_r = bullet.iter().map(|p| (p - center).norm()).fold(0.0f64, f64::max);
        let conical_r = conical.iter().map(|p| (p - center).norm()).fold(0.0f64, f64::max);
        assert!(bullet_r > conical_r);
    }
}
