//! TlantiCAD Model Creator Module
//!
//! Study model generation, individual tooth dies, base plate design, and bite articulation.

pub mod study_model;
pub mod die;
pub mod base;
pub mod articulation;

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::{Point3, Vector3};

    // ── StudyModel ──────────────────────────────────────────────
    #[test]
    fn test_study_model_new() {
        let m = study_model::StudyModel::new(
            study_model::ModelType::FullArch,
            study_model::ArchType::Upper,
            "intraoral",
        );
        assert_eq!(m.model_type, study_model::ModelType::FullArch);
        assert_eq!(m.arch, study_model::ArchType::Upper);
        assert_eq!(m.scan_source, "intraoral");
        assert!(m.teeth.is_empty());
    }

    #[test]
    fn test_study_model_add_find_tooth() {
        let mut m = study_model::StudyModel::new(
            study_model::ModelType::WorkingModel,
            study_model::ArchType::Lower,
            "scan",
        );
        m.add_tooth(die::ToothDie::new(36));
        m.add_tooth(die::ToothDie::new(37));
        assert_eq!(m.teeth.len(), 2);
        assert!(m.find_tooth(36).is_some());
        assert!(m.find_tooth(99).is_none());
    }

    // ── ToothDie ────────────────────────────────────────────────
    #[test]
    fn test_tooth_die_new() {
        let d = die::ToothDie::new(21);
        assert_eq!(d.fdi_number, 21);
        assert!(d.mesh.is_none());
        assert!(d.margin_line.is_empty());
        assert!((d.insertion_axis.z - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_calculate_insertion_axis_no_mesh() {
        let d = die::ToothDie::new(11);
        let axis = die::calculate_insertion_axis(&d);
        assert!((axis.z - 1.0).abs() < 1e-6); // default z-up
    }

    #[test]
    fn test_calculate_insertion_axis_with_mesh() {
        let mut d = die::ToothDie::new(11);
        let mut mesh = tlanticad_mesh::Mesh::new("die");
        mesh.vertices = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.5, 1.0, 0.0),
            Point3::new(0.5, 0.5, 2.0),
        ];
        mesh.indices = vec![[0, 1, 2], [0, 1, 3], [1, 2, 3], [0, 2, 3]];
        mesh.calculate_normals();
        d.mesh = Some(mesh);
        let axis = die::calculate_insertion_axis(&d);
        assert!(axis.norm() > 0.9);
    }

    // ── ModelBase ───────────────────────────────────────────────
    #[test]
    fn test_model_base_default() {
        let b = base::ModelBase::default();
        assert_eq!(b.base_type, base::BaseType::Horseshoe);
        assert!((b.height - 15.0).abs() < 1e-6);
        assert!((b.width - 80.0).abs() < 1e-6);
        assert!((b.thickness - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_generate_base_outline_empty() {
        let mesh = tlanticad_mesh::Mesh::new("empty");
        let b = base::ModelBase::default();
        let outline = base::generate_base_outline(&mesh, &b);
        assert!(outline.is_empty());
    }

    #[test]
    fn test_generate_base_outline_horseshoe() {
        let mut mesh = tlanticad_mesh::Mesh::new("arch");
        mesh.vertices = vec![
            Point3::new(-20.0, -10.0, 0.0),
            Point3::new(20.0, -10.0, 0.0),
            Point3::new(0.0, 10.0, 5.0),
        ];
        mesh.indices = vec![[0, 1, 2]];
        let b = base::ModelBase::default();
        let outline = base::generate_base_outline(&mesh, &b);
        assert!(!outline.is_empty());
    }

    // ── Articulation ────────────────────────────────────────────
    #[test]
    fn test_calculate_centric_occlusion_empty() {
        let upper = study_model::StudyModel::new(
            study_model::ModelType::FullArch,
            study_model::ArchType::Upper,
            "s",
        );
        let lower = study_model::StudyModel::new(
            study_model::ModelType::FullArch,
            study_model::ArchType::Lower,
            "s",
        );
        let record = articulation::calculate_centric_occlusion(&upper, &lower);
        assert!(record.contacts.is_empty());
    }

    #[test]
    fn test_register_bite() {
        let upper = tlanticad_mesh::Mesh::new("u");
        let lower = tlanticad_mesh::Mesh::new("l");
        let reg = tlanticad_mesh::Mesh::new("r");
        let iso = articulation::register_bite(&upper, &lower, &reg);
        // identity-like when all meshes empty
        let t = iso.translation.vector;
        assert!(t.norm() < 1e-6);
    }
}
