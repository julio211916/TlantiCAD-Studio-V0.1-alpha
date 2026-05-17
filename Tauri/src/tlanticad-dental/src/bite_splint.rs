//! S186-S190: Motor Bite Splint / Freeform — splint design and freeform sculpting.
//!
//! Night guard / occlusal splint generation, surgical guide adaptation,
//! retainer design, and freeform mesh sculpting tools.

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Splint type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SplintType {
    NightGuard,
    MichiganSplint,
    Deprogrammer,
    SurgicalGuide,
    Retainer,
    Bleaching,
}

/// Splint coverage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SplintCoverage {
    FullArch,
    Anterior,
    Posterior,
    SingleTooth,
}

/// Parameters for splint generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplintParams {
    pub splint_type: SplintType,
    pub coverage: SplintCoverage,
    pub occlusal_thickness: f64,
    pub wall_thickness: f64,
    pub undercut_blockout: f64,
    pub labial_height: f64,
    pub lingual_height: f64,
    pub flat_plane: bool,
}

impl Default for SplintParams {
    fn default() -> Self {
        Self {
            splint_type: SplintType::NightGuard,
            coverage: SplintCoverage::FullArch,
            occlusal_thickness: 2.0,
            wall_thickness: 1.5,
            undercut_blockout: 0.5,
            labial_height: 3.0,
            lingual_height: 4.0,
            flat_plane: false,
        }
    }
}

/// Splint result.
#[derive(Debug, Clone)]
pub struct SplintResult {
    pub outer_vertices: Vec<Point3<f64>>,
    pub inner_vertices: Vec<Point3<f64>>,
    pub indices: Vec<[u32; 3]>,
    pub occlusal_flat_plane: Option<f64>, // height of flat plane if applicable
    pub metrics: SplintMetrics,
}

/// Splint quality metrics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SplintMetrics {
    pub min_thickness: f64,
    pub max_thickness: f64,
    pub coverage_percent: f64,
    pub undercut_count: u32,
    pub warnings: Vec<String>,
}

// ---------------------------------------------------------------------------
// Freeform sculpting types
// ---------------------------------------------------------------------------

/// Sculpting tool enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SculptTool {
    Push,
    Pull,
    Smooth,
    Flatten,
    Pinch,
    Inflate,
}

/// Single sculpting stroke.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SculptStroke {
    pub tool: SculptTool,
    pub center: [f64; 3],
    pub radius: f64,
    pub strength: f64,
    pub direction: [f64; 3],
}

// ---------------------------------------------------------------------------
// Splint generation (S186-S188)
// ---------------------------------------------------------------------------

/// Generate a bite splint from the archscan.
pub fn generate_splint(
    arch_vertices: &[Point3<f64>],
    arch_normals: &[Vector3<f64>],
    arch_indices: &[[u32; 3]],
    params: &SplintParams,
) -> SplintResult {
    if arch_vertices.is_empty() {
        return SplintResult {
            outer_vertices: vec![],
            inner_vertices: vec![],
            indices: vec![],
            occlusal_flat_plane: None,
            metrics: SplintMetrics::default(),
        };
    }

    // Inner surface: slight expansion for fit + blockout
    let gap = params.undercut_blockout;
    let inner: Vec<Point3<f64>> = arch_vertices
        .iter()
        .zip(arch_normals.iter())
        .map(|(v, n)| v + n.normalize() * gap)
        .collect();

    // Outer surface: offset by occlusal thickness
    let outer: Vec<Point3<f64>> = inner
        .iter()
        .zip(arch_normals.iter())
        .map(|(v, n)| v + n.normalize() * params.occlusal_thickness)
        .collect();

    // Flat plane for Michigan splint
    let flat_plane = if params.flat_plane {
        let max_z = outer.iter().map(|v| v.z).fold(f64::MIN, f64::max);
        Some(max_z)
    } else {
        None
    };

    let outer = if params.flat_plane {
        let plane_h = flat_plane.unwrap();
        outer.into_iter().map(|mut v| { v.z = v.z.max(plane_h - params.occlusal_thickness * 0.1); v }).collect()
    } else {
        outer
    };

    // Metrics
    let thicknesses: Vec<f64> = inner
        .iter()
        .zip(outer.iter())
        .map(|(i, o)| (o - i).norm())
        .collect();
    let min_t = thicknesses.iter().cloned().fold(f64::MAX, f64::min);
    let max_t = thicknesses.iter().cloned().fold(0.0f64, f64::max);

    SplintResult {
        outer_vertices: outer,
        inner_vertices: inner,
        indices: arch_indices.to_vec(),
        occlusal_flat_plane: flat_plane,
        metrics: SplintMetrics {
            min_thickness: min_t,
            max_thickness: max_t,
            coverage_percent: 100.0, // Full coverage mode
            undercut_count: 0,
            warnings: vec![],
        },
    }
}

