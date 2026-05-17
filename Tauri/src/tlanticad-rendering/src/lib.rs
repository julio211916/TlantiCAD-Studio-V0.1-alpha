//! TlantiCAD 3D Rendering Module

pub mod viewport;
pub mod shader;
pub mod camera;
pub mod scene;
pub mod material;
pub mod light;
pub mod picker;
pub mod screenshot;

// S201-S210: Render pipeline & multi-viewport
pub mod pipeline;
pub mod multi_viewport;

// S211-S215: Dental shaders, SSAO, shadows, env lighting
pub mod dental_shaders;

// S216-S220: Selection, gizmo, measurement, clipping
pub mod interaction;

// S221-S225: Visualization modes, color maps, heatmaps
pub mod visualization;

// S231-S245: Performance rendering & annotations
pub mod performance;
pub mod annotation;

pub struct Renderer {
    pub width: u32,
    pub height: u32,
}

impl Renderer {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}
