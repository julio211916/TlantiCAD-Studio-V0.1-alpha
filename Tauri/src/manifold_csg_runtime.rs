use serde::Serialize;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifoldCsgProbe {
    pub available: bool,
    pub route: &'static str,
    pub backend: &'static str,
    pub operations: Vec<&'static str>,
    pub notes: Vec<&'static str>,
    pub sample: Option<ManifoldCsgSample>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifoldCsgSample {
    pub cube_volume: f64,
    pub drilled_volume: f64,
    pub removed_volume: f64,
    pub cross_section_area: f64,
    pub extruded_volume: f64,
}

#[cfg(feature = "backend-manifold-csg")]
pub fn run_manifold_csg_probe() -> ManifoldCsgProbe {
    use manifold_csg::{CrossSection, JoinType, Manifold};

    let cube = Manifold::cube(20.0, 20.0, 20.0, true);
    let hole = Manifold::cylinder(30.0, 5.0, 5.0, 64, false);
    let drilled = &cube - &hole;

    let section = CrossSection::square(10.0, 10.0, true);
    let expanded = section.offset(2.0, JoinType::Round, 2.0, 16);
    let solid = expanded.extrude(20.0);

    let cube_volume = cube.volume();
    let drilled_volume = drilled.volume();
    let extruded_volume = solid.volume();

    ManifoldCsgProbe {
        available: true,
        route: "workspace/backend/manifold-csg/probe",
        backend: "manifold-csg",
        operations: vec![
            "union",
            "difference",
            "intersection",
            "extrude",
            "revolve",
            "offset",
            "mesh-io",
        ],
        notes: vec![
            "Good fit for mesh-first CAD tooling and procedural solid ops.",
            "Treat as short-term CSG layer, not a full B-Rep/STEP replacement for OCCT.",
        ],
        sample: Some(ManifoldCsgSample {
            cube_volume,
            drilled_volume,
            removed_volume: cube_volume - drilled_volume,
            cross_section_area: expanded.area(),
            extruded_volume,
        }),
    }
}

#[cfg(not(feature = "backend-manifold-csg"))]
pub fn run_manifold_csg_probe() -> ManifoldCsgProbe {
    ManifoldCsgProbe {
        available: false,
        route: "workspace/backend/manifold-csg/probe-disabled",
        backend: "manifold-csg",
        operations: vec![
            "union",
            "difference",
            "intersection",
            "extrude",
            "revolve",
            "offset",
            "mesh-io",
        ],
        notes: vec![
            "Feature flag backend-manifold-csg is disabled in this build.",
            "Enable it to validate mesh-first CSG workflows in the Rust backend.",
        ],
        sample: None,
    }
}
