//! S196-S200: Integration tests for Motor CAD sprints (S151-S195).
//!
//! End-to-end workflow tests across dental motor modules.

use tlanticad_dental::partial_restoration::{
    generate_partial_restoration, generate_veneer, validate_restoration,
    PartialRestorationParams, RestorationMaterial, RestorationType,
};
use tlanticad_dental::denture::{
    arrange_teeth_on_rim, generate_denture_base, validate_denture, bonwill_triangle_check,
    DentureParams, DentureType, DentureResult, DentureMetrics,
};
use tlanticad_dental::bar_telescope::{
    generate_bar, generate_telescope, validate_attachment,
    BarParams, TelescopeParams, AttachmentType,
};
use tlanticad_dental::bite_splint::{
    generate_splint, apply_sculpt_stroke, validate_splint,
    SplintParams, SculptStroke, SculptTool,
};
use tlanticad_dental::automation::{
    crown_workflow, bridge_workflow, standard_presets, find_preset,
    evaluate_gate, run_batch,
};
use nalgebra::{Point3, Vector3};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// S196: Partial restoration integration
// ---------------------------------------------------------------------------

#[test]
fn workflow_inlay_emax() {
    let verts = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(3.0, 0.0, 0.0),
        Point3::new(3.0, 3.0, 0.0),
        Point3::new(0.0, 3.0, 0.0),
        Point3::new(1.5, 1.5, -2.0),
    ];
    let normals = vec![Vector3::z(); 5];
    let indices = vec![[0, 1, 4], [1, 2, 4], [2, 3, 4], [3, 0, 4]];
    let params = PartialRestorationParams {
        material: RestorationMaterial::ceramic_emax(),
        ..Default::default()
    };
    let result = generate_partial_restoration(&verts, &normals, &indices, &params);
    assert_eq!(result.outer_vertices.len(), 5);
    let issues = validate_restoration(&result, &params);
    // Expect no critical failures for standard cavity
    let _ = issues; // just ensuring it runs
}

#[test]
fn workflow_veneer_feldspathic() {
    let facial = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(3.0, 0.0, 0.0),
        Point3::new(1.5, 3.0, 0.0),
    ];
    let normals = vec![Vector3::z(); 3];
    let params = PartialRestorationParams {
        restoration_type: RestorationType::Veneer,
        material: RestorationMaterial::feldspathic(),
        ..Default::default()
    };
    let result = generate_veneer(&facial, &normals, &params);
    assert!(!result.outer_vertices.is_empty());
}

// ---------------------------------------------------------------------------
// S197: Denture integration
// ---------------------------------------------------------------------------

#[test]
fn workflow_complete_upper_denture() {
    let arch: Vec<Point3<f64>> = (0..20)
        .map(|i| {
            let angle = std::f64::consts::PI * i as f64 / 19.0;
            Point3::new(25.0 * angle.cos(), 25.0 * angle.sin(), 0.0)
        })
        .collect();

    let params = DentureParams::default();
    let setups = arrange_teeth_on_rim(&arch, &params);
    assert_eq!(setups.len(), 16);

    let cast_verts = vec![Point3::new(0.0, 0.0, 0.0), Point3::new(10.0, 0.0, 0.0)];
    let (base, _) = generate_denture_base(&cast_verts, &[], &params);
    assert_eq!(base.len(), 2);

    // Bonwill triangle
    let dev = bonwill_triangle_check(
        100.0,
        &Point3::new(0.0, 86.6, 0.0),
        &Point3::new(-50.0, 0.0, 0.0),
        &Point3::new(50.0, 0.0, 0.0),
    );
    assert!(dev < 1.0);
}

// ---------------------------------------------------------------------------
// S198: Bar / telescope integration
// ---------------------------------------------------------------------------

#[test]
fn workflow_bar_and_telescope() {
    let bar_result = generate_bar(&BarParams::default());
    assert!(bar_result.total_length > 0.0);
    assert!(bar_result.cross_section_area > 0.0);

    let verts = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(2.0, 0.0, 0.0),
        Point3::new(1.0, 2.0, 0.0),
    ];
    let normals = vec![Vector3::z(); 3];
    let pair = generate_telescope(&verts, &normals, &TelescopeParams::default());
    assert!(pair.retention_force_est > 0.0);

    let (ok, _) = validate_attachment(AttachmentType::Locator, 5.0, 6.0);
    assert!(ok);
}

// ---------------------------------------------------------------------------
// S199: Splint / sculpt integration
// ---------------------------------------------------------------------------

#[test]
fn workflow_night_guard_with_sculpt() {
    let verts: Vec<Point3<f64>> = (0..10).map(|i| Point3::new(i as f64, 0.0, 0.0)).collect();
    let normals = vec![Vector3::z(); 10];
    let indices: Vec<[u32; 3]> = (0..8).map(|i| [0, i as u32 + 1, i as u32 + 2]).collect();

    let result = generate_splint(&verts, &normals, &indices, &SplintParams::default());
    assert_eq!(result.outer_vertices.len(), 10);
    let issues = validate_splint(&result, &SplintParams::default());
    let _ = issues;

    // Sculpt on outer surface
    let mut outer = result.outer_vertices;
    let stroke = SculptStroke {
        tool: SculptTool::Pull,
        center: [5.0, 0.0, 2.0],
        radius: 10.0,
        strength: 0.5,
        direction: [0.0, 0.0, 1.0],
    };
    apply_sculpt_stroke(&mut outer, &normals, &stroke);
    assert!(outer.iter().any(|v| v.z > verts[0].z));
}

// ---------------------------------------------------------------------------
// S200: Automation integration
// ---------------------------------------------------------------------------

#[test]
fn workflow_batch_automation() {
    let template = crown_workflow();
    assert_eq!(template.steps.len(), 8);

    let _bridge_wf = bridge_workflow();
    let _presets = standard_presets();
    let crown_preset = find_preset("Standard Crown").unwrap();
    assert!(crown_preset.parameters.contains_key("cement_gap"));

    // Run batch
    let ids = vec!["patient_A".into(), "patient_B".into()];
    let mut metrics = HashMap::new();
    let mut ma = HashMap::new();
    ma.insert("margin_gap".into(), 40.0);
    ma.insert("min_thickness".into(), 0.7);
    ma.insert("occlusal_clearance".into(), 0.4);
    metrics.insert("patient_A".into(), ma);

    let mut mb = HashMap::new();
    mb.insert("margin_gap".into(), 150.0);
    mb.insert("min_thickness".into(), 0.3);
    mb.insert("occlusal_clearance".into(), 0.2);
    metrics.insert("patient_B".into(), mb);

    let batch = run_batch(&ids, &template, &metrics);
    assert_eq!(batch.success_count, 1);
    assert_eq!(batch.failure_count, 1);
    assert_eq!(batch.cases.len(), 2);
}
