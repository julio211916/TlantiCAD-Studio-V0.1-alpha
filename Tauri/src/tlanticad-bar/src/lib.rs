//! TlantiCAD Bar Design Module
//! Implant-supported bar frameworks for removable overdentures

pub mod bar_design;
pub mod attachment;
pub mod cross_section;

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::Point3;
    use uuid::Uuid;

    // ── bar_design tests ────────────────────────────────────────
    #[test]
    fn test_bar_framework_new() {
        let fw = bar_design::BarFramework::new();
        assert!(fw.segments.is_empty());
        assert!(fw.attachments.is_empty());
        assert_eq!(fw.retentive_clips, 0);
    }

    #[test]
    fn test_bar_framework_default() {
        let fw = bar_design::BarFramework::default();
        assert!(fw.segments.is_empty());
    }

    #[test]
    fn test_bar_segment_create() {
        let seg = bar_design::BarSegment {
            start_implant: Uuid::new_v4(),
            end_implant: Uuid::new_v4(),
            bar_type: bar_design::BarType::Dolder,
            material: bar_design::BarMaterial::Titanium,
            length_mm: 15.0,
        };
        assert_eq!(seg.length_mm, 15.0);
        assert_eq!(seg.bar_type, bar_design::BarType::Dolder);
    }

    #[test]
    fn test_design_bar_path_single_point() {
        let pts = vec![Point3::new(0.0, 0.0, 0.0)];
        let path = bar_design::design_bar_path(&pts);
        assert_eq!(path.len(), 1);
    }

    #[test]
    fn test_design_bar_path_two_points() {
        let pts = vec![Point3::new(0.0, 0.0, 0.0), Point3::new(10.0, 0.0, 0.0)];
        let path = bar_design::design_bar_path(&pts);
        assert!(path.len() > 2); // interpolated
        assert!((path.first().unwrap().x).abs() < 1e-6);
    }

    #[test]
    fn test_design_bar_path_three_implants() {
        let pts = vec![
            Point3::new(-10.0, 0.0, 0.0),
            Point3::new(0.0, 5.0, 0.0),
            Point3::new(10.0, 0.0, 0.0),
        ];
        let path = bar_design::design_bar_path(&pts);
        assert!(path.len() >= 10);
    }

    #[test]
    fn test_calculate_bar_length_empty() {
        assert_eq!(bar_design::calculate_bar_length(&[]), 0.0);
    }

    #[test]
    fn test_calculate_bar_length_straight() {
        let pts = vec![Point3::new(0.0, 0.0, 0.0), Point3::new(10.0, 0.0, 0.0)];
        let len = bar_design::calculate_bar_length(&pts);
        assert!((len - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_generate_tube_mesh() {
        let path = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(0.0, 0.0, 5.0),
            Point3::new(0.0, 0.0, 10.0),
        ];
        let mesh = bar_design::generate_tube_mesh(&path, 1.0, 8);
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.indices.is_empty());
    }

    #[test]
    fn test_generate_tube_mesh_degenerate() {
        let path = vec![Point3::new(0.0, 0.0, 0.0)];
        let mesh = bar_design::generate_tube_mesh(&path, 1.0, 8);
        assert!(mesh.vertices.is_empty());
    }

    #[test]
    fn test_bar_material_variants() {
        let materials = [
            bar_design::BarMaterial::Titanium,
            bar_design::BarMaterial::CobaltChrome,
            bar_design::BarMaterial::PEEK,
            bar_design::BarMaterial::Zirconia,
        ];
        assert_eq!(materials.len(), 4);
    }

    // ── attachment tests ────────────────────────────────────────
    #[test]
    fn test_retention_force_new() {
        let f = attachment::get_retention_force(&attachment::AttachmentType::LocatorR, 0.0);
        assert!((f - 20.0).abs() < 1e-6);
    }

    #[test]
    fn test_retention_force_full_wear() {
        let f = attachment::get_retention_force(&attachment::AttachmentType::LocatorR, 1.0);
        assert!((f - 20.0 * 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_retention_force_half_wear() {
        let f = attachment::get_retention_force(&attachment::AttachmentType::Ball, 0.5);
        let expected = 10.0 * (1.0 - 0.5 * 0.6);
        assert!((f - expected).abs() < 1e-6);
    }

    #[test]
    fn test_retention_all_types() {
        let types = [
            attachment::AttachmentType::LocatorR,
            attachment::AttachmentType::Ball,
            attachment::AttachmentType::Magnets,
            attachment::AttachmentType::ERA,
            attachment::AttachmentType::Dalbo,
        ];
        for t in &types {
            let f = attachment::get_retention_force(t, 0.0);
            assert!(f > 0.0);
        }
    }

    #[test]
    fn test_retention_clamp_negative_wear() {
        let f = attachment::get_retention_force(&attachment::AttachmentType::Magnets, -0.5);
        let expected = attachment::get_retention_force(&attachment::AttachmentType::Magnets, 0.0);
        assert!((f - expected).abs() < 1e-6);
    }

    // ── cross_section tests ─────────────────────────────────────
    #[test]
    fn test_cross_section_round() {
        let area = cross_section::cross_section_area(&cross_section::CrossSectionShape::Round(4.0));
        let expected = std::f64::consts::PI * 4.0;
        assert!((area - expected).abs() < 1e-6);
    }

    #[test]
    fn test_cross_section_rectangular() {
        let area = cross_section::cross_section_area(&cross_section::CrossSectionShape::Rectangular { w: 3.0, h: 5.0 });
        assert!((area - 15.0).abs() < 1e-6);
    }

    #[test]
    fn test_cross_section_oval() {
        let area = cross_section::cross_section_area(&cross_section::CrossSectionShape::Oval { w: 4.0, h: 6.0 });
        let expected = std::f64::consts::PI * 2.0 * 3.0;
        assert!((area - expected).abs() < 1e-6);
    }

    #[test]
    fn test_minimum_cross_section_all_materials() {
        let materials = [
            bar_design::BarMaterial::Titanium,
            bar_design::BarMaterial::CobaltChrome,
            bar_design::BarMaterial::PEEK,
            bar_design::BarMaterial::Zirconia,
        ];
        for m in &materials {
            let cs = cross_section::minimum_cross_section(m);
            let area = cross_section::cross_section_area(&cs);
            assert!(area > 0.0);
        }
    }
}
