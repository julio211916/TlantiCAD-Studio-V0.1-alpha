//! S206-S210: Advanced viewport — quad view, camera presets, fit-to-view, animation

use nalgebra::Point3;
use serde::{Deserialize, Serialize};
use crate::camera::OrbitCamera;
use crate::viewport::Viewport;

/// Predefined camera view angles for dental inspection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CameraPreset {
    Front,
    Back,
    Left,
    Right,
    Top,
    Bottom,
    IsometricFrontLeft,
    IsometricFrontRight,
    Occlusal,     // top-down for occlusal view
    Buccal,       // side view
    Lingual,      // inner side
}

impl CameraPreset {
    /// Azimuth and elevation angles (radians) for each preset
    pub fn angles(self) -> (f64, f64) {
        use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI};
        match self {
            Self::Front              => (0.0, 0.0),
            Self::Back               => (PI, 0.0),
            Self::Left               => (-FRAC_PI_2, 0.0),
            Self::Right              => (FRAC_PI_2, 0.0),
            Self::Top | Self::Occlusal => (0.0, FRAC_PI_2 - 0.01),
            Self::Bottom             => (0.0, -FRAC_PI_2 + 0.01),
            Self::IsometricFrontLeft => (-FRAC_PI_4, FRAC_PI_4 * 0.8),
            Self::IsometricFrontRight=> (FRAC_PI_4, FRAC_PI_4 * 0.8),
            Self::Buccal             => (FRAC_PI_2, 0.1),
            Self::Lingual            => (-FRAC_PI_2, 0.1),
        }
    }

    /// Apply preset to an OrbitCamera
    pub fn apply(self, cam: &mut OrbitCamera) {
        let (az, el) = self.angles();
        cam.azimuth = az;
        cam.elevation = el;
    }
}

/// Viewport configuration for quad-view
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ViewportLayout {
    Single,
    DualHorizontal,
    DualVertical,
    Quad,
}

/// A viewport slot in a multi-view layout
#[derive(Debug, Clone)]
pub struct ViewportSlot {
    pub index: usize,
    pub viewport: Viewport,
    pub camera: OrbitCamera,
    pub preset: Option<CameraPreset>,
    pub active: bool,
}

/// Multi-viewport manager
#[derive(Debug, Clone)]
pub struct MultiViewport {
    pub layout: ViewportLayout,
    pub slots: Vec<ViewportSlot>,
    pub active_slot: usize,
    pub total_width: u32,
    pub total_height: u32,
}

impl MultiViewport {
    /// Create a single full-screen viewport
    pub fn single(width: u32, height: u32) -> Self {
        Self {
            layout: ViewportLayout::Single,
            slots: vec![ViewportSlot {
                index: 0,
                viewport: Viewport { width, height, ..Default::default() },
                camera: OrbitCamera::default(),
                preset: None,
                active: true,
            }],
            active_slot: 0,
            total_width: width,
            total_height: height,
        }
    }

    /// Create quad-view: front, right, top, isometric
    pub fn quad(width: u32, height: u32) -> Self {
        let hw = width / 2;
        let hh = height / 2;
        let presets = [
            CameraPreset::Front,
            CameraPreset::Right,
            CameraPreset::Top,
            CameraPreset::IsometricFrontRight,
        ];
        let slots = presets.iter().enumerate().map(|(i, &p)| {
            let mut cam = OrbitCamera::default();
            p.apply(&mut cam);
            ViewportSlot {
                index: i,
                viewport: Viewport { width: hw, height: hh, ..Default::default() },
                camera: cam,
                preset: Some(p),
                active: i == 3, // isometric is active by default
            }
        }).collect();

        Self {
            layout: ViewportLayout::Quad,
            slots,
            active_slot: 3,
            total_width: width,
            total_height: height,
        }
    }

    /// Set the active viewport based on screen click position
    pub fn pick_viewport(&mut self, x: u32, y: u32) -> usize {
        match self.layout {
            ViewportLayout::Single => { self.active_slot = 0; 0 }
            ViewportLayout::Quad => {
                let half_w = self.total_width / 2;
                let half_h = self.total_height / 2;
                let slot = if x < half_w && y < half_h { 0 }
                    else if x >= half_w && y < half_h { 1 }
                    else if x < half_w && y >= half_h { 2 }
                    else { 3 };
                self.active_slot = slot;
                slot
            }
            _ => { self.active_slot = 0; 0 }
        }
    }
}

/// Bounding sphere for fit-to-view computation
#[derive(Debug, Clone, Copy)]
pub struct BoundingSphere {
    pub center: Point3<f64>,
    pub radius: f64,
}

