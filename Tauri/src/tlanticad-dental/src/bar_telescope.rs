//! S181-S185: Motor Bar / Telescope / Attachment — removable prosthetic mechanisms.
//!
//! Bar design, telescope coping (primary/secondary), and attachment
//! (ball, locator, ERA) for implant-retained or tooth-supported prostheses.

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Bar cross-section shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BarProfile {
    Round,
    Oval,
    Dolder,       // egg-shaped
    HaderClip,    // keyhole
    Rectangular,
}

/// Attachment type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AttachmentType {
    Ball,
    Locator,
    Era,
    DalboPlus,
    PressFit,
}

/// Telescope type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TelescopeType {
    /// Parallel-walled (friction)
    Parallel,
    /// Conical (frictional retention)
    Conical,
    /// Milled (precision attachment)
    Milled,
}

/// Parameters for bar design.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarParams {
    pub profile: BarProfile,
    pub width: f64,
    pub height: f64,
    pub clearance_to_tissue: f64,
    pub abutment_positions: Vec<[f64; 3]>,
}

impl Default for BarParams {
    fn default() -> Self {
        Self {
            profile: BarProfile::Dolder,
            width: 3.0,
            height: 3.0,
            clearance_to_tissue: 1.0,
            abutment_positions: vec![[-10.0, 0.0, 0.0], [10.0, 0.0, 0.0]],
        }
    }
}

/// Parameters for telescope coping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelescopeParams {
    pub telescope_type: TelescopeType,
    pub primary_taper_degrees: f64,
    pub primary_wall_thickness: f64,
    pub secondary_gap: f64,
    pub secondary_wall_thickness: f64,
    pub friction_height: f64,
}

impl Default for TelescopeParams {
    fn default() -> Self {
        Self {
            telescope_type: TelescopeType::Conical,
            primary_taper_degrees: 6.0,
            primary_wall_thickness: 0.4,
            secondary_gap: 0.02,
            secondary_wall_thickness: 0.5,
            friction_height: 5.0,
        }
    }
}

/// Bar generation result.
#[derive(Debug, Clone)]
pub struct BarResult {
    pub centerline: Vec<Point3<f64>>,
    pub profile_vertices: Vec<Point3<f64>>,
    pub profile_indices: Vec<[u32; 3]>,
    pub total_length: f64,
    pub cross_section_area: f64,
}

/// Telescope pair result.
#[derive(Debug, Clone)]
pub struct TelescopePair {
    pub primary_vertices: Vec<Point3<f64>>,
    pub primary_indices: Vec<[u32; 3]>,
    pub secondary_vertices: Vec<Point3<f64>>,
    pub secondary_indices: Vec<[u32; 3]>,
    pub retention_force_est: f64,
}

// ---------------------------------------------------------------------------
// Bar design (S181-S182)
// ---------------------------------------------------------------------------

/// Generate a bar connecting multiple abutments.
pub fn generate_bar(params: &BarParams) -> BarResult {
    let positions: Vec<Point3<f64>> = params
        .abutment_positions
        .iter()
        .map(|p| Point3::new(p[0], p[1], p[2]))
        .collect();

    if positions.len() < 2 {
        return BarResult {
            centerline: positions,
            profile_vertices: vec![],
            profile_indices: vec![],
            total_length: 0.0,
            cross_section_area: 0.0,
        };
    }

    // Generate centerline by interpolating between abutments
    let mut centerline = Vec::new();
    let mut total_length = 0.0;
    for i in 0..positions.len() - 1 {
        let span = (positions[i + 1] - positions[i]).norm();
        total_length += span;
        let steps = (span / 0.5).ceil() as usize;
        for s in 0..steps {
            let t = s as f64 / steps as f64;
            centerline.push(Point3::from(
                positions[i].coords * (1.0 - t) + positions[i + 1].coords * t,
            ));
        }
    }
    centerline.push(*positions.last().unwrap());

    // Cross-section area
    let area = match params.profile {
        BarProfile::Round => std::f64::consts::PI * (params.width / 2.0).powi(2),
        BarProfile::Oval | BarProfile::Dolder => std::f64::consts::PI * params.width / 2.0 * params.height / 2.0,
        BarProfile::HaderClip | BarProfile::Rectangular => params.width * params.height,
    };

    // Profile vertices: sweep cross-section along centerline
    let mut profile_verts = Vec::new();
    let n_ring = 8;
    for center in &centerline {
        for i in 0..n_ring {
            let angle = std::f64::consts::TAU * i as f64 / n_ring as f64;
            let rx = params.width / 2.0;
            let ry = params.height / 2.0;
            profile_verts.push(Point3::new(
                center.x + rx * angle.cos(),
                center.y + ry * angle.sin(),
                center.z + params.clearance_to_tissue,
            ));
        }
    }

    BarResult {
        centerline,
        profile_vertices: profile_verts,
        profile_indices: vec![], // Would generate ring-to-ring quads
        total_length,
        cross_section_area: area,
    }
}

