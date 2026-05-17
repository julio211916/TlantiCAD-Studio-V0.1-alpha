//! S116-S120: Model creation — die cutting, base generation, articulation setup.
//!
//! Creates working models from segmented scans: isolate individual tooth dies,
//! generate model bases, and set up articulator parameters.

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};
use crate::segmentation::SegmentLabel;
use crate::scan_import::RawScan;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A single tooth die extracted from the working model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DieCut {
    pub fdi_number: u8,
    pub vertices: Vec<Point3<f64>>,
    pub indices: Vec<[u32; 3]>,
    pub normals: Vec<Vector3<f64>>,
    /// Margin line indices (indices into `vertices`).
    pub margin_indices: Vec<u32>,
    /// Insertion axis for this die.
    pub insertion_axis: Vector3<f64>,
}

/// Parameters for base generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseParams {
    /// Height of the base in mm.
    pub height: f64,
    /// Offset from the arch outline in mm.
    pub offset: f64,
    /// Whether to add a horseshoe cutout.
    pub horseshoe: bool,
    /// Number of points in the base outline.
    pub outline_resolution: usize,
}

impl Default for BaseParams {
    fn default() -> Self {
        Self {
            height: 15.0,
            offset: 2.0,
            horseshoe: true,
            outline_resolution: 64,
        }
    }
}

/// Generated model base.
#[derive(Debug, Clone)]
pub struct ModelBase {
    pub outline: Vec<Point3<f64>>,
    pub vertices: Vec<Point3<f64>>,
    pub indices: Vec<[u32; 3]>,
    pub height: f64,
}

/// Articulator settings for model mounting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticulatorParams {
    /// Inter-condylar distance (mm).
    pub condylar_distance: f64,
    /// Bennett angle (degrees).
    pub bennett_angle: f64,
    /// Condylar inclination (degrees).
    pub condylar_inclination: f64,
    /// Incisal guidance angle (degrees).
    pub incisal_guidance: f64,
}

impl Default for ArticulatorParams {
    fn default() -> Self {
        Self {
            condylar_distance: 110.0,
            bennett_angle: 15.0,
            condylar_inclination: 30.0,
            incisal_guidance: 10.0,
        }
    }
}

/// Result of articulator setup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticulatorSetup {
    pub params: ArticulatorParams,
    /// Face-bow transfer rotation (Euler angles in degrees).
    pub facebow_rotation: [f64; 3],
    /// Vertical dimension of occlusion (mm).
    pub vertical_dimension: f64,
}

// ---------------------------------------------------------------------------
// Die cutting (S116-S117)
// ---------------------------------------------------------------------------

