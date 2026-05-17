//! TlantiCAD WaxUp Module
//!
//! Digital wax-up workflow: state machine, sculpting, anatomy library, and virtual articulator.

pub mod workflow;
pub mod sculpt_tool;
pub mod anatomy_library;
pub mod articulator;

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::{Point3, Vector3};
    use uuid::Uuid;

    // ── workflow tests ──────────────────────────────────────────
    #[test]
    fn test_session_new() {
        let pid = Uuid::new_v4();
        let s = workflow::WaxupSession::new(pid, vec![11, 21]);
        assert_eq!(s.patient_id, pid);
        assert_eq!(s.tooth_numbers, vec![11, 21]);
        assert_eq!(s.state, workflow::WaxupState::NotStarted);
    }

    #[test]
    fn test_advance_state_full_cycle() {
        let mut s = workflow::WaxupSession::new(Uuid::new_v4(), vec![11]);
        assert!(s.advance_state()); // → ScanLoaded
        assert_eq!(s.state, workflow::WaxupState::ScanLoaded);
        assert!(s.advance_state()); // → AnatomySelected
        assert!(s.advance_state()); // → Roughed
        assert!(s.advance_state()); // → Detailed
        assert!(s.advance_state()); // → Finalized
        assert!(!s.advance_state()); // stays Finalized
    }

    #[test]
    fn test_can_export() {
        let mut s = workflow::WaxupSession::new(Uuid::new_v4(), vec![11]);
        assert!(!s.can_export()); // NotStarted
        s.advance_state();
        assert!(!s.can_export()); // ScanLoaded
        s.advance_state();
        assert!(!s.can_export()); // AnatomySelected
        s.advance_state();
        assert!(!s.can_export()); // Roughed
        s.advance_state();
        assert!(s.can_export()); // Detailed
        s.advance_state();
        assert!(s.can_export()); // Finalized
    }

    // ── anatomy_library tests ───────────────────────────────────
    #[test]
    fn test_get_template_names() {
        let names = anatomy_library::get_template_names();
        assert_eq!(names.len(), 14);
        assert!(names.contains(&"Upper Central Incisor"));
        assert!(names.contains(&"Lower First Molar"));
    }

    #[test]
    fn test_morphology_params_upper_central() {
        let p = anatomy_library::get_morphology_params(11);
        assert!((p.length - 10.5).abs() < 1e-6);
        assert!((p.cusp_height).abs() < 1e-6);
    }

    #[test]
    fn test_morphology_params_lower_molar() {
        let p = anatomy_library::get_morphology_params(46);
        assert!(p.cusp_height > 0.0);
        assert!(p.width_buccal > 10.0);
    }

    #[test]
    fn test_morphology_params_mirror_symmetry() {
        let p11 = anatomy_library::get_morphology_params(11);
        let p21 = anatomy_library::get_morphology_params(21);
        assert!((p11.length - p21.length).abs() < 1e-6);
    }

    #[test]
    fn test_morphology_params_unknown_tooth() {
        let p = anatomy_library::get_morphology_params(99);
        assert!((p.length - 8.0).abs() < 1e-6);
    }

    // ── sculpt_tool tests ───────────────────────────────────────
    #[test]
    fn test_sculpt_brush_default() {
        let b = sculpt_tool::SculptBrush::default();
        assert!((b.radius - 2.0).abs() < 1e-6);
        assert_eq!(b.brush_type, sculpt_tool::BrushType::AddClay);
    }

    #[test]
    fn test_apply_brush_add_clay() {
        let mut mesh = tlanticad_mesh::Mesh::new("test");
        mesh.vertices = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.5, 1.0, 0.0),
        ];
        mesh.indices = vec![[0, 1, 2]];
        mesh.calculate_normals();
        let original_z = mesh.vertices[0].z;

        let brush = sculpt_tool::SculptBrush {
            radius: 5.0,
            strength: 1.0,
            falloff: 0.5,
            brush_type: sculpt_tool::BrushType::AddClay,
        };
        sculpt_tool::apply_brush(&mut mesh, &Point3::new(0.5, 0.3, 0.0), &brush, &Vector3::z());
        assert!(mesh.vertices[0].z > original_z);
    }

    #[test]
    fn test_smooth_region() {
        let mut mesh = tlanticad_mesh::Mesh::new("test");
        mesh.vertices = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.5, 1.0, 0.0),
            Point3::new(0.5, 0.5, 2.0),
        ];
        mesh.indices = vec![[0, 1, 2], [0, 1, 3], [1, 2, 3], [0, 2, 3]];
        mesh.calculate_normals();

        sculpt_tool::smooth_region(&mut mesh, &Point3::new(0.5, 0.5, 0.5), 5.0, 3);
        assert!(mesh.vertices[3].z < 2.0);
    }

    #[test]
    fn test_undo_stack_push() {
        let mut stack = Vec::new();
        let mesh = tlanticad_mesh::Mesh::new("test");
        sculpt_tool::undo_stack_push(&mut stack, &mesh, 5);
        assert_eq!(stack.len(), 1);
    }

    #[test]
    fn test_undo_stack_max_size() {
        let mut stack = Vec::new();
        let mesh = tlanticad_mesh::Mesh::new("test");
        for _ in 0..10 {
            sculpt_tool::undo_stack_push(&mut stack, &mesh, 5);
        }
        assert_eq!(stack.len(), 5);
    }

    // ── articulator tests ───────────────────────────────────────
    #[test]
    fn test_mounting_record_default() {
        let m = articulator::MountingRecord::default();
        assert!((m.condylar_inclination - 40.0).abs() < 1e-6);
        assert!((m.bennett_angle - 15.0).abs() < 1e-6);
    }

    #[test]
    fn test_simulate_protrusive() {
        let m = articulator::MountingRecord::default();
        let iso = articulator::simulate_excursion(&m, articulator::ExcursionType::Protrusive, 5.0);
        let t = iso.translation;
        assert!(t.y.abs() > 0.0);
    }

    #[test]
    fn test_simulate_left_lateral() {
        let m = articulator::MountingRecord::default();
        let iso = articulator::simulate_excursion(&m, articulator::ExcursionType::LeftLateral, 5.0);
        let t = iso.translation;
        assert!(t.y.abs() > 0.0);
    }

    #[test]
    fn test_simulate_zero_angle() {
        let m = articulator::MountingRecord::default();
        let iso = articulator::simulate_excursion(&m, articulator::ExcursionType::Protrusive, 0.0);
        let t = iso.translation;
        assert!(t.vector.norm() < 1e-6);
    }
}
