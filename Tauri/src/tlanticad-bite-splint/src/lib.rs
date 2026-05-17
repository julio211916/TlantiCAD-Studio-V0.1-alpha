//! TlantiCAD Bite Splint Module
//!
//! Design of occlusal splints (night guards, Michigan splint, etc.)

pub mod multi_tooth_seg;
pub mod freeform;

use tlanticad_core::Result;
use nalgebra::Point3;

/// Splint type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SplintType {
    Michigan,       // full-coverage hard splint
    NightGuard,     // soft/dual laminate
    Anterior,       // front teeth only (Lucia jig)
    NTI,            // nociceptive trigeminal inhibition
    Ortho,          // orthodontic retainer
}

/// Splint design parameters
#[derive(Debug, Clone)]
pub struct SplintParams {
    pub splint_type: SplintType,
    pub arch: Arch,
    pub thickness: f64,             // mm, occlusal thickness
    pub lateral_thickness: f64,     // mm, buccal/lingual wall
    pub coverage_height: f64,       // mm, how far down the teeth
    pub canine_guidance: bool,      // include canine ramp
    pub anterior_guidance: bool,    // include anterior ramp
    pub flat_plane: bool,           // flat occlusal surface
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Arch {
    Upper,
    Lower,
}

impl Default for SplintParams {
    fn default() -> Self {
        Self {
            splint_type: SplintType::Michigan,
            arch: Arch::Upper,
            thickness: 2.0,
            lateral_thickness: 1.5,
            coverage_height: 4.0,
            canine_guidance: true,
            anterior_guidance: true,
            flat_plane: true,
        }
    }
}

pub struct BiteSplintDesigner {
    params: SplintParams,
    arch_scan: Option<tlanticad_mesh::Mesh>,
    antagonist_scan: Option<tlanticad_mesh::Mesh>,
}

impl BiteSplintDesigner {
    pub fn new(params: SplintParams) -> Self {
        Self {
            params,
            arch_scan: None,
            antagonist_scan: None,
        }
    }

    pub fn set_arch_scan(&mut self, mesh: tlanticad_mesh::Mesh) {
        self.arch_scan = Some(mesh);
    }

    pub fn set_antagonist(&mut self, mesh: tlanticad_mesh::Mesh) {
        self.antagonist_scan = Some(mesh);
    }

    /// Generate the splint mesh
    pub fn generate(&self) -> Result<tlanticad_mesh::Mesh> {
        let arch = self.arch_scan.as_ref()
            .ok_or_else(|| tlanticad_core::TlantiError::InvalidParameter("no arch scan provided".into()))?;

        // Start from a copy of the arch, offset outward
        let mut splint = arch.clone();
        splint.name = format!("splint_{:?}", self.params.splint_type);

        // Offset to create splint surface
        tlanticad_mesh::offset(&mut splint, self.params.thickness);

        // Smooth the occlusal surface
        tlanticad_mesh::smooth(&mut splint, self.params.thickness as usize, 0.5);

        splint.calculate_normals();
        Ok(splint)
    }

    /// Analyze occlusal contacts with antagonist
    pub fn analyze_contacts(&self, splint: &tlanticad_mesh::Mesh) -> Vec<ContactPoint> {
        let mut contacts = Vec::new();
        if let Some(ant) = &self.antagonist_scan {
            for (i, v) in splint.vertices.iter().enumerate() {
                for av in &ant.vertices {
                    let dist = (v - av).norm();
                    if dist < 0.1 { // 100μm contact threshold
                        contacts.push(ContactPoint {
                            position: *v,
                            distance: dist,
                            vertex_index: i,
                        });
                    }
                }
            }
        }
        contacts
    }
}

/// A contact point between splint and antagonist
#[derive(Debug, Clone)]
pub struct ContactPoint {
    pub position: Point3<f64>,
    pub distance: f64,
    pub vertex_index: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_splint_params_default() {
        let p = SplintParams::default();
        assert_eq!(p.splint_type, SplintType::Michigan);
        assert_eq!(p.arch, Arch::Upper);
        assert!((p.thickness - 2.0).abs() < 1e-6);
        assert!(p.canine_guidance);
        assert!(p.anterior_guidance);
        assert!(p.flat_plane);
    }

