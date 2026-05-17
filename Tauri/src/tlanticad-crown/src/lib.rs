//! TlantiCAD Crown Design Module
//!
//! Crown generation with margin adaptation, anatomy, occlusion, contacts,
//! material space validation, and design parameters.

pub mod adaptation;
pub mod anatomy;
pub mod contacts;
pub mod occlusion;
pub mod material_space;
pub mod design_parameters;
pub mod motor;

// AR-V367 — CrownBottom (variable cement gap + border safety + ramp)
pub mod bottom;

// AR-V408 — Adapt library tooth model to virtual preparation
pub mod adapt_to_prep;

// AR-V412 — Crown border safety zone (per-vertex weights + cement-gap blending)
pub mod border_safety;

// AR-V369 — feedback / constraint bounds / thickness assurance
pub mod feedback;

// AR-V368 — full crown generation pipeline (7-step orchestrator)
pub mod pipeline;

// AR-V405 — adjusting in-situ (occlusal contact reduction against antagonist)
pub mod adjusting_situ;

use tlanticad_core::Result;
use tlanticad_mesh::Mesh;
use nalgebra::{Point3, Vector3};

/// Crown design parameters
#[derive(Debug, Clone)]
pub struct CrownParams {
    pub tooth_number: u8,
    pub cement_gap: f64,        // μm, typically 30-80
    pub extra_gap: f64,         // μm, additional spacing
    pub margin_thickness: f64,  // mm, minimum wall at margin
    pub occlusal_thickness: f64,// mm, minimum wall at occlusal
    pub occlusal_offset: f64,   // mm, reduction from antagonist
    pub smoothing_iterations: u32,
}

impl Default for CrownParams {
    fn default() -> Self {
        Self {
            tooth_number: 0,
            cement_gap: 50.0,
            extra_gap: 0.0,
            margin_thickness: 0.6,
            occlusal_thickness: 1.5,
            occlusal_offset: 0.1,
            smoothing_iterations: 3,
        }
    }
}

pub struct CrownDesigner {
    params: CrownParams,
    margin_points: Vec<Point3<f64>>,
    insertion_direction: Vector3<f64>,
}

impl CrownDesigner {
    pub fn new(params: CrownParams) -> Self {
        Self {
            params,
            margin_points: Vec::new(),
            insertion_direction: Vector3::z(),
        }
    }

    pub fn set_margin(&mut self, points: Vec<Point3<f64>>) {
        self.margin_points = points;
    }

    pub fn set_insertion_direction(&mut self, dir: Vector3<f64>) {
        self.insertion_direction = dir.normalize();
    }

    /// Generate the crown bottom (coping) from margin + preparation
    pub fn generate_bottom(margin: &[Point3<f64>], _preparation: &Mesh) -> Result<Mesh> {
        let mut mesh = Mesh::new("crown_bottom");
        if margin.is_empty() { return Ok(mesh); }

        // Generate ring of vertices from margin points
        let center: Vector3<f64> = margin.iter()
            .map(|p| p.coords)
            .sum::<Vector3<f64>>() / margin.len() as f64;
        let center_pt = Point3::from(center);

        // Create vertices from margin with slight inward offset for cement gap
        for (_i, pt) in margin.iter().enumerate() {
            let dir = (pt - center_pt).normalize();
            let offset_pt = pt - dir * 0.05; // 50μm cement gap
            mesh.vertices.push(offset_pt);
        }

        // Create triangulated ring surface
        let n = margin.len() as u32;
        let top_idx = mesh.vertices.len() as u32;
        mesh.vertices.push(center_pt + Vector3::new(0.0, 0.0, 2.0)); // top center

        for i in 0..n {
            mesh.indices.push([i, (i + 1) % n, top_idx]);
        }

        mesh.calculate_normals();
        Ok(mesh)
    }

    /// Generate full anatomic crown
    pub fn generate_anatomic(&self, _preparation: &Mesh, library_tooth: Option<&Mesh>) -> Result<Mesh> {
        let mut crown = Mesh::new("crown_anatomic");

        // Start from preparation or library tooth
        if let Some(lib) = library_tooth {
            crown = lib.clone();
            crown.name = "crown_anatomic".to_string();
        }

        // Apply cement gap offset
        if !self.margin_points.is_empty() {
            let center: Vector3<f64> = self.margin_points.iter()
                .map(|p| p.coords)
                .sum::<Vector3<f64>>() / self.margin_points.len() as f64;

            for v in &mut crown.vertices {
                let dir = (v.coords - center).normalize();
                *v = Point3::from(v.coords + dir * (self.params.cement_gap / 1000.0));
            }
        }

        crown.calculate_normals();
        Ok(crown)
    }

    /// Check minimum thickness against antagonist
    pub fn check_occlusal_clearance(&self, crown: &Mesh, antagonist: &Mesh) -> Vec<(usize, f64)> {
        let mut thin_spots = Vec::new();
        for (i, v) in crown.vertices.iter().enumerate() {
            let mut min_dist = f64::MAX;
            for av in &antagonist.vertices {
                let d = (v - av).norm();
                if d < min_dist { min_dist = d; }
            }
            if min_dist < self.params.occlusal_offset {
                thin_spots.push((i, min_dist));
            }
        }
        thin_spots
    }
}

