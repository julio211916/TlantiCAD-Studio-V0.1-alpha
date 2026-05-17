// AR-V377 — Articulator engine (Tauri command surface).
//
// Closes audit no-stubs item #9: relabels `cad_articulator/simulate` from "mock" to "bonwill".
//
// Three commands:
//   * `cad_articulator_default_triangle` — return canonical Bonwill triangle.
//   * `cad_articulator_register`         — register from 3 landmarks.
//   * `cad_articulator_simulate`         — compute mandibular transform for a motion state.
//   * `cad_articulator_fit_plane`        — fit a plane (occlusal/Frankfort/Camper) from points.

use serde::{Deserialize, Serialize};
use tlanticad_articulator::bonwill::{default_triangle, BonwillParams, BonwillTriangle};
use tlanticad_articulator::jaw_motion::{
    mandibular_transform, opening_path, AffineTransform, JawMotionState,
};
use tlanticad_articulator::planes::{
    camper_plane, fit_plane, frankfort_plane, occlusal_plane, Plane,
};
use tlanticad_articulator::registration::{register_from_landmarks, RegistrationResult};
use tlanticad_mesh::nalgebra::Point3;

#[derive(Debug, Serialize, thiserror::Error)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ArticulatorError {
    #[error("invalid request: {message}")]
    Invalid { message: String },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DefaultTriangleRequest {
    #[serde(default = "default_side")]
    pub side_length_mm: f64,
    #[serde(default = "default_balkwill")]
    pub balkwill_angle_deg: f64,
    #[serde(default = "default_spee")]
    pub curve_of_spee_radius_mm: f64,
}

fn default_side() -> f64 {
    100.0
}
fn default_balkwill() -> f64 {
    26.0
}
fn default_spee() -> f64 {
    110.0
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DefaultTriangleResponse {
    pub triangle: BonwillTriangle,
    pub backend: &'static str,
}

#[tauri::command]
pub fn cad_articulator_default_triangle(
    request: DefaultTriangleRequest,
) -> DefaultTriangleResponse {
    let params = BonwillParams {
        side_length_mm: request.side_length_mm,
        balkwill_angle_deg: request.balkwill_angle_deg,
        curve_of_spee_radius_mm: request.curve_of_spee_radius_mm,
    };
    DefaultTriangleResponse {
        triangle: default_triangle(&params),
        backend: "bonwill",
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    pub incisor: [f64; 3],
    pub condyle_right: [f64; 3],
    pub condyle_left: [f64; 3],
    #[serde(default = "default_side")]
    pub side_length_mm: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterResponse {
    pub registration: RegistrationResult,
    pub backend: &'static str,
}

#[tauri::command]
pub fn cad_articulator_register(request: RegisterRequest) -> RegisterResponse {
    let params = BonwillParams {
        side_length_mm: request.side_length_mm,
        ..Default::default()
    };
    let registration = register_from_landmarks(
        Point3::new(request.incisor[0], request.incisor[1], request.incisor[2]),
        Point3::new(
            request.condyle_right[0],
            request.condyle_right[1],
            request.condyle_right[2],
        ),
        Point3::new(
            request.condyle_left[0],
            request.condyle_left[1],
            request.condyle_left[2],
        ),
        &params,
    );
    RegisterResponse {
        registration,
        backend: "bonwill",
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulateRequest {
    pub triangle: BonwillTriangle,
    pub state: JawMotionState,
    /// If > 0, returns N samples of the opening path instead of a single transform.
    #[serde(default)]
    pub sample_path: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulateResponse {
    pub transform: AffineTransform,
    pub path: Option<Vec<AffineTransform>>,
    pub backend: &'static str,
}

#[tauri::command]
pub fn cad_articulator_simulate(request: SimulateRequest) -> SimulateResponse {
    let transform = mandibular_transform(&request.triangle, &request.state);
    let path = request.sample_path.and_then(|n| {
        if n == 0 {
            None
        } else {
            Some(opening_path(
                &request.triangle,
                request.state.opening_deg,
                n as usize,
            ))
        }
    });
    SimulateResponse {
        transform,
        path,
        backend: "bonwill",
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PlaneKind {
    Occlusal,
    Frankfort,
    Camper,
    LeastSquares,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FitPlaneRequest {
    pub kind: PlaneKind,
    pub points: Vec<[f64; 3]>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FitPlaneResponse {
    pub plane: Plane,
    pub backend: &'static str,
}

#[tauri::command]
pub fn cad_articulator_fit_plane(
    request: FitPlaneRequest,
) -> Result<FitPlaneResponse, ArticulatorError> {
    let pts: Vec<Point3<f64>> = request
        .points
        .iter()
        .map(|p| Point3::new(p[0], p[1], p[2]))
        .collect();
    let plane = match request.kind {
        PlaneKind::Frankfort => {
            if pts.len() < 3 {
                return Err(ArticulatorError::Invalid {
                    message: "Frankfort plane requires 3 points".into(),
                });
            }
            frankfort_plane(pts[0], pts[1], pts[2])
        }
        PlaneKind::Camper => {
            if pts.len() < 3 {
                return Err(ArticulatorError::Invalid {
                    message: "Camper plane requires 3 points".into(),
                });
            }
            camper_plane(pts[0], pts[1], pts[2])
        }
        PlaneKind::Occlusal => occlusal_plane(&pts).ok_or_else(|| ArticulatorError::Invalid {
            message: "occlusal plane needs ≥3 cusp tips".into(),
        })?,
        PlaneKind::LeastSquares => fit_plane(&pts).ok_or_else(|| ArticulatorError::Invalid {
            message: "least-squares plane needs ≥3 points".into(),
        })?,
    };
    Ok(FitPlaneResponse {
        plane,
        backend: "tlanticad-articulator::planes",
    })
}
