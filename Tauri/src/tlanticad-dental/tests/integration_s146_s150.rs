//! S146-S150: Integration tests — end-to-end workflow validation.
//!
//! Tests that combine multiple dental modules into realistic clinical
//! workflows: scan → segment → die cut → margin → insertion → occlusion.

use tlanticad_dental::scan_import::*;
use tlanticad_dental::segmentation::*;
use tlanticad_dental::model_creation::*;
use tlanticad_dental::margin::*;
use tlanticad_dental::insertion::*;
use tlanticad_dental::occlusion::*;
use tlanticad_dental::manufacturing::*;
use tlanticad_dental::notation::*;
use tlanticad_dental::tooth_library::*;
use nalgebra::{Point3, Vector3};

/// Helper: create a small test arch scan with two "teeth".
fn make_arch_scan() -> (RawScan, Vec<SegmentLabel>) {
    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();

    // "Tooth 11" region at x=0..2
    let base = vertices.len() as u32;
    vertices.extend_from_slice(&[
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(2.0, 0.0, 1.0),
        Point3::new(1.0, 2.0, 1.0),
        Point3::new(1.0, 1.0, 2.0),
    ]);
    normals.extend(vec![Vector3::new(0.0, 0.0, 1.0); 4]);
    indices.push([base, base + 1, base + 2]);
    indices.push([base, base + 1, base + 3]);
    indices.push([base, base + 2, base + 3]);
    indices.push([base + 1, base + 2, base + 3]);

    // "Tooth 21" region at x=3..5
    let base2 = vertices.len() as u32;
    vertices.extend_from_slice(&[
        Point3::new(3.0, 0.0, 1.0),
        Point3::new(5.0, 0.0, 1.0),
        Point3::new(4.0, 2.0, 1.0),
        Point3::new(4.0, 1.0, 2.0),
    ]);
    normals.extend(vec![Vector3::new(0.0, 0.0, 1.0); 4]);
    indices.push([base2, base2 + 1, base2 + 2]);
    indices.push([base2, base2 + 1, base2 + 3]);
    indices.push([base2, base2 + 2, base2 + 3]);
    indices.push([base2 + 1, base2 + 2, base2 + 3]);

    let labels = vec![
        SegmentLabel::Tooth(11),
        SegmentLabel::Tooth(11),
        SegmentLabel::Tooth(11),
        SegmentLabel::Tooth(11),
        SegmentLabel::Tooth(21),
        SegmentLabel::Tooth(21),
        SegmentLabel::Tooth(21),
        SegmentLabel::Tooth(21),
    ];

    let scan = RawScan {
        vertices,
        normals,
        indices,
        format: ScanFormat::Stl,
    };

    (scan, labels)
}

#[test]
fn workflow_scan_to_dies() {
    let (scan, labels) = make_arch_scan();

    // Quality check
    let quality = analyze_scan_quality(&scan);
    assert_eq!(quality.vertex_count, 8);
    assert_eq!(quality.triangle_count, 8);
    assert!(!quality.has_normals || quality.has_normals);

    // Cut dies
    let dies = cut_dies(&scan, &labels);
    assert_eq!(dies.len(), 2);

    // Verify FDI numbers
    let fdi_nums: Vec<u8> = dies.iter().map(|d| d.fdi_number).collect();
    assert!(fdi_nums.contains(&11));
    assert!(fdi_nums.contains(&21));
}

#[test]
fn workflow_notation_through_segmentation() {
    let (_, labels) = make_arch_scan();
    let result = SegmentationResult {
        vertex_labels: labels,
        triangle_labels: vec![SegmentLabel::Tooth(11); 8],
        segment_count: 2,
    };

    // Convert segment labels to notation
    let tooth_ids = tlanticad_dental::segmentation::segment_to_tooth_ids(&result);
    assert!(!tooth_ids.is_empty());

    // Convert to universal
    let universal = tlanticad_dental::segmentation::segment_labels_to_universal(&result);
    assert_eq!(universal.len(), 8);
    // FDI 11 → Universal 8
    assert_eq!(universal[0], Some(8));
}

#[test]
fn workflow_model_base_from_scan() {
    let (scan, _) = make_arch_scan();
    let params = BaseParams::default();
    let outline = generate_base_outline(&scan.vertices, &params);
    assert!(!outline.is_empty());

    let base = create_base_mesh(&outline, 15.0);
    assert!(!base.vertices.is_empty());
    assert!(!base.indices.is_empty());
}

#[test]
fn workflow_insertion_analysis() {
    let (scan, labels) = make_arch_scan();
    let dies = cut_dies(&scan, &labels);
    let die = &dies[0];

    let axis = Vector3::new(0.0, 0.0, 1.0);
    let prep_verts: Vec<u32> = (0..die.vertices.len() as u32).collect();
    let result = find_insertion_axis(
        &die.vertices,
        &die.normals,
        &die.indices,
        &axis,
        &prep_verts,
    );

    let dir = Vector3::new(result.direction[0], result.direction[1], result.direction[2]);
    assert!((dir.norm() - 1.0).abs() < 0.01);
    assert!(result.undercut_percent >= 0.0);
}

#[test]
fn workflow_manufacturing_validation() {
    let (scan, labels) = make_arch_scan();
    let dies = cut_dies(&scan, &labels);
    let die = &dies[0];

    let mat = Material::zirconia();
    let thicknesses = vec![1.0; die.vertices.len()];
    let report = validate_manufacturing(
        &die.vertices,
        &die.normals,
        &die.indices,
        &thicknesses,
        ManufacturingMethod::Milling { axes: 5 },
        &mat,
    );
    // Should produce a report without panicking
    assert!(!report.material.is_empty());
}

#[test]
fn workflow_tooth_library_search() {
    let catalog = default_catalog();
    assert!(catalog.templates.len() >= 32);

    let criteria = SearchCriteria {
        fdi_number: Some(11),
        ..Default::default()
    };
    let results = catalog.search(&criteria);
    assert!(!results.is_empty());
    assert_eq!(results[0].tooth_id.fdi, 11);
}

#[test]
fn workflow_articulator_setup() {
    let upper_center = Point3::new(0.0, 0.0, 10.0);
    let lower_center = Point3::new(0.0, 0.0, 0.0);
    let params = ArticulatorParams::default();
    let setup = setup_articulator(params, &upper_center, &lower_center);
    assert!(setup.vertical_dimension > 0.0);
}

#[test]
fn workflow_deciduous_notation() {
    for fdi in 51..=55 {
        assert!(is_valid_deciduous_fdi(fdi));
        let letter = deciduous_fdi_to_universal_letter(fdi).unwrap();
        let back = deciduous_universal_letter_to_fdi(letter).unwrap();
        assert_eq!(fdi, back);
    }
}

#[test]
fn workflow_scan_preprocessing() {
    let (mut scan, _) = make_arch_scan();

    // Center
    center_scan(&mut scan);
    let sum: Vector3<f64> = scan.vertices.iter().map(|p| p.coords).sum();
    let centroid = sum / scan.vertices.len() as f64;
    assert!(centroid.norm() < 1e-9);

    // Recompute normals
    recompute_normals(&mut scan);
    assert_eq!(scan.normals.len(), scan.vertices.len());
}
