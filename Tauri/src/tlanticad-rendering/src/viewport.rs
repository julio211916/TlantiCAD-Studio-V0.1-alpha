//! Viewport management: size, projection, coordinate transformations

use nalgebra::Matrix4;

/// Viewport describes the render target dimensions and projection
#[derive(Debug, Clone)]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
    pub fov: f64,
    pub near: f64,
    pub far: f64,
}

impl Default for Viewport {
    fn default() -> Self {
        Self { width: 1280, height: 720, fov: 45.0, near: 0.1, far: 10000.0 }
    }
}

impl Viewport {
    pub fn aspect_ratio(&self) -> f64 {
        self.width as f64 / self.height.max(1) as f64
    }

    /// Perspective projection matrix
    pub fn perspective_matrix(&self) -> Matrix4<f64> {
        let fov_rad = self.fov.to_radians();
        let f = 1.0 / (fov_rad / 2.0).tan();
        let aspect = self.aspect_ratio();
        let nf = self.near - self.far;
        Matrix4::new(
            f / aspect, 0.0, 0.0, 0.0,
            0.0, f, 0.0, 0.0,
            0.0, 0.0, (self.far + self.near) / nf, 2.0 * self.far * self.near / nf,
            0.0, 0.0, -1.0, 0.0,
        )
    }

    /// Orthographic projection matrix
    pub fn ortho_matrix(&self, zoom: f64) -> Matrix4<f64> {
        let hw = (self.width as f64 / 2.0) / zoom;
        let hh = (self.height as f64 / 2.0) / zoom;
        Matrix4::new(
            1.0 / hw, 0.0, 0.0, 0.0,
            0.0, 1.0 / hh, 0.0, 0.0,
            0.0, 0.0, -2.0 / (self.far - self.near), -(self.far + self.near) / (self.far - self.near),
            0.0, 0.0, 0.0, 1.0,
        )
    }

    /// Convert screen coordinates (pixels) to NDC [-1, 1]
    pub fn screen_to_ndc(&self, x: f64, y: f64) -> (f64, f64) {
        let nx = (2.0 * x / self.width as f64) - 1.0;
        let ny = 1.0 - (2.0 * y / self.height as f64);
        (nx, ny)
    }

    /// Convert NDC to screen coordinates
    pub fn ndc_to_screen(&self, nx: f64, ny: f64) -> (f64, f64) {
        let x = (nx + 1.0) * self.width as f64 / 2.0;
        let y = (1.0 - ny) * self.height as f64 / 2.0;
        (x, y)
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }
}
