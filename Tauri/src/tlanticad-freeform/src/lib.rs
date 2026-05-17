//! TlantiCAD Freeform Tools Module
//!
//! Full sculpting brush engine, symmetry, stamp brushes, and undo/redo history.

pub mod brush_engine;
pub mod symmetry;
pub mod stamp;
pub mod history;

// AR-V374 — paint-pull / smooth / drape + emergence profile
pub mod paint_pull;

// AR-V376 — distance shader (cierra audit no-stubs #10)
pub mod distance_shader;

// AR-V375 — specialty shapes (multi-anchor bar, telescope pair, post+core)
pub mod specialty;

// AR-V383 — full prosthetic base loft (cervical → emergence → flange)
pub mod prosthetic_base;

// AR-V384 — full multi-anchor bar with profile selection
pub mod bar;

// AR-V386 — partial framework (clasps + connectors + finishline)
pub mod partial_framework;

// AR-V394 — approximal blockout visualizer
pub mod approximal_blockout;

// AR-V397 — discrete rotation handler
pub mod discrete_rotation;

// AR-V401 — gingiva segmentation by complement of tooth regions
pub mod gingiva_seg;

// AR-V402 — virtual preparation generation from intact anatomy
pub mod virtual_preparation;

// AR-V414 — interactive adapt-to-virtual-prep (drape + bottom offset + smoothing)
pub mod adapt_to_prep;