/// Extract individual tooth dies from a segmented scan.
pub fn cut_dies(
    scan: &RawScan,
    vertex_labels: &[SegmentLabel],
) -> Vec<DieCut> {
    // Collect unique tooth FDI numbers
    let mut fdi_set = std::collections::BTreeSet::new();
    for label in vertex_labels {
        if let SegmentLabel::Tooth(fdi) | SegmentLabel::Preparation(fdi) = label {
            fdi_set.insert(*fdi);
        }
    }

    fdi_set
        .into_iter()
        .filter_map(|fdi| {
            // Collect vertex indices belonging to this tooth
            let vert_indices: Vec<usize> = vertex_labels
                .iter()
                .enumerate()
                .filter(|(_, l)| matches!(l, SegmentLabel::Tooth(t) | SegmentLabel::Preparation(t) if *t == fdi))
                .map(|(i, _)| i)
                .collect();

            if vert_indices.is_empty() {
                return None;
            }

            // Remap vertices
            let mut remap = vec![u32::MAX; scan.vertices.len()];
            let mut new_verts = Vec::new();
            let mut new_normals = Vec::new();
            for &vi in &vert_indices {
                remap[vi] = new_verts.len() as u32;
                new_verts.push(scan.vertices[vi]);
                if vi < scan.normals.len() {
                    new_normals.push(scan.normals[vi]);
                }
            }

            // Remap triangles
            let new_indices: Vec<[u32; 3]> = scan
                .indices
                .iter()
                .filter_map(|tri| {
                    let a = remap[tri[0] as usize];
                    let b = remap[tri[1] as usize];
                    let c = remap[tri[2] as usize];
                    if a != u32::MAX && b != u32::MAX && c != u32::MAX {
                        Some([a, b, c])
                    } else {
                        None
                    }
                })
                .collect();

            Some(DieCut {
                fdi_number: fdi,
                vertices: new_verts,
                indices: new_indices,
                normals: new_normals,
                margin_indices: vec![],
                insertion_axis: Vector3::new(0.0, 0.0, 1.0),
            })
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Base generation (S118)
// ---------------------------------------------------------------------------

/// Generate a flat base outline from the arch boundary vertices.
pub fn generate_base_outline(
    arch_vertices: &[Point3<f64>],
    params: &BaseParams,
) -> Vec<Point3<f64>> {
    if arch_vertices.is_empty() {
        return vec![];
    }

    // Project to XY plane, find convex hull-like outline
    let sum: Vector3<f64> = arch_vertices.iter().map(|p| p.coords).sum();
    let center = sum / arch_vertices.len() as f64;

    // Generate outline as an ellipse around the arch center
    let mut max_r = 0.0f64;
    for v in arch_vertices {
        let dx = v.x - center.x;
        let dy = v.y - center.y;
        let r = (dx * dx + dy * dy).sqrt();
        max_r = max_r.max(r);
    }

    let r = max_r + params.offset;
    let n = params.outline_resolution;
    (0..n)
        .map(|i| {
            let angle = std::f64::consts::TAU * (i as f64) / (n as f64);
            Point3::new(
                center.x + r * angle.cos(),
                center.y + r * angle.sin(),
                center.z - params.height,
            )
        })
        .collect()
}

/// Create the full base mesh (outline extruded to a plate).
pub fn create_base_mesh(
    outline: &[Point3<f64>],
    height: f64,
) -> ModelBase {
    if outline.is_empty() {
        return ModelBase {
            outline: vec![],
            vertices: vec![],
            indices: vec![],
            height,
        };
    }

    let n = outline.len();
    let mut vertices = Vec::with_capacity(n * 2);
    let mut indices = Vec::new();

    // Bottom ring
    for p in outline {
        vertices.push(*p);
    }
    // Top ring (at z + height)
    for p in outline {
        vertices.push(Point3::new(p.x, p.y, p.z + height));
    }

    // Side faces
    for i in 0..n {
        let next = (i + 1) % n;
        let b0 = i as u32;
        let b1 = next as u32;
        let t0 = (i + n) as u32;
        let t1 = (next + n) as u32;
        indices.push([b0, b1, t1]);
        indices.push([b0, t1, t0]);
    }

    ModelBase {
        outline: outline.to_vec(),
        vertices,
        indices,
        height,
    }
}

// ---------------------------------------------------------------------------
// Articulator setup (S119-S120)
// ---------------------------------------------------------------------------

/// Set up articulator from face-bow transfer measurements.
pub fn setup_articulator(
    params: ArticulatorParams,
    upper_model_center: &Point3<f64>,
    lower_model_center: &Point3<f64>,
) -> ArticulatorSetup {
    let vd = (upper_model_center - lower_model_center).norm();

    // Estimate facebow rotation from model centers
    let axis = (upper_model_center - lower_model_center).normalize();
    let facebow_rotation = [
        axis.x.atan2(axis.z).to_degrees(),
        axis.y.atan2(axis.z).to_degrees(),
        0.0,
    ];

    ArticulatorSetup {
        params,
        facebow_rotation,
        vertical_dimension: vd,
    }
}

/// Compute hinge axis point from condylar distance and inclination.
pub fn hinge_axis_point(
    params: &ArticulatorParams,
    side: f64, // -1.0 for left, 1.0 for right
) -> Point3<f64> {
    let half_dist = params.condylar_distance / 2.0;
    Point3::new(
        side * half_dist,
        0.0,
        0.0,
    )
}

/// Simulate opening rotation around hinge axis.
pub fn simulate_opening(
    lower_vertices: &[Point3<f64>],
    hinge_left: &Point3<f64>,
    hinge_right: &Point3<f64>,
    angle_degrees: f64,
) -> Vec<Point3<f64>> {
    let hinge_center = Point3::new(
        (hinge_left.x + hinge_right.x) / 2.0,
        (hinge_left.y + hinge_right.y) / 2.0,
        (hinge_left.z + hinge_right.z) / 2.0,
    );
    let hinge_axis = (hinge_right - hinge_left).normalize();
    let angle = angle_degrees.to_radians();

    lower_vertices
        .iter()
        .map(|v| {
            let relative = v - hinge_center;
            let parallel = hinge_axis * relative.dot(&hinge_axis);
            let perpendicular = relative - parallel;
            let perp_len = perpendicular.norm();
            if perp_len < 1e-12 {
                return *v;
            }
            let perp_unit = perpendicular / perp_len;
            let tangent = hinge_axis.cross(&perp_unit);
            let rotated = perp_unit * angle.cos() + tangent * angle.sin();
            hinge_center + parallel + rotated * perp_len
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_labeled_scan() -> (RawScan, Vec<SegmentLabel>) {
        let scan = RawScan {
            vertices: vec![
                Point3::new(0.0, 0.0, 0.0),
                Point3::new(1.0, 0.0, 0.0),
                Point3::new(0.0, 1.0, 0.0),
                Point3::new(2.0, 0.0, 0.0),
                Point3::new(2.0, 1.0, 0.0),
            ],
            normals: vec![Vector3::z(); 5],
            indices: vec![[0, 1, 2], [1, 3, 4]],
            format: crate::scan_import::ScanFormat::Stl,
        };
        let labels = vec![
            SegmentLabel::Tooth(11),
            SegmentLabel::Tooth(11),
            SegmentLabel::Tooth(11),
            SegmentLabel::Tooth(12),
            SegmentLabel::Tooth(12),
        ];
        (scan, labels)
    }

    #[test]
    fn cut_dies_separates_teeth() {
        let (scan, labels) = make_labeled_scan();
        let dies = cut_dies(&scan, &labels);
        assert_eq!(dies.len(), 2);
        let fdi_nums: Vec<u8> = dies.iter().map(|d| d.fdi_number).collect();
        assert!(fdi_nums.contains(&11));
        assert!(fdi_nums.contains(&12));
    }

    #[test]
    fn cut_dies_vertex_counts() {
        let (scan, labels) = make_labeled_scan();
        let dies = cut_dies(&scan, &labels);
        let die_11 = dies.iter().find(|d| d.fdi_number == 11).unwrap();
        assert_eq!(die_11.vertices.len(), 3);
        let die_12 = dies.iter().find(|d| d.fdi_number == 12).unwrap();
        assert_eq!(die_12.vertices.len(), 2);
    }

    #[test]
    fn generate_base_outline_circle() {
        let arch = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(10.0, 0.0, 0.0),
            Point3::new(0.0, 10.0, 0.0),
        ];
        let params = BaseParams { outline_resolution: 32, ..Default::default() };
        let outline = generate_base_outline(&arch, &params);
        assert_eq!(outline.len(), 32);
    }

    #[test]
    fn create_base_mesh_geometry() {
        let outline = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(1.0, 1.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        ];
        let base = create_base_mesh(&outline, 10.0);
        assert_eq!(base.vertices.len(), 8); // 4 bottom + 4 top
        assert_eq!(base.indices.len(), 8); // 4 sides × 2 triangles
    }

    #[test]
    fn articulator_setup_defaults() {
        let params = ArticulatorParams::default();
        let upper = Point3::new(0.0, 0.0, 10.0);
        let lower = Point3::new(0.0, 0.0, 0.0);
        let setup = setup_articulator(params, &upper, &lower);
        assert!(setup.vertical_dimension > 0.0);
        assert!(setup.params.condylar_distance > 0.0);
    }

    #[test]
    fn hinge_axis_symmetric() {
        let params = ArticulatorParams::default();
        let left = hinge_axis_point(&params, -1.0);
        let right = hinge_axis_point(&params, 1.0);
        assert!((left.x + right.x).abs() < 1e-9);
    }

    #[test]
    fn opening_zero_angle_identity() {
        let verts = vec![
            Point3::new(0.0, 0.0, -5.0),
            Point3::new(1.0, 0.0, -5.0),
        ];
        let hl = Point3::new(-55.0, 0.0, 0.0);
        let hr = Point3::new(55.0, 0.0, 0.0);
        let result = simulate_opening(&verts, &hl, &hr, 0.0);
        for (orig, rotated) in verts.iter().zip(&result) {
            assert!((orig - rotated).norm() < 1e-9);
        }
    }

    #[test]
    fn cut_dies_empty_labels() {
        let scan = RawScan {
            vertices: vec![Point3::origin()],
            normals: vec![Vector3::z()],
            indices: vec![],
            format: crate::scan_import::ScanFormat::Stl,
        };
        let labels = vec![SegmentLabel::Gingiva];
        let dies = cut_dies(&scan, &labels);
        assert!(dies.is_empty());
    }
}
