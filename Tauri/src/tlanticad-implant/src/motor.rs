//! S161-S165: Motor Implant — automated implant placement & guide generation.
//!
//! Optimal implant positioning, bone density analysis, surgical guide design,
//! abutment selection, and implant-prosthetic workflow.

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Bone quality classification (Lekholm & Zarb).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BoneQuality {
    TypeI,   // dense cortical
    TypeII,  // thick cortical, dense trabecular
    TypeIII, // thin cortical, dense trabecular
    TypeIV,  // thin cortical, sparse trabecular
}

/// Implant placement parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacementParams {
    pub target_fdi: u8,
    pub implant_diameter: f64,
    pub implant_length: f64,
    pub min_bone_around: f64,
    pub min_distance_to_nerve: f64,
    pub prosthetic_axis: Vector3<f64>,
    pub depth_below_crest: f64,
}

impl Default for PlacementParams {
    fn default() -> Self {
        Self {
            target_fdi: 36,
            implant_diameter: 4.0,
            implant_length: 10.0,
            min_bone_around: 1.5,
            min_distance_to_nerve: 2.0,
            prosthetic_axis: Vector3::new(0.0, 0.0, -1.0),
            depth_below_crest: 0.5,
        }
    }
}

/// Result of optimal placement computation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacementResult {
    pub position: [f64; 3],
    pub axis: [f64; 3],
    pub depth: f64,
    pub bone_quality: BoneQuality,
    pub available_bone: f64,
    pub nerve_clearance: f64,
    pub angulation_to_prosthetic: f64,
    pub warnings: Vec<String>,
}

/// Surgical guide sleeve descriptor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuideSleeve {
    pub position: Point3<f64>,
    pub axis: Vector3<f64>,
    pub inner_diameter: f64,
    pub outer_diameter: f64,
    pub height: f64,
}

/// Surgical guide result.
#[derive(Debug, Clone)]
pub struct SurgicalGuide {
    pub guide_vertices: Vec<Point3<f64>>,
    pub guide_indices: Vec<[u32; 3]>,
    pub sleeves: Vec<GuideSleeve>,
    pub retention_depth: f64,
}

// ---------------------------------------------------------------------------
// Optimal placement (S161-S162)
// ---------------------------------------------------------------------------

/// Compute optimal implant position given bone volume and constraints.
pub fn compute_optimal_placement(
    bone_vertices: &[Point3<f64>],
    bone_normals: &[Vector3<f64>],
    crest_points: &[Point3<f64>],
    nerve_canal: Option<&[Point3<f64>]>,
    params: &PlacementParams,
) -> PlacementResult {
    let mut warnings = Vec::new();

    // Compute crest center
    let crest_center = if !crest_points.is_empty() {
        let sum: Vector3<f64> = crest_points.iter().map(|p| p.coords).sum();
        Point3::from(sum / crest_points.len() as f64)
    } else {
        Point3::origin()
    };

    // Determine bone quality from density (simplified: use normal alignment)
    let quality = estimate_bone_quality(bone_normals);

    // Available bone height
    let available_bone = if !bone_vertices.is_empty() {
        let min_z = bone_vertices.iter().map(|v| v.z).fold(f64::MAX, f64::min);
        let max_z = bone_vertices.iter().map(|v| v.z).fold(f64::MIN, f64::max);
        max_z - min_z
    } else {
        0.0
    };

    if available_bone < params.implant_length {
        warnings.push(format!(
            "Available bone {:.1} mm < implant length {:.1} mm",
            available_bone, params.implant_length
        ));
    }

    // Nerve clearance
    let nerve_clearance = if let Some(canal) = nerve_canal {
        canal
            .iter()
            .map(|p| (p - crest_center).norm())
            .fold(f64::MAX, f64::min)
    } else {
        f64::MAX
    };

    if nerve_clearance < params.min_distance_to_nerve + params.implant_length {
        warnings.push(format!(
            "Nerve clearance {:.1} mm may be insufficient",
            nerve_clearance
        ));
    }

    // Optimal axis: blend between bone axis and prosthetic axis
    let bone_axis = estimate_bone_axis(bone_normals);
    let blend = (bone_axis + params.prosthetic_axis).normalize();
    let angulation = bone_axis
        .dot(&params.prosthetic_axis)
        .clamp(-1.0, 1.0)
        .acos()
        .to_degrees();

    if angulation > 25.0 {
        warnings.push(format!(
            "Angulation {:.1}° exceeds 25° — consider angled abutment",
            angulation
        ));
    }

    PlacementResult {
        position: [crest_center.x, crest_center.y, crest_center.z - params.depth_below_crest],
        axis: [blend.x, blend.y, blend.z],
        depth: params.depth_below_crest,
        bone_quality: quality,
        available_bone,
        nerve_clearance: nerve_clearance.min(100.0),
        angulation_to_prosthetic: angulation,
        warnings,
    }
}