// AR-V419 — text emboss / deboss on freeform surface
pub mod text_attachment;

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::{Point3, Vector3};

    // ── brush_engine tests ──────────────────────────────────────
    #[test]
    fn test_brush_engine_default() {
        let e = brush_engine::BrushEngine::default();
        assert_eq!(e.symmetry, symmetry::SymmetryMode::None);
        assert!(e.pressure_sensitivity);
    }

    #[test]
    fn test_sculpt_brush_default() {
        let b = brush_engine::SculptBrush::default();
        assert_eq!(b.brush_type, brush_engine::BrushType::Clay);
        assert!((b.radius - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_gaussian_falloff_center() {
        let v = brush_engine::gaussian_falloff(0.0, 1.0);
        assert!((v - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_gaussian_falloff_edge() {
        let v = brush_engine::gaussian_falloff(1.0, 1.0);
        assert_eq!(v, 0.0); // at edge = radius, returns 0
    }

    #[test]
    fn test_gaussian_falloff_half() {
        let v = brush_engine::gaussian_falloff(0.5, 1.0);
        assert!(v > 0.0 && v < 1.0);
    }

    #[test]
    fn test_gaussian_falloff_outside() {
        let v = brush_engine::gaussian_falloff(5.0, 1.0);
        assert_eq!(v, 0.0);
    }

    #[test]
    fn test_process_stroke_empty() {
        let engine = brush_engine::BrushEngine::default();
        let mut mesh = tlanticad_mesh::Mesh::new("test");
        mesh.vertices = vec![Point3::new(0.0, 0.0, 0.0)];
        let affected = brush_engine::process_stroke(&engine, &mut mesh, &[]);
        assert!(affected.is_empty());
    }

    #[test]
    fn test_process_stroke_clay() {
        let engine = brush_engine::BrushEngine {
            active_brush: brush_engine::SculptBrush {
                radius: 10.0,
                strength: 1.0,
                falloff: 0.5,
                brush_type: brush_engine::BrushType::Clay,
            },
            symmetry: symmetry::SymmetryMode::None,
            pressure_sensitivity: false,
        };
        let mut mesh = tlanticad_mesh::Mesh::new("test");
        mesh.vertices = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.5, 1.0, 0.0),
        ];
        mesh.indices = vec![[0, 1, 2]];
        mesh.calculate_normals();

        let stroke = vec![brush_engine::StrokePoint {
            position: Point3::new(0.5, 0.3, 0.0),
            normal: Vector3::z(),
            pressure: 1.0,
            timestamp: 0.0,
        }];
        let affected = brush_engine::process_stroke(&engine, &mut mesh, &stroke);
        assert!(!affected.is_empty());
    }

    // ── symmetry tests ──────────────────────────────────────────
    #[test]
    fn test_mirror_stroke_none() {
        let stroke = vec![brush_engine::StrokePoint {
            position: Point3::new(1.0, 2.0, 3.0),
            normal: Vector3::z(),
            pressure: 1.0,
            timestamp: 0.0,
        }];
        let mirrored = symmetry::mirror_stroke(&stroke, &symmetry::SymmetryMode::None);
        assert!(mirrored.is_empty());
    }

    #[test]
    fn test_mirror_stroke_x() {
        let stroke = vec![brush_engine::StrokePoint {
            position: Point3::new(1.0, 2.0, 3.0),
            normal: Vector3::x(),
            pressure: 1.0,
            timestamp: 0.0,
        }];
        let mirrored = symmetry::mirror_stroke(&stroke, &symmetry::SymmetryMode::MirrorX);
        assert_eq!(mirrored.len(), 1);
        assert!((mirrored[0].position.x - (-1.0)).abs() < 1e-6);
        assert!((mirrored[0].position.y - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_mirror_stroke_y() {
        let stroke = vec![brush_engine::StrokePoint {
            position: Point3::new(1.0, 2.0, 3.0),
            normal: Vector3::y(),
            pressure: 1.0,
            timestamp: 0.0,
        }];
        let mirrored = symmetry::mirror_stroke(&stroke, &symmetry::SymmetryMode::MirrorY);
        assert_eq!(mirrored.len(), 1);
        assert!((mirrored[0].position.y - (-2.0)).abs() < 1e-6);
    }

    #[test]
    fn test_mirror_stroke_z() {
        let stroke = vec![brush_engine::StrokePoint {
            position: Point3::new(1.0, 2.0, 3.0),
            normal: Vector3::z(),
            pressure: 1.0,
            timestamp: 0.0,
        }];
        let mirrored = symmetry::mirror_stroke(&stroke, &symmetry::SymmetryMode::MirrorZ);
        assert_eq!(mirrored.len(), 1);
        assert!((mirrored[0].position.z - (-3.0)).abs() < 1e-6);
    }

    #[test]
    fn test_radial_symmetry_4() {
        let stroke = vec![brush_engine::StrokePoint {
            position: Point3::new(1.0, 0.0, 0.0),
            normal: Vector3::y(),
            pressure: 1.0,
            timestamp: 0.0,
        }];
        let result = symmetry::apply_radial_symmetry(&stroke, 4);
        assert_eq!(result.len(), 3); // count - 1 copies
    }

    #[test]
    fn test_radial_symmetry_one() {
        let stroke = vec![brush_engine::StrokePoint {
            position: Point3::new(1.0, 0.0, 0.0),
            normal: Vector3::y(),
            pressure: 1.0,
            timestamp: 0.0,
        }];
        let result = symmetry::apply_radial_symmetry(&stroke, 1);
        assert!(result.is_empty());
    }

    // ── stamp tests ─────────────────────────────────────────────
    #[test]
    fn test_apply_stamp_moves_vertices() {
        let mut mesh = tlanticad_mesh::Mesh::new("test");
        mesh.vertices = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.5, 1.0, 0.0),
        ];
        mesh.indices = vec![[0, 1, 2]];
        mesh.calculate_normals();
        let original_z0 = mesh.vertices[0].z;
        stamp::apply_stamp(
            &mut mesh,
            &Point3::new(0.5, 0.3, 0.0),
            &Vector3::z(),
            &stamp::StampShape::Circle,
            1.0,
            5.0,
        );
        // At least some vertices should have moved
        let moved = mesh.vertices.iter().any(|v| (v.z - original_z0).abs() > 1e-6);
        assert!(moved);
    }

    #[test]
    fn test_apply_stamp_zero_size() {
        let mut mesh = tlanticad_mesh::Mesh::new("test");
        mesh.vertices = vec![Point3::new(0.0, 0.0, 0.0)];
        let before = mesh.vertices[0];
        stamp::apply_stamp(
            &mut mesh,
            &Point3::new(0.0, 0.0, 0.0),
            &Vector3::z(),
            &stamp::StampShape::Circle,
            1.0,
            0.0,
        );
        assert_eq!(mesh.vertices[0], before);
    }

    // ── history tests ───────────────────────────────────────────
    #[test]
    fn test_history_new() {
        let h = history::HistoryStack::new(10);
        assert_eq!(h.undo_count(), 0);
        assert_eq!(h.redo_count(), 0);
    }

    #[test]
    fn test_history_push_undo() {
        let mut h = history::HistoryStack::new(10);
        let mesh = tlanticad_mesh::Mesh::new("test");
        h.push(&mesh);
        assert_eq!(h.undo_count(), 1);

        let mut mesh2 = tlanticad_mesh::Mesh::new("test2");
        let undone = h.undo(&mut mesh2);
        assert!(undone);
        assert_eq!(h.undo_count(), 0);
        assert_eq!(h.redo_count(), 1);
    }

    #[test]
    fn test_history_redo() {
        let mut h = history::HistoryStack::new(10);
        let mesh = tlanticad_mesh::Mesh::new("t");
        h.push(&mesh);
        let mut current = tlanticad_mesh::Mesh::new("c");
        h.undo(&mut current);
        let redone = h.redo(&mut current);
        assert!(redone);
        assert_eq!(h.redo_count(), 0);
    }

    #[test]
    fn test_history_undo_empty() {
        let mut h = history::HistoryStack::new(10);
        let mut mesh = tlanticad_mesh::Mesh::new("t");
        assert!(!h.undo(&mut mesh));
    }

    #[test]
    fn test_history_redo_empty() {
        let mut h = history::HistoryStack::new(10);
        let mut mesh = tlanticad_mesh::Mesh::new("t");
        assert!(!h.redo(&mut mesh));
    }

    #[test]
    fn test_history_clear() {
        let mut h = history::HistoryStack::new(10);
        let mesh = tlanticad_mesh::Mesh::new("t");
        h.push(&mesh);
        h.push(&mesh);
        h.clear();
        assert_eq!(h.undo_count(), 0);
    }

    #[test]
    fn test_history_max_size() {
        let mut h = history::HistoryStack::new(3);
        let mesh = tlanticad_mesh::Mesh::new("t");
        for _ in 0..5 {
            h.push(&mesh);
        }
        assert!(h.undo_count() <= 3);
    }

    #[test]
    fn test_history_push_clears_redo() {
        let mut h = history::HistoryStack::new(10);
        let mesh = tlanticad_mesh::Mesh::new("t");
        h.push(&mesh);
        h.push(&mesh);
        let mut current = tlanticad_mesh::Mesh::new("c");
        h.undo(&mut current);
        assert_eq!(h.redo_count(), 1);
        h.push(&mesh);
        assert_eq!(h.redo_count(), 0);
    }
}