impl BoundingSphere {
    /// Compute from axis-aligned bounding box corners
    pub fn from_aabb(min: Point3<f64>, max: Point3<f64>) -> Self {
        let center = nalgebra::center(&min, &max);
        let radius = nalgebra::distance(&center, &max);
        Self { center, radius }
    }

    /// Fit camera to show the full bounding sphere
    pub fn fit_camera(&self, cam: &mut OrbitCamera, fov_deg: f64) {
        cam.target = self.center;
        let fov_rad = fov_deg.to_radians();
        cam.distance = self.radius / (fov_rad / 2.0).sin();
    }
}

/// Smooth camera animation state
#[derive(Debug, Clone)]
pub struct CameraAnimation {
    pub start_azimuth: f64,
    pub start_elevation: f64,
    pub start_distance: f64,
    pub start_target: Point3<f64>,
    pub end_azimuth: f64,
    pub end_elevation: f64,
    pub end_distance: f64,
    pub end_target: Point3<f64>,
    pub progress: f64,     // 0..1
    pub duration_secs: f64,
    pub active: bool,
}

impl CameraAnimation {
    pub fn new(cam: &OrbitCamera, preset: CameraPreset, duration: f64) -> Self {
        let (az, el) = preset.angles();
        Self {
            start_azimuth: cam.azimuth,
            start_elevation: cam.elevation,
            start_distance: cam.distance,
            start_target: cam.target,
            end_azimuth: az,
            end_elevation: el,
            end_distance: cam.distance,
            end_target: cam.target,
            progress: 0.0,
            duration_secs: duration,
            active: true,
        }
    }

    /// Ease-in-out cubic interpolation
    fn ease(t: f64) -> f64 {
        if t < 0.5 { 4.0 * t * t * t }
        else { 1.0 - (-2.0 * t + 2.0_f64).powi(3) / 2.0 }
    }

    /// Advance by `dt` seconds and apply to camera. Returns true if animation still active.
    pub fn update(&mut self, cam: &mut OrbitCamera, dt: f64) -> bool {
        if !self.active { return false; }
        self.progress = (self.progress + dt / self.duration_secs).min(1.0);
        let t = Self::ease(self.progress);
        cam.azimuth = self.start_azimuth + (self.end_azimuth - self.start_azimuth) * t;
        cam.elevation = self.start_elevation + (self.end_elevation - self.start_elevation) * t;
        cam.distance = self.start_distance + (self.end_distance - self.start_distance) * t;
        cam.target = Point3::from(
            self.start_target.coords + (self.end_target.coords - self.start_target.coords) * t
        );
        if self.progress >= 1.0 { self.active = false; }
        self.active
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_presets() {
        let mut cam = OrbitCamera::default();
        CameraPreset::Top.apply(&mut cam);
        assert!((cam.elevation - (std::f64::consts::FRAC_PI_2 - 0.01)).abs() < 0.001);
    }

    #[test]
    fn test_quad_viewport() {
        let mv = MultiViewport::quad(1920, 1080);
        assert_eq!(mv.slots.len(), 4);
        assert_eq!(mv.slots[0].viewport.width, 960);
        assert_eq!(mv.slots[0].viewport.height, 540);
        assert_eq!(mv.active_slot, 3);
    }

    #[test]
    fn test_bounding_sphere() {
        let bs = BoundingSphere::from_aabb(
            Point3::new(-10.0, -10.0, -10.0),
            Point3::new(10.0, 10.0, 10.0),
        );
        assert!((bs.center.x).abs() < 0.001);
        assert!((bs.radius - 17.32).abs() < 0.1);
    }

    #[test]
    fn test_fit_camera() {
        let bs = BoundingSphere {
            center: Point3::origin(),
            radius: 50.0,
        };
        let mut cam = OrbitCamera::default();
        bs.fit_camera(&mut cam, 45.0);
        assert!(cam.distance > 50.0);
        assert_eq!(cam.target, Point3::origin());
    }

    #[test]
    fn test_camera_animation() {
        let cam = OrbitCamera::default();
        let mut anim = CameraAnimation::new(&cam, CameraPreset::Top, 1.0);
        let mut cam2 = cam.clone();
        assert!(anim.active);
        anim.update(&mut cam2, 1.0);
        assert!(!anim.active);
        assert!((cam2.elevation - (std::f64::consts::FRAC_PI_2 - 0.01)).abs() < 0.01);
    }

    #[test]
    fn test_pick_viewport_quad() {
        let mut mv = MultiViewport::quad(1920, 1080);
        assert_eq!(mv.pick_viewport(100, 100), 0);  // top-left
        assert_eq!(mv.pick_viewport(1000, 100), 1);  // top-right
        assert_eq!(mv.pick_viewport(100, 600), 2);   // bottom-left
        assert_eq!(mv.pick_viewport(1000, 600), 3);  // bottom-right
    }
}
