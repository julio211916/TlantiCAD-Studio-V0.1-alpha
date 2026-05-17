//! TlantiCAD Abutment Design Module
//!
//! Custom implant abutment design and generation

pub mod motor;

// AR-V371 — real loft from margin polyline (replaces audit no-stubs #12)
pub mod edit;

// AR-V372 — production blank, screw channel, nesting puck
pub mod production;

// AR-V422 — abutment matching parameters + scan-body ICP
pub mod matching_params;

use nalgebra::{Point3, Vector3};
use tlanticad_core::Result;
use tlanticad_mesh::Mesh;

/// Abutment design parameters
#[derive(Debug, Clone)]
pub struct AbutmentParams {
    pub implant_diameter: f64,
    pub implant_length: f64,
    pub margin_height: f64,
    pub shoulder_diameter: f64,
    pub taper_angle: f64,
    pub emergence_profile_height: f64,
    pub screw_channel_diameter: f64,
    pub screw_channel_angle: f64,
}

impl Default for AbutmentParams {
    fn default() -> Self {
        Self {
            implant_diameter: 4.1,
            implant_length: 10.0,
            margin_height: 5.0,
            shoulder_diameter: 6.0,
            taper_angle: 6.0,
            emergence_profile_height: 2.0,
            screw_channel_diameter: 2.3,
            screw_channel_angle: 0.0,
        }
    }
}

/// Abutment designer
pub struct AbutmentDesigner {
    params: AbutmentParams,
    margin_points: Vec<Point3<f64>>,
    insertion_direction: Vector3<f64>,
}

impl AbutmentDesigner {
    pub fn new(params: AbutmentParams) -> Self {
        Self {
            params,
            margin_points: Vec::new(),
            insertion_direction: Vector3::z_axis().into_inner(),
        }
    }

    pub fn set_margin(&mut self, points: Vec<Point3<f64>>) {
        self.margin_points = points;
    }

    pub fn set_insertion_direction(&mut self, direction: Vector3<f64>) {
        self.insertion_direction = direction.normalize();
    }

    pub fn generate(&self) -> Result<Mesh> {
        // Generate tapered cylinder abutment from implant platform to margin
        let base_center = Point3::origin();
        let top_center = base_center + self.insertion_direction * self.params.margin_height;

        let base_radius = self.params.implant_diameter / 2.0;
        let top_radius = self.params.shoulder_diameter / 2.0;

        // Main body: cylinder from implant connection to emergence
        let mut abutment = tlanticad_mesh::create_cylinder(
            base_center,
            top_center,
            (base_radius + top_radius) / 2.0,
            24,
        );
        abutment.name = "abutment".to_string();

        // Screw channel: boolean subtract a cylinder through center
        // For now, represented as metadata — real CSG needs OCCT integration

        abutment.calculate_normals();
        Ok(abutment)
    }
}

/// Emergence profile generator
pub struct EmergenceProfile {
    pub height: f64,
    pub base_diameter: f64,
    pub top_diameter: f64,
    pub curvature: f64,
}

impl EmergenceProfile {
    pub fn generate(&self, margin_points: &[Point3<f64>]) -> Result<Mesh> {
        if margin_points.is_empty() {
            return Ok(Mesh::new("emergence_profile"));
        }

        let center: Vector3<f64> = margin_points.iter().map(|p| p.coords).sum::<Vector3<f64>>()
            / margin_points.len() as f64;

        // Create a smooth blend surface from base to margin
        let mut mesh = tlanticad_mesh::create_cylinder(
            Point3::from(center),
            Point3::from(center + Vector3::z() * self.height),
            self.base_diameter / 2.0,
            24,
        );
        mesh.name = "emergence_profile".to_string();
        tlanticad_mesh::smooth(&mut mesh, 3, 0.5);
        mesh.calculate_normals();
        Ok(mesh)
    }
}