// ---------------------------------------------------------------------------
// Freeform sculpting (S189-S190)
// ---------------------------------------------------------------------------

/// Apply a freeform sculpt stroke to a mesh.
pub fn apply_sculpt_stroke(
    vertices: &mut [Point3<f64>],
    normals: &[Vector3<f64>],
    stroke: &SculptStroke,
) {
    let center = Point3::new(stroke.center[0], stroke.center[1], stroke.center[2]);
    let direction = Vector3::new(stroke.direction[0], stroke.direction[1], stroke.direction[2]);
    let r2 = stroke.radius * stroke.radius;

    for (i, vertex) in vertices.iter_mut().enumerate() {
        let dist2 = (vertex.coords - center.coords).norm_squared();
        if dist2 > r2 {
            continue;
        }
        // Smooth falloff
        let falloff = 1.0 - dist2 / r2;
        let displacement = stroke.strength * falloff;

        match stroke.tool {
            SculptTool::Push => {
                *vertex -= direction.normalize() * displacement;
            }
            SculptTool::Pull => {
                *vertex += direction.normalize() * displacement;
            }
            SculptTool::Smooth => {
                // Move toward local average (simplified: toward center)
                let to_center = center - *vertex;
                *vertex += to_center * displacement * 0.1;
            }
            SculptTool::Flatten => {
                // Project toward a plane through center with given normal
                let n = direction.normalize();
                let to_center = *vertex - center;
                let proj = to_center.dot(&n);
                *vertex -= n * proj * displacement * 0.5;
            }
            SculptTool::Pinch => {
                let to_center = (center - *vertex).normalize();
                *vertex += to_center * displacement * 0.3;
            }
            SculptTool::Inflate => {
                if i < normals.len() {
                    *vertex += normals[i].normalize() * displacement;
                }
            }
        }
    }
}

/// Validate splint design.
pub fn validate_splint(result: &SplintResult, params: &SplintParams) -> Vec<String> {
    let mut issues = Vec::new();
    let m = &result.metrics;

    if m.min_thickness < params.wall_thickness * 0.5 {
        issues.push(format!(
            "Min thickness {:.2} mm — risk of fracture",
            m.min_thickness
        ));
    }

    if params.flat_plane && result.occlusal_flat_plane.is_none() {
        issues.push("Flat plane requested but not generated".into());
    }

    issues
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_arch() -> (Vec<Point3<f64>>, Vec<Vector3<f64>>, Vec<[u32; 3]>) {
        let verts: Vec<Point3<f64>> = (0..10)
            .map(|i| Point3::new(i as f64, 0.0, 0.0))
            .collect();
        let normals = vec![Vector3::z(); 10];
        let indices = (0..8).map(|i| [0, i as u32 + 1, i as u32 + 2]).collect();
        (verts, normals, indices)
    }

    #[test]
    fn generate_splint_basic() {
        let (v, n, i) = make_arch();
        let params = SplintParams::default();
        let result = generate_splint(&v, &n, &i, &params);
        assert_eq!(result.outer_vertices.len(), v.len());
        assert!(result.metrics.min_thickness > 0.0);
    }

    #[test]
    fn flat_plane_splint() {
        let (v, n, i) = make_arch();
        let params = SplintParams { flat_plane: true, splint_type: SplintType::MichiganSplint, ..Default::default() };
        let result = generate_splint(&v, &n, &i, &params);
        assert!(result.occlusal_flat_plane.is_some());
    }

    #[test]
    fn sculpt_push_moves_inward() {
        let mut verts = vec![Point3::new(0.0, 0.0, 0.0)];
        let normals = vec![Vector3::z()];
        let stroke = SculptStroke {
            tool: SculptTool::Push,
            center: [0.0, 0.0, 0.0],
            radius: 5.0,
            strength: 1.0,
            direction: [0.0, 0.0, 1.0],
        };
        apply_sculpt_stroke(&mut verts, &normals, &stroke);
        assert!(verts[0].z < 0.0);
    }

    #[test]
    fn sculpt_pull_moves_outward() {
        let mut verts = vec![Point3::new(0.0, 0.0, 0.0)];
        let normals = vec![Vector3::z()];
        let stroke = SculptStroke {
            tool: SculptTool::Pull,
            center: [0.0, 0.0, 0.0],
            radius: 5.0,
            strength: 1.0,
            direction: [0.0, 0.0, 1.0],
        };
        apply_sculpt_stroke(&mut verts, &normals, &stroke);
        assert!(verts[0].z > 0.0);
    }

    #[test]
    fn empty_splint_no_crash() {
        let result = generate_splint(&[], &[], &[], &SplintParams::default());
        assert!(result.outer_vertices.is_empty());
    }
}
