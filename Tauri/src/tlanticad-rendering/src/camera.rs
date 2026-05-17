//! Orbit camera for 3D viewport

use nalgebra::{Matrix4, Point3, Vector3};

/// Orbit camera that rotates around a target point
#[derive(Debug, Clone)]
pub struct OrbitCamera {
    pub target: Point3<f64>,
    pub distance: f64,
    pub azimuth: f64,   // horizontal angle in radians
    pub elevation: f64, // vertical angle in radians
    pub up: Vector3<f64>,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            target: Point3::origin(),
            distance: 100.0,
            azimuth: 0.0,
            elevation: 0.3,
            up: Vector3::y(),
        }
    }
}

impl OrbitCamera {
    pub fn new(target: Point3<f64>, distance: f64) -> Self {
        Self { target, distance, ..Default::default() }
    }

    /// Current camera position
    pub fn position(&self) -> Point3<f64> {
        let x = self.distance * self.elevation.cos() * self.azimuth.sin();
        let y = self.distance * self.elevation.sin();
        let z = self.distance * self.elevation.cos() * self.azimuth.cos();
        self.target + Vector3::new(x, y, z)
    }

    /// View matrix (look-at)
    pub fn view_matrix(&self) -> Matrix4<f64> {
        let eye = self.position();
        let f = (self.target - eye).normalize();
        let s = f.cross(&self.up).normalize();
        let u = s.cross(&f);
        Matrix4::new(
            s.x, s.y, s.z, -s.dot(&eye.coords),
            u.x, u.y, u.z, -u.dot(&eye.coords),
            -f.x, -f.y, -f.z, f.dot(&eye.coords),
            0.0, 0.0, 0.0, 1.0,
        )
    }

    /// Orbit (rotate around target)
    pub fn orbit(&mut self, delta_azimuth: f64, delta_elevation: f64) {
        self.azimuth += delta_azimuth;
        self.elevation = (self.elevation + delta_elevation)
            .clamp(-std::f64::consts::FRAC_PI_2 + 0.01, std::f64::consts::FRAC_PI_2 - 0.01);
    }

    /// Zoom (change distance)
    pub fn zoom(&mut self, delta: f64) {
        self.distance = (self.distance - delta).max(0.1);
    }

    /// Pan (move target in screen plane)
    pub fn pan(&mut self, dx: f64, dy: f64) {
        let eye = self.position();
        let forward = (self.target - eye).normalize();
        let right = forward.cross(&self.up).normalize();
        let up = right.cross(&forward);
        self.target += right * dx + up * dy;
    }

    /// Focus on a bounding box
    pub fn focus_on(&mut self, center: Point3<f64>, radius: f64) {
        self.target = center;
        self.distance = radius * 2.5;
    }
}