// ---------------------------------------------------------------------------
// Surgical guide (S163-S164)
// ---------------------------------------------------------------------------

/// Generate a surgical guide shell from arch scan and placement results.
pub fn generate_surgical_guide(
    arch_vertices: &[Point3<f64>],
    arch_indices: &[[u32; 3]],
    placements: &[PlacementResult],
    guide_thickness: f64,
) -> SurgicalGuide {
    // Offset arch outward to create guide shell
    let guide_vertices: Vec<Point3<f64>> = arch_vertices
        .iter()
        .map(|v| Point3::new(v.x, v.y, v.z + guide_thickness))
        .collect();

    let sleeves: Vec<GuideSleeve> = placements
        .iter()
        .map(|p| GuideSleeve {
            position: Point3::new(p.position[0], p.position[1], p.position[2]),
            axis: Vector3::new(p.axis[0], p.axis[1], p.axis[2]),
            inner_diameter: 4.0,  // standard drill diameter
            outer_diameter: 6.0,
            height: 5.0,
        })
        .collect();

    SurgicalGuide {
        guide_vertices,
        guide_indices: arch_indices.to_vec(),
        sleeves,
        retention_depth: 3.0,
    }
}

// ---------------------------------------------------------------------------
// Bone analysis helpers (S165)
// ---------------------------------------------------------------------------

fn estimate_bone_quality(normals: &[Vector3<f64>]) -> BoneQuality {
    if normals.is_empty() {
        return BoneQuality::TypeIII;
    }
    // Simplified: consistency of normals → dense bone
    let avg: Vector3<f64> = normals.iter().sum::<Vector3<f64>>() / normals.len() as f64;
    let consistency: f64 = normals
        .iter()
        .map(|n| n.normalize().dot(&avg.normalize()).abs())
        .sum::<f64>()
        / normals.len() as f64;

    if consistency > 0.9 { BoneQuality::TypeI }
    else if consistency > 0.7 { BoneQuality::TypeII }
    else if consistency > 0.5 { BoneQuality::TypeIII }
    else { BoneQuality::TypeIV }
}

fn estimate_bone_axis(normals: &[Vector3<f64>]) -> Vector3<f64> {
    if normals.is_empty() {
        return Vector3::new(0.0, 0.0, -1.0);
    }
    let avg: Vector3<f64> = normals.iter().sum::<Vector3<f64>>();
    if avg.norm() < 1e-12 {
        Vector3::new(0.0, 0.0, -1.0)
    } else {
        avg.normalize()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optimal_placement_basic() {
        let bone_v = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(5.0, 0.0, 0.0),
            Point3::new(2.5, 5.0, 0.0),
            Point3::new(2.5, 2.5, -15.0),
        ];
        let bone_n = vec![Vector3::new(0.0, 0.0, -1.0); 4];
        let crest = vec![Point3::new(2.5, 2.5, 0.0)];
        let params = PlacementParams::default();
        let result = compute_optimal_placement(&bone_v, &bone_n, &crest, None, &params);
        assert!(result.available_bone > 0.0);
        assert!(result.nerve_clearance > 0.0);
    }

    #[test]
    fn placement_warns_short_bone() {
        let bone_v = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, -3.0), // only 3mm bone
        ];
        let bone_n = vec![Vector3::z(); 2];
        let crest = vec![Point3::origin()];
        let params = PlacementParams { implant_length: 10.0, ..Default::default() };
        let result = compute_optimal_placement(&bone_v, &bone_n, &crest, None, &params);
        assert!(result.warnings.iter().any(|w| w.contains("Available bone")));
    }

    #[test]
    fn surgical_guide_creates_sleeves() {
        let arch_v = vec![Point3::origin(); 4];
        let arch_i = vec![[0, 1, 2], [1, 2, 3]];
        let placement = PlacementResult {
            position: [0.0, 0.0, -1.0],
            axis: [0.0, 0.0, -1.0],
            depth: 1.0,
            bone_quality: BoneQuality::TypeII,
            available_bone: 12.0,
            nerve_clearance: 5.0,
            angulation_to_prosthetic: 5.0,
            warnings: vec![],
        };
        let guide = generate_surgical_guide(&arch_v, &arch_i, &[placement], 2.0);
        assert_eq!(guide.sleeves.len(), 1);
        assert_eq!(guide.guide_vertices.len(), 4);
    }

    #[test]
    fn bone_quality_classification() {
        // All normals same direction → TypeI
        let normals = vec![Vector3::new(0.0, 0.0, -1.0); 20];
        let q = estimate_bone_quality(&normals);
        assert_eq!(q, BoneQuality::TypeI);
    }

    #[test]
    fn empty_bone_defaults() {
        let result = compute_optimal_placement(&[], &[], &[], None, &PlacementParams::default());
        assert_eq!(result.available_bone, 0.0);
    }
}