    #[test]
    fn test_bite_splint_designer_new() {
        let d = BiteSplintDesigner::new(SplintParams::default());
        assert!(d.arch_scan.is_none());
        assert!(d.antagonist_scan.is_none());
    }

    #[test]
    fn test_set_arch_scan() {
        let mut d = BiteSplintDesigner::new(SplintParams::default());
        let mesh = tlanticad_mesh::Mesh::new("arch");
        d.set_arch_scan(mesh);
        assert!(d.arch_scan.is_some());
    }

    #[test]
    fn test_set_antagonist() {
        let mut d = BiteSplintDesigner::new(SplintParams::default());
        let mesh = tlanticad_mesh::Mesh::new("ant");
        d.set_antagonist(mesh);
        assert!(d.antagonist_scan.is_some());
    }

    #[test]
    fn test_generate_no_scan_returns_error() {
        let d = BiteSplintDesigner::new(SplintParams::default());
        assert!(d.generate().is_err());
    }

    #[test]
    fn test_generate_with_scan() {
        let mut d = BiteSplintDesigner::new(SplintParams::default());
        let mut mesh = tlanticad_mesh::Mesh::new("arch");
        mesh.vertices = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.5, 1.0, 0.0),
            Point3::new(0.5, 0.5, 1.0),
        ];
        mesh.indices = vec![[0, 1, 2], [0, 1, 3], [1, 2, 3], [0, 2, 3]];
        mesh.calculate_normals();
        d.set_arch_scan(mesh);
        let splint = d.generate().unwrap();
        assert!(!splint.vertices.is_empty());
    }

    #[test]
    fn test_analyze_contacts_no_antagonist() {
        let d = BiteSplintDesigner::new(SplintParams::default());
        let splint = tlanticad_mesh::Mesh::new("s");
        let contacts = d.analyze_contacts(&splint);
        assert!(contacts.is_empty());
    }

    #[test]
    fn test_analyze_contacts_with_close_vertices() {
        let mut d = BiteSplintDesigner::new(SplintParams::default());
        let mut ant = tlanticad_mesh::Mesh::new("ant");
        ant.vertices = vec![Point3::new(0.0, 0.0, 0.05)];
        d.set_antagonist(ant);

        let mut splint = tlanticad_mesh::Mesh::new("s");
        splint.vertices = vec![Point3::new(0.0, 0.0, 0.0)];
        let contacts = d.analyze_contacts(&splint);
        assert_eq!(contacts.len(), 1);
        assert!(contacts[0].distance < 0.1);
    }

    #[test]
    fn test_analyze_contacts_far_vertices() {
        let mut d = BiteSplintDesigner::new(SplintParams::default());
        let mut ant = tlanticad_mesh::Mesh::new("ant");
        ant.vertices = vec![Point3::new(100.0, 0.0, 0.0)];
        d.set_antagonist(ant);

        let mut splint = tlanticad_mesh::Mesh::new("s");
        splint.vertices = vec![Point3::new(0.0, 0.0, 0.0)];
        let contacts = d.analyze_contacts(&splint);
        assert!(contacts.is_empty());
    }

    #[test]
    fn test_splint_type_variants() {
        assert_ne!(SplintType::Michigan, SplintType::NightGuard);
        assert_ne!(SplintType::Anterior, SplintType::NTI);
        assert_ne!(SplintType::Ortho, SplintType::Michigan);
    }

    #[test]
    fn test_arch_variants() {
        assert_ne!(Arch::Upper, Arch::Lower);
    }

    #[test]
    fn test_custom_splint_params() {
        let p = SplintParams {
            splint_type: SplintType::NTI,
            arch: Arch::Lower,
            thickness: 3.5,
            lateral_thickness: 2.0,
            coverage_height: 5.0,
            canine_guidance: false,
            anterior_guidance: false,
            flat_plane: false,
        };
        assert_eq!(p.splint_type, SplintType::NTI);
        assert!(!p.canine_guidance);
    }
}

