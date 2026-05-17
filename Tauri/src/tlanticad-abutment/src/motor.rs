//! S166-S170: Motor Abutment — automated custom abutment generation.
//!
//! Emergence profile computation, screw-channel optimization,
//! multi-unit abutment design, and Ti-base selection.

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};
use tlanticad_mesh::{is_watertight, Mesh};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Ti-base connection type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConnectionType {
    InternalHex,
    InternalOctagon,
    ConicalMorse,
    ExternalHex,
}

/// Material for custom abutment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AbutmentMaterial {
    Titanium,
    Zirconia,
    Hybrid, // Ti-base + zirconia
    Peek,
}

/// Parameters for motor abutment generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotorAbutmentParams {
    pub connection: ConnectionType,
    pub material: AbutmentMaterial,
    pub implant_diameter: f64,
    pub margin_height: f64,
    pub emergence_angle: f64,
    pub shoulder_width: f64,
    pub taper_degrees: f64,
    pub screw_channel_diameter: f64,
    pub gingival_height: f64,
}

impl Default for MotorAbutmentParams {
    fn default() -> Self {
        Self {
            connection: ConnectionType::InternalHex,
            material: AbutmentMaterial::Hybrid,
            implant_diameter: 4.0,
            margin_height: 1.5,
            emergence_angle: 30.0,
            shoulder_width: 0.5,
            taper_degrees: 6.0,
            screw_channel_diameter: 2.5,
            gingival_height: 3.0,
        }
    }
}

/// Generated abutment result.
#[derive(Debug, Clone)]
pub struct MotorAbutmentResult {
    pub outer_profile: Vec<Point3<f64>>,
    pub screw_channel: Vec<Point3<f64>>,
    pub emergence_profile: Vec<Point3<f64>>,
    pub total_height: f64,
    pub convergence_angle: f64,
    pub retention_area: f64,
    pub warnings: Vec<String>,
}

/// Cross-section presets ported from the inspected Abutments/designs STL set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AbutmentProfilePreset {
    Default,
    Round,
    Rectangle,
    Square,
    Shoulder,
    Clip,
}

/// Mesh-first request for the Rust abutment generator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomAbutmentMeshRequest {
    /// Closed margin loop in case/world coordinates.
    pub margin_polyline: Vec<[f64; 3]>,
    /// Implant insertion axis. The generator normalizes it.
    pub implant_axis: [f64; 3],
    /// Implant platform diameter in mm.
    pub implant_diameter_mm: f64,
    /// Total emergence body height from platform to margin in mm.
    pub emergence_height_mm: f64,
    /// Shoulder expansion near the margin in mm.
    pub shoulder_width_mm: f64,
    /// Taper angle in degrees.
    pub taper_degrees: f64,
    /// Ring count along axis. Values below 4 are clamped.
    pub axial_rings: usize,
    pub profile: AbutmentProfilePreset,
}

