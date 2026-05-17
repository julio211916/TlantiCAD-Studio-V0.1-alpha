use cad_core::{Mat4f, Point3f, Vec3f};
use std::f32::consts::PI;

/// Spherical-coordinate orbit camera.
///
/// Azimuth   — rotation around the Y-axis (radians).
/// Elevation — angle above the XZ plane (radians, clamped to ±89°).
#[derive(Debug, Clone)]
pub struct OrbitCamera {
    /// The world-space point the camera orbits around.
    pub target: Point3f,
    /// Distance from target to eye (> 0).
    pub distance: f32,
    /// Horizontal angle in radians.
    pub azimuth: f32,
    /// Vertical angle in radians (positive = up).
    pub elevation: f32,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            target: Point3f::origin(),
            distance: 5.0,
            azimuth: 0.0,
            elevation: PI / 6.0, // 30°
        }
    }
}

impl OrbitCamera {
    pub fn new(target: Point3f, distance: f32, azimuth: f32, elevation: f32) -> Self {
        let max_elev = PI / 2.0 - 0.01;
        Self {
            target,
            distance: distance.max(0.001),
            azimuth,
            elevation: elevation.clamp(-max_elev, max_elev),
        }
    }

    /// Eye position in world space.
    pub fn eye(&self) -> Point3f {
        let x = self.distance * self.elevation.cos() * self.azimuth.sin();
        let y = self.distance * self.elevation.sin();
        let z = self.distance * self.elevation.cos() * self.azimuth.cos();
        self.target + Vec3f::new(x, y, z)
    }

    /// Column-major view matrix (world → camera).
    pub fn view_matrix(&self) -> Mat4f {
        let eye = self.eye();
        let up = Vec3f::new(0.0, 1.0, 0.0);
        Mat4f::look_at_rh(&eye, &self.target, &up)
    }

    /// Column-major symmetric perspective projection matrix.
    ///
    /// * `aspect` — viewport width / height
    /// * `fov_y`  — vertical field of view in radians
    /// * `near`   — near plane distance (> 0)
    /// * `far`    — far plane distance  (> near)
    pub fn proj_matrix(&self, aspect: f32, fov_y: f32, near: f32, far: f32) -> Mat4f {
        Mat4f::new_perspective(aspect, fov_y, near, far)
    }

    /// Orbit by `delta_az` / `delta_el` radians (e.g., from mouse drag).
    pub fn orbit(&mut self, delta_az: f32, delta_el: f32) {
        let max_elev = PI / 2.0 - 0.01;
        self.azimuth += delta_az;
        self.elevation = (self.elevation + delta_el).clamp(-max_elev, max_elev);
    }

    /// Zoom in / out: positive `delta` moves closer, negative moves away.
    pub fn zoom(&mut self, delta: f32) {
        self.distance = (self.distance - delta).max(0.001);
    }

    /// Pan the target in the camera's local XY plane.
    pub fn pan(&mut self, dx: f32, dy: f32) {
        let forward = (self.target - self.eye()).normalize();
        let right = forward.cross(&Vec3f::new(0.0, 1.0, 0.0)).normalize();
        let up = right.cross(&forward).normalize();
        self.target += right * dx + up * dy;
    }
}