// ---------------------------------------------------------------------------
// Telescope (S183-S184)
// ---------------------------------------------------------------------------

/// Generate a telescope pair (primary + secondary coping).
pub fn generate_telescope(
    abutment_vertices: &[Point3<f64>],
    abutment_normals: &[Vector3<f64>],
    params: &TelescopeParams,
) -> TelescopePair {
    if abutment_vertices.is_empty() {
        return TelescopePair {
            primary_vertices: vec![],
            primary_indices: vec![],
            secondary_vertices: vec![],
            secondary_indices: vec![],
            retention_force_est: 0.0,
        };
    }

    // Primary: offset inward by wall thickness
    let primary: Vec<Point3<f64>> = abutment_vertices
        .iter()
        .zip(abutment_normals.iter())
        .map(|(v, n)| v - n.normalize() * params.primary_wall_thickness)
        .collect();

    // Secondary: offset outward by gap + wall
    let total_offset = params.secondary_gap + params.secondary_wall_thickness;
    let secondary: Vec<Point3<f64>> = abutment_vertices
        .iter()
        .zip(abutment_normals.iter())
        .map(|(v, n)| v + n.normalize() * total_offset)
        .collect();

    // Estimated retention force (simplified: proportional to friction area)
    let retention = match params.telescope_type {
        TelescopeType::Parallel => params.friction_height * 4.0 * std::f64::consts::PI, // higher for parallel
        TelescopeType::Conical => params.friction_height * 3.0,
        TelescopeType::Milled => params.friction_height * 5.0,
    };

    TelescopePair {
        primary_vertices: primary,
        primary_indices: vec![],
        secondary_vertices: secondary,
        secondary_indices: vec![],
        retention_force_est: retention,
    }
}

// ---------------------------------------------------------------------------
// Attachment fitting (S185)
// ---------------------------------------------------------------------------

/// Check attachment clearance and retention for selected type.
pub fn validate_attachment(
    attachment_type: AttachmentType,
    available_height: f64,
    available_width: f64,
) -> (bool, Vec<String>) {
    let mut issues = Vec::new();
    let (min_h, min_w) = match attachment_type {
        AttachmentType::Ball => (4.0, 3.5),
        AttachmentType::Locator => (3.5, 5.0),
        AttachmentType::Era => (3.0, 4.0),
        AttachmentType::DalboPlus => (5.0, 4.5),
        AttachmentType::PressFit => (4.0, 4.0),
    };

    if available_height < min_h {
        issues.push(format!(
            "Height {:.1} mm < minimum {:.1} mm for {:?}",
            available_height, min_h, attachment_type
        ));
    }
    if available_width < min_w {
        issues.push(format!(
            "Width {:.1} mm < minimum {:.1} mm for {:?}",
            available_width, min_w, attachment_type
        ));
    }

    (issues.is_empty(), issues)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bar_generation_basic() {
        let params = BarParams::default();
        let result = generate_bar(&params);
        assert!(result.total_length > 0.0);
        assert!(result.cross_section_area > 0.0);
        assert!(!result.centerline.is_empty());
    }

    #[test]
    fn bar_single_abutment() {
        let params = BarParams {
            abutment_positions: vec![[0.0, 0.0, 0.0]],
            ..Default::default()
        };
        let result = generate_bar(&params);
        assert_eq!(result.total_length, 0.0);
    }

    #[test]
    fn telescope_generates_pair() {
        let verts = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.5, 1.0, 0.0),
        ];
        let normals = vec![Vector3::z(); 3];
        let params = TelescopeParams::default();
        let pair = generate_telescope(&verts, &normals, &params);
        assert_eq!(pair.primary_vertices.len(), 3);
        assert_eq!(pair.secondary_vertices.len(), 3);
        assert!(pair.retention_force_est > 0.0);
    }

    #[test]
    fn attachment_validates_height() {
        let (ok, _) = validate_attachment(AttachmentType::Ball, 5.0, 5.0);
        assert!(ok);
        let (ok, issues) = validate_attachment(AttachmentType::Ball, 2.0, 2.0);
        assert!(!ok);
        assert!(!issues.is_empty());
    }

    #[test]
    fn bar_profiles_different_areas() {
        let round = BarParams { profile: BarProfile::Round, ..Default::default() };
        let rect = BarParams { profile: BarProfile::Rectangular, ..Default::default() };
        let r_result = generate_bar(&round);
        let q_result = generate_bar(&rect);
        // Round area < rectangular for same width/height (π < 4)
        assert!(r_result.cross_section_area < q_result.cross_section_area);
    }
}
