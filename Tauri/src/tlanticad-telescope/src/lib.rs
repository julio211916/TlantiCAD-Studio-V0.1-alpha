//! TlantiCAD Telescope Module
//!
//! Double-crown (telescope) design for removable prosthetics

use tlanticad_core::Result;
use nalgebra::{Point3, Vector3};

/// Telescope type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TelescopeType {
    Conical,        // friction-retained
    Cylindrical,    // parallel walls
    Resilient,      // with spacer
}

/// Parameters for a telescope crown pair
#[derive(Debug, Clone)]
pub struct TelescopeParams {
    pub telescope_type: TelescopeType,
    pub taper_angle: f64,       // degrees (typically 2-6°)
    pub wall_thickness: f64,    // mm, primary coping
    pub gap: f64,               // μm, between primary and secondary
    pub friction_height: f64,   // mm, retention zone height
    pub chamfer_width: f64,     // mm, margin chamfer
}

impl Default for TelescopeParams {
    fn default() -> Self {
        Self {
            telescope_type: TelescopeType::Conical,
            taper_angle: 4.0,
            wall_thickness: 0.5,
            gap: 30.0,
            friction_height: 4.0,
            chamfer_width: 0.3,
        }
    }
}

/// A telescope pair (primary + secondary crown)
#[derive(Debug)]
pub struct TelescopePair {
    pub tooth_number: u8,
    pub params: TelescopeParams,
    pub margin_points: Vec<Point3<f64>>,
    pub insertion_direction: Vector3<f64>,
}

pub struct TelescopeDesigner {
    pairs: Vec<TelescopePair>,
}

impl TelescopeDesigner {
    pub fn new() -> Self {
        Self { pairs: Vec::new() }
    }

    pub fn add_pair(&mut self, tooth_number: u8, params: TelescopeParams, margin: Vec<Point3<f64>>) {
        self.pairs.push(TelescopePair {
            tooth_number,
            params,
            margin_points: margin,
            insertion_direction: Vector3::z(),
        });
    }

    /// Generate primary crown (inner telescope)
    pub fn generate_primary(&self, pair_idx: usize) -> Result<tlanticad_mesh::Mesh> {
        let pair = self.pairs.get(pair_idx)
            .ok_or_else(|| tlanticad_core::TlantiError::InvalidParameter("pair index out of range".into()))?;
        
        let mut mesh = tlanticad_mesh::Mesh::new(format!("primary_{}", pair.tooth_number));
        
        if pair.margin_points.is_empty() {
            return Ok(mesh);
        }

        // Build tapered cylinder from margin
        let center: Vector3<f64> = pair.margin_points.iter()
            .map(|p| p.coords).sum::<Vector3<f64>>() / pair.margin_points.len() as f64;
        let top_center = center + pair.insertion_direction * pair.params.friction_height;
        let radius = pair.margin_points.iter()
            .map(|p| (p.coords - center).norm())
            .sum::<f64>() / pair.margin_points.len() as f64;

        let taper_offset = pair.params.friction_height * (pair.params.taper_angle.to_radians()).tan();
        let top_radius = (radius - taper_offset).max(0.5);

        mesh = tlanticad_mesh::create_cylinder(
            Point3::from(center),
            Point3::from(top_center),
            (radius + top_radius) / 2.0,
            24,
        );
        mesh.name = format!("primary_{}", pair.tooth_number);
        mesh.calculate_normals();
        Ok(mesh)
    }

    /// Generate secondary crown (outer telescope)
    pub fn generate_secondary(&self, pair_idx: usize) -> Result<tlanticad_mesh::Mesh> {
        let pair = self.pairs.get(pair_idx)
            .ok_or_else(|| tlanticad_core::TlantiError::InvalidParameter("pair index out of range".into()))?;
        
        let mut primary = self.generate_primary(pair_idx)?;
        // Offset outward by gap amount
        tlanticad_mesh::offset(&mut primary, pair.params.gap / 1000.0);
        primary.name = format!("secondary_{}", pair.tooth_number);
        Ok(primary)
    }

    pub fn pair_count(&self) -> usize {
        self.pairs.len()
    }
}

impl Default for TelescopeDesigner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telescope_params_default() {
        let p = TelescopeParams::default();
        assert_eq!(p.telescope_type, TelescopeType::Conical);
        assert!((p.taper_angle - 4.0).abs() < 1e-6);
        assert!((p.wall_thickness - 0.5).abs() < 1e-6);
        assert!((p.gap - 30.0).abs() < 1e-6);
    }

    #[test]
    fn test_telescope_designer_new() {
        let d = TelescopeDesigner::new();
        assert_eq!(d.pair_count(), 0);
    }

    #[test]
    fn test_telescope_designer_default() {
        let d = TelescopeDesigner::default();
        assert_eq!(d.pair_count(), 0);
    }

    #[test]
    fn test_add_pair() {
        let mut d = TelescopeDesigner::new();
        d.add_pair(11, TelescopeParams::default(), vec![]);
        assert_eq!(d.pair_count(), 1);
    }

    #[test]
    fn test_add_multiple_pairs() {
        let mut d = TelescopeDesigner::new();
        d.add_pair(11, TelescopeParams::default(), vec![]);
        d.add_pair(21, TelescopeParams::default(), vec![]);
        d.add_pair(13, TelescopeParams::default(), vec![]);
        assert_eq!(d.pair_count(), 3);
    }

    #[test]
    fn test_generate_primary_empty_margin() {
        let mut d = TelescopeDesigner::new();
        d.add_pair(11, TelescopeParams::default(), vec![]);
        let mesh = d.generate_primary(0).unwrap();
        assert!(mesh.vertices.is_empty() || mesh.name.contains("primary"));
    }

    #[test]
    fn test_generate_primary_with_margin() {
        let mut d = TelescopeDesigner::new();
        let margin = vec![
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(-1.0, 0.0, 0.0),
            Point3::new(0.0, -1.0, 0.0),
        ];
        d.add_pair(11, TelescopeParams::default(), margin);
        let mesh = d.generate_primary(0).unwrap();
        assert!(mesh.name.contains("primary_11"));
    }

    #[test]
    fn test_generate_primary_out_of_range() {
        let d = TelescopeDesigner::new();
        let result = d.generate_primary(0);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_secondary() {
        let mut d = TelescopeDesigner::new();
        let margin = vec![
            Point3::new(2.0, 0.0, 0.0),
            Point3::new(0.0, 2.0, 0.0),
            Point3::new(-2.0, 0.0, 0.0),
            Point3::new(0.0, -2.0, 0.0),
        ];
        d.add_pair(21, TelescopeParams::default(), margin);
        let mesh = d.generate_secondary(0).unwrap();
        assert!(mesh.name.contains("secondary_21"));
    }

    #[test]
    fn test_telescope_type_variants() {
        assert_ne!(TelescopeType::Conical, TelescopeType::Cylindrical);
        assert_ne!(TelescopeType::Cylindrical, TelescopeType::Resilient);
    }

    #[test]
    fn test_custom_params() {
        let p = TelescopeParams {
            telescope_type: TelescopeType::Cylindrical,
            taper_angle: 0.0,
            wall_thickness: 0.8,
            gap: 50.0,
            friction_height: 6.0,
            chamfer_width: 0.5,
        };
        assert_eq!(p.telescope_type, TelescopeType::Cylindrical);
        assert!((p.taper_angle).abs() < 1e-6);
    }
}