impl Default for CustomAbutmentMeshRequest {
    fn default() -> Self {
        Self {
            margin_polyline: Vec::new(),
            implant_axis: [0.0, 0.0, 1.0],
            implant_diameter_mm: 4.1,
            emergence_height_mm: 5.0,
            shoulder_width_mm: 0.5,
            taper_degrees: 6.0,
            axial_rings: 12,
            profile: AbutmentProfilePreset::Default,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbutmentMeshQa {
    pub vertex_count: usize,
    pub triangle_count: usize,
    pub watertight: bool,
    pub signed_volume_mm3: f64,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrewChannelPlan {
    pub points: Vec<[f64; 3]>,
    pub angle_degrees: f64,
    pub diameter_mm: f64,
    pub valid_for_library_limit: bool,
    pub warnings: Vec<String>,
}

// ---------------------------------------------------------------------------
// Emergence profile (S166-S167)
// ---------------------------------------------------------------------------

/// Compute emergence profile from implant platform to crown margin.
pub fn compute_emergence_profile(
    implant_position: &Point3<f64>,
    implant_axis: &Vector3<f64>,
    gingival_contour: &[Point3<f64>],
    params: &MotorAbutmentParams,
) -> Vec<Point3<f64>> {
    let axis = implant_axis.normalize();
    let n_levels = 12;
    let mut profile = Vec::with_capacity(n_levels * 8);

    for level in 0..n_levels {
        let t = level as f64 / (n_levels - 1) as f64;
        let height = t * params.gingival_height;
        let center = implant_position + axis * height;

        // Radius transitions from implant diameter to shoulder + emergence
        let base_r = params.implant_diameter / 2.0;
        let target_r =
            base_r + params.shoulder_width + height * params.emergence_angle.to_radians().tan();

        // Blend with gingival contour if available
        let r = if !gingival_contour.is_empty() {
            let gingival_r = gingival_contour
                .iter()
                .map(|g| ((g.x - center.x).powi(2) + (g.y - center.y).powi(2)).sqrt())
                .fold(0.0f64, f64::max);
            target_r.min(gingival_r)
        } else {
            target_r
        };

        // Generate ring at this level
        for i in 0..8 {
            let angle = std::f64::consts::TAU * i as f64 / 8.0;
            profile.push(Point3::new(
                center.x + r * angle.cos(),
                center.y + r * angle.sin(),
                center.z,
            ));
        }
    }

    profile
}

// ---------------------------------------------------------------------------
// Screw channel (S168)
// ---------------------------------------------------------------------------

/// Compute screw channel path, optionally angled.
pub fn compute_screw_channel(
    implant_position: &Point3<f64>,
    implant_axis: &Vector3<f64>,
    prosthetic_axis: &Vector3<f64>,
    params: &MotorAbutmentParams,
) -> (Vec<Point3<f64>>, f64) {
    let axis = implant_axis.normalize();
    let _prosth = prosthetic_axis.normalize();

    let angulation = axis
        .dot(&prosthetic_axis.normalize())
        .clamp(-1.0, 1.0)
        .acos()
        .to_degrees();

    // Channel follows implant axis (straight screw channel)
    let steps = 10;
    let channel: Vec<Point3<f64>> = (0..steps)
        .map(|i| {
            let t = i as f64 / (steps - 1) as f64;
            let height = t * (params.gingival_height + params.margin_height);
            implant_position + axis * height
        })
        .collect();

    (channel, angulation)
}

// ---------------------------------------------------------------------------
// Full abutment generation (S169-S170)
// ---------------------------------------------------------------------------

/// Generate a complete custom abutment.
pub fn generate_abutment(
    implant_position: &Point3<f64>,
    implant_axis: &Vector3<f64>,
    gingival_contour: &[Point3<f64>],
    prosthetic_axis: &Vector3<f64>,
    params: &MotorAbutmentParams,
) -> MotorAbutmentResult {
    let mut warnings = Vec::new();

    let emergence =
        compute_emergence_profile(implant_position, implant_axis, gingival_contour, params);
    let (channel, angulation) =
        compute_screw_channel(implant_position, implant_axis, prosthetic_axis, params);

    if angulation > 25.0 {
        warnings.push(format!(
            "Screw channel angulation {:.1}° — consider angled screw access",
            angulation
        ));
    }

    let total_height = params.gingival_height + params.margin_height;
    let retention_area = std::f64::consts::PI * params.implant_diameter * total_height * 0.5;
    let convergence = params.taper_degrees * 2.0;

    MotorAbutmentResult {
        outer_profile: emergence,
        screw_channel: channel,
        emergence_profile: vec![], // Simplified
        total_height,
        convergence_angle: convergence,
        retention_area,
        warnings,
    }
}

/// Generate a closed custom abutment mesh from an implant platform to a margin loop.
///
/// This is the Rust port target for the Blender/B4D Abutments logic:
/// - `Cross_Section` / `CURVE` -> `profile_scale`
/// - `Margins` / `Outline` -> `margin_polyline`
/// - `Collar` / `Free_Formed` -> lofted emergence body
///
/// Heavy booleans and surface shrinkwrap are separate jobs. This generator is
/// deterministic, allocation-bounded by `O(axial_rings * margin_points)`, and
/// does not touch React/Three state.
pub fn generate_custom_abutment_mesh(
    request: &CustomAbutmentMeshRequest,
) -> Result<(Mesh, AbutmentMeshQa), String> {
    if request.margin_polyline.len() < 6 {
        return Err("margin_polyline must contain at least 6 points".to_string());
    }
    if request.implant_diameter_mm <= 0.0 {
        return Err("implant_diameter_mm must be positive".to_string());
    }
    if request.emergence_height_mm <= 0.0 {
        return Err("emergence_height_mm must be positive".to_string());
    }

    let axis = normalized_or(request.implant_axis, Vector3::z());
    let margin: Vec<Point3<f64>> = request
        .margin_polyline
        .iter()
        .map(|p| Point3::new(p[0], p[1], p[2]))
        .collect();
    let margin_center = centroid(&margin);
    let base_center = margin_center - axis * request.emergence_height_mm;
    let radial = radial_vectors(&margin, &margin_center, &axis);

    let n = margin.len();
    let rings = request.axial_rings.clamp(4, 64);
    let platform_radius = request.implant_diameter_mm * 0.5;
    let shoulder = request.shoulder_width_mm.max(0.0);
    let taper = request.taper_degrees.to_radians().tan().max(0.0);

    let mut mesh = Mesh::new("custom_abutment_rust");
    mesh.vertices.reserve(rings * n + 2);
    mesh.indices.reserve((rings - 1) * n * 2 + n * 2);

    for ring in 0..rings {
        let t = ring as f64 / (rings - 1) as f64;
        let eased = smoothstep(t);
        let center = base_center + axis * (request.emergence_height_mm * t);
        for i in 0..n {
            let target = margin[i];
            let top_vec = target - margin_center;
            let base_vec = radial[i] * platform_radius;
            let axial_growth = axis * 0.0;
            let taper_offset = radial[i] * (taper * request.emergence_height_mm * t);
            let shoulder_offset = radial[i] * (shoulder * smoothstep(t));
            let profile = profile_scale(request.profile, i, n, t);
            let blended =
                base_vec * (1.0 - eased) + (top_vec + taper_offset + shoulder_offset) * eased;
            mesh.vertices
                .push(center + (blended + axial_growth) * profile);
        }
    }

    for ring in 0..(rings - 1) {
        let a0 = ring * n;
        let b0 = (ring + 1) * n;
        for i in 0..n {
            let next = (i + 1) % n;
            mesh.indices
                .push([(a0 + i) as u32, (a0 + next) as u32, (b0 + i) as u32]);
            mesh.indices
                .push([(a0 + next) as u32, (b0 + next) as u32, (b0 + i) as u32]);
        }
    }

    let bottom_center_idx = mesh.vertices.len() as u32;
    mesh.vertices.push(base_center);
    for i in 0..n {
        let next = (i + 1) % n;
        mesh.indices
            .push([bottom_center_idx, next as u32, i as u32]);
    }

    let top_center_idx = mesh.vertices.len() as u32;
    mesh.vertices.push(margin_center);
    let top_start = (rings - 1) * n;
    for i in 0..n {
        let next = (i + 1) % n;
        mesh.indices.push([
            top_center_idx,
            (top_start + i) as u32,
            (top_start + next) as u32,
        ]);
    }

    mesh.calculate_normals();
    let qa = qa_abutment_mesh(&mesh);
    Ok((mesh, qa))
}

/// Compute the screw-channel centerline and angle validation. Boolean cutting
/// consumes this path in a later manifold/CSG job.
pub fn plan_screw_channel(
    implant_position: [f64; 3],
    implant_axis: [f64; 3],
    prosthetic_axis: [f64; 3],
    length_mm: f64,
    diameter_mm: f64,
    library_angle_limit_deg: f64,
) -> ScrewChannelPlan {
    let origin = Point3::new(
        implant_position[0],
        implant_position[1],
        implant_position[2],
    );
    let axis = normalized_or(implant_axis, Vector3::z());
    let prosthetic = normalized_or(prosthetic_axis, axis);
    let angle = axis.dot(&prosthetic).clamp(-1.0, 1.0).acos().to_degrees();
    let steps = 16usize;
    let points = (0..steps)
        .map(|i| {
            let t = i as f64 / (steps - 1) as f64;
            let p = origin + axis * (length_mm.max(0.0) * t);
            [p.x, p.y, p.z]
        })
        .collect::<Vec<_>>();

    let mut warnings = Vec::new();
    if angle > library_angle_limit_deg {
        warnings.push(format!(
            "Screw channel angle {:.1}° exceeds library limit {:.1}°",
            angle, library_angle_limit_deg
        ));
    } else if angle > 20.0 {
        warnings.push(format!(
            "Screw channel angle {:.1}° is high; verify sleeve/driver clearance",
            angle
        ));
    }

    ScrewChannelPlan {
        points,
        angle_degrees: angle,
        diameter_mm,
        valid_for_library_limit: angle <= library_angle_limit_deg,
        warnings,
    }
}

pub fn qa_abutment_mesh(mesh: &Mesh) -> AbutmentMeshQa {
    let volume = signed_volume(mesh).abs();
    let watertight = is_watertight(mesh);
    let mut warnings = Vec::new();
    if !watertight {
        warnings.push("Mesh is not watertight".to_string());
    }
    if volume <= 1e-6 {
        warnings.push("Mesh volume is near zero".to_string());
    }
    AbutmentMeshQa {
        vertex_count: mesh.vertex_count(),
        triangle_count: mesh.triangle_count(),
        watertight,
        signed_volume_mm3: volume,
        warnings,
    }
}

fn centroid(points: &[Point3<f64>]) -> Point3<f64> {
    let sum = points
        .iter()
        .fold(Vector3::zeros(), |acc, p| acc + p.coords);
    Point3::from(sum / points.len() as f64)
}

fn normalized_or(value: [f64; 3], fallback: Vector3<f64>) -> Vector3<f64> {
    let v = Vector3::new(value[0], value[1], value[2]);
    if v.norm_squared() > 1e-12 {
        v.normalize()
    } else {
        fallback.normalize()
    }
}

fn radial_vectors(
    points: &[Point3<f64>],
    center: &Point3<f64>,
    axis: &Vector3<f64>,
) -> Vec<Vector3<f64>> {
    points
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let v = p - center;
            let planar = v - axis * v.dot(axis);
            if planar.norm_squared() > 1e-12 {
                planar.normalize()
            } else {
                let angle = std::f64::consts::TAU * i as f64 / points.len() as f64;
                Vector3::new(angle.cos(), angle.sin(), 0.0)
            }
        })
        .collect()
}

fn smoothstep(t: f64) -> f64 {
    let x = t.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

fn profile_scale(profile: AbutmentProfilePreset, i: usize, n: usize, t: f64) -> f64 {
    let theta = std::f64::consts::TAU * i as f64 / n as f64;
    match profile {
        AbutmentProfilePreset::Default | AbutmentProfilePreset::Round => 1.0,
        AbutmentProfilePreset::Square => {
            1.0 + 0.08 * theta.cos().abs().max(theta.sin().abs()) * smoothstep(t)
        }
        AbutmentProfilePreset::Rectangle => 1.0 + 0.12 * theta.cos().abs() * smoothstep(t),
        AbutmentProfilePreset::Shoulder => 1.0 + 0.18 * smoothstep(t),
        AbutmentProfilePreset::Clip => {
            let clip = if theta.sin() > 0.65 { -0.12 } else { 0.04 };
            1.0 + clip * smoothstep(t)
        }
    }
}

fn signed_volume(mesh: &Mesh) -> f64 {
    mesh.indices.iter().fold(0.0, |acc, tri| {
        let a = mesh.vertices[tri[0] as usize].coords;
        let b = mesh.vertices[tri[1] as usize].coords;
        let c = mesh.vertices[tri[2] as usize].coords;
        acc + a.dot(&b.cross(&c)) / 6.0
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emergence_profile_has_rings() {
        let pos = Point3::new(0.0, 0.0, 0.0);
        let axis = Vector3::new(0.0, 0.0, 1.0);
        let params = MotorAbutmentParams::default();
        let profile = compute_emergence_profile(&pos, &axis, &[], &params);
        assert_eq!(profile.len(), 12 * 8); // 12 levels × 8 points
    }

    #[test]
    fn screw_channel_aligned() {
        let pos = Point3::origin();
        let axis = Vector3::z();
        let prosth = Vector3::z();
        let params = MotorAbutmentParams::default();
        let (channel, angle) = compute_screw_channel(&pos, &axis, &prosth, &params);
        assert!(!channel.is_empty());
        assert!(angle < 1.0); // nearly aligned
    }

    #[test]
    fn generate_abutment_complete() {
        let pos = Point3::origin();
        let axis = Vector3::z();
        let prosth = Vector3::z();
        let params = MotorAbutmentParams::default();
        let result = generate_abutment(&pos, &axis, &[], &prosth, &params);
        assert!(result.total_height > 0.0);
        assert!(result.retention_area > 0.0);
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn angled_abutment_warns() {
        let pos = Point3::origin();
        let axis = Vector3::z();
        let prosth = Vector3::new(1.0, 0.0, 1.0).normalize(); // 45° angle
        let params = MotorAbutmentParams::default();
        let result = generate_abutment(&pos, &axis, &[], &prosth, &params);
        assert!(result.warnings.iter().any(|w| w.contains("angulation")));
    }

    #[test]
    fn custom_abutment_mesh_is_closed_and_nonzero() {
        let margin = (0..24)
            .map(|i| {
                let theta = std::f64::consts::TAU * i as f64 / 24.0;
                [3.2 * theta.cos(), 2.6 * theta.sin(), 5.0]
            })
            .collect();
        let req = CustomAbutmentMeshRequest {
            margin_polyline: margin,
            implant_axis: [0.0, 0.0, 1.0],
            implant_diameter_mm: 4.1,
            emergence_height_mm: 5.0,
            shoulder_width_mm: 0.45,
            taper_degrees: 5.0,
            axial_rings: 14,
            profile: AbutmentProfilePreset::Shoulder,
        };

        let (mesh, qa) = generate_custom_abutment_mesh(&req).expect("abutment mesh");
        assert_eq!(mesh.vertex_count(), 14 * 24 + 2);
        assert_eq!(mesh.triangle_count(), (14 - 1) * 24 * 2 + 48);
        assert!(qa.watertight, "{:?}", qa.warnings);
        assert!(qa.signed_volume_mm3 > 1.0);
    }

    #[test]
    fn custom_abutment_rejects_short_margin() {
        let req = CustomAbutmentMeshRequest {
            margin_polyline: vec![[0.0, 0.0, 0.0]; 5],
            ..CustomAbutmentMeshRequest::default()
        };
        assert!(generate_custom_abutment_mesh(&req).is_err());
    }

    #[test]
    fn screw_channel_plan_flags_library_limit() {
        let plan = plan_screw_channel(
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            10.0,
            2.2,
            25.0,
        );
        assert!(!plan.valid_for_library_limit);
        assert!(plan.angle_degrees > 40.0);
        assert_eq!(plan.points.len(), 16);
    }
}
