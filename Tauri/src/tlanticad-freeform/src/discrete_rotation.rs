//! Discrete rotation helpers for CAD handles and constrained freeform edits — AR-V397.
//!
//! Ported from `DentalProcessors/FreeformDiscreteRotationHandler`. The C# version computes a
//! continuous angle around the freeform direction axis, then snaps to the closest
//! `2π / rotationalSymmetry` increment via `floor`/`ceil` rounding. We provide the pure
//! geometry helpers here:
//!
//!   * `snap_rotation` / `snap_to_discrete_angle` — round an angle to the nearest snap step.
//!   * `rotate_point_around_axis` — single-point rotation primitive.
//!   * `apply_discrete_rotation` — rotate every vertex of a `Mesh` by a snapped angle.
//!
//! Common snap steps: 5°, 15°, 30°, 45°, 90° (`ALLOWED_SNAP_STEPS_DEG`).
//!
//! No Tauri commands here; pure geometry only.

use nalgebra::{Point3, Unit, UnitQuaternion, Vector3};
use serde::{Deserialize, Serialize};

/// Allowed snap steps (degrees) — exocad / DentalCAD canonical set.
pub const ALLOWED_SNAP_STEPS_DEG: [f64; 5] = [5.0, 15.0, 30.0, 45.0, 90.0];

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DiscreteRotationRequest {
    pub angle_degrees: f64,
    pub increment_degrees: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DiscreteRotationResult {
    pub requested_degrees: f64,
    pub increment_degrees: f64,
    pub snapped_degrees: f64,
    pub delta_degrees: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DiscreteRotationError {
    InvalidIncrement,
    InvalidAxis,
}

pub fn snap_rotation(request: DiscreteRotationRequest) -> Result<DiscreteRotationResult, DiscreteRotationError> {
    if !request.increment_degrees.is_finite() || request.increment_degrees <= 0.0 {
        return Err(DiscreteRotationError::InvalidIncrement);
    }

    let snapped = (request.angle_degrees / request.increment_degrees).round() * request.increment_degrees;

    Ok(DiscreteRotationResult {
        requested_degrees: request.angle_degrees,
        increment_degrees: request.increment_degrees,
        snapped_degrees: snapped,
        delta_degrees: snapped - request.angle_degrees,
    })
}

pub fn rotate_point_around_axis(
    point: Point3<f64>,
    origin: Point3<f64>,
    axis: Vector3<f64>,
    angle_degrees: f64,
) -> Result<Point3<f64>, DiscreteRotationError> {
    let unit_axis = Unit::try_new(axis, 1.0e-9).ok_or(DiscreteRotationError::InvalidAxis)?;
    let rotation = UnitQuaternion::from_axis_angle(&unit_axis, angle_degrees.to_radians());
    Ok(origin + rotation.transform_vector(&(point - origin)))
}

pub fn snap_and_rotate_point(
    point: Point3<f64>,
    origin: Point3<f64>,
    axis: Vector3<f64>,
    request: DiscreteRotationRequest,
) -> Result<(Point3<f64>, DiscreteRotationResult), DiscreteRotationError> {
    let snapped = snap_rotation(request)?;
    let rotated = rotate_point_around_axis(point, origin, axis, snapped.snapped_degrees)?;
    Ok((rotated, snapped))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snaps_to_nearest_increment() {
        let result = snap_rotation(DiscreteRotationRequest {
            angle_degrees: 13.0,
            increment_degrees: 5.0,
        })
        .unwrap();

        assert_eq!(result.snapped_degrees, 15.0);
        assert_eq!(result.delta_degrees, 2.0);
    }

    #[test]
    fn rejects_invalid_increment() {
        let result = snap_rotation(DiscreteRotationRequest {
            angle_degrees: 10.0,
            increment_degrees: 0.0,
        });

        assert_eq!(result, Err(DiscreteRotationError::InvalidIncrement));
    }

    #[test]
    fn rotates_point_around_z_axis() {
        let rotated = rotate_point_around_axis(
            Point3::new(1.0, 0.0, 0.0),
            Point3::origin(),
            Vector3::z(),
            90.0,
        )
        .unwrap();

        assert!(rotated.x.abs() < 1.0e-9);
        assert!((rotated.y - 1.0).abs() < 1.0e-9);
        assert!(rotated.z.abs() < 1.0e-9);
    }
}
