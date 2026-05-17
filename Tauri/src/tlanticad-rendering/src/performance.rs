//! S231-S235: Performance rendering — LOD, frustum culling, instancing, batching

use nalgebra::{Matrix4, Point3, Vector4};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ────────────────────────────────────────────────────────────────────
//  Level of Detail (LOD)
// ────────────────────────────────────────────────────────────────────

/// A single LOD level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LodLevel {
    pub level: u32,
    /// Maximum screen-space error (pixels) before switching to next level
    pub max_error: f32,
    pub triangle_count: u32,
    pub mesh_id: Uuid,
}

/// LOD group attached to a scene node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LodGroup {
    pub node_id: Uuid,
    pub levels: Vec<LodLevel>,
    pub current_level: u32,
}

impl LodGroup {
    pub fn new(node_id: Uuid, levels: Vec<LodLevel>) -> Self {
        Self { node_id, levels, current_level: 0 }
    }

    /// Select appropriate LOD level based on distance from camera
    pub fn select_level(&mut self, camera_pos: &Point3<f64>, object_pos: &Point3<f64>) -> u32 {
        let dist = nalgebra::distance(camera_pos, object_pos) as f32;
        let mut selected = 0;
        for level in &self.levels {
            if dist > level.max_error * 10.0 {
                selected = level.level;
            }
        }
        self.current_level = selected.min(self.levels.len().saturating_sub(1) as u32);
        self.current_level
    }

    pub fn active_triangle_count(&self) -> u32 {
        self.levels
            .get(self.current_level as usize)
            .map(|l| l.triangle_count)
            .unwrap_or(0)
    }
}

// ────────────────────────────────────────────────────────────────────
//  Frustum culling
// ────────────────────────────────────────────────────────────────────

/// The six planes of a view frustum (ax + by + cz + d >= 0 = inside)
#[derive(Debug, Clone)]
pub struct Frustum {
    pub planes: [[f64; 4]; 6],
}

impl Frustum {
    /// Extract frustum planes from a combined view-projection matrix (row-major)
    pub fn from_view_proj(vp: &Matrix4<f64>) -> Self {
        let row = |i: usize| -> Vector4<f64> {
            Vector4::new(vp[(i, 0)], vp[(i, 1)], vp[(i, 2)], vp[(i, 3)])
        };
        let r0 = row(0); let r1 = row(1); let r2 = row(2); let r3 = row(3);
        let planes_raw = [
            r3 + r0,       // left
            r3 - r0,       // right
            r3 + r1,       // bottom
            r3 - r1,       // top
            r3 + r2,       // near
            r3 - r2,       // far
        ];
        let mut planes = [[0.0f64; 4]; 6];
        for (i, p) in planes_raw.iter().enumerate() {
            let len = (p.x * p.x + p.y * p.y + p.z * p.z).sqrt();
            if len > 1e-12 {
                planes[i] = [p.x / len, p.y / len, p.z / len, p.w / len];
            }
        }
        Self { planes }
    }

    /// Axis-Aligned Bounding Box intersection test.
    /// Returns true if the AABB is at least partially inside the frustum.
    pub fn intersects_aabb(&self, min: &Point3<f64>, max: &Point3<f64>) -> bool {
        for plane in &self.planes {
            let px = if plane[0] >= 0.0 { max.x } else { min.x };
            let py = if plane[1] >= 0.0 { max.y } else { min.y };
            let pz = if plane[2] >= 0.0 { max.z } else { min.z };
            if plane[0] * px + plane[1] * py + plane[2] * pz + plane[3] < 0.0 {
                return false;
            }
        }
        true
    }

    /// Sphere intersection test
    pub fn intersects_sphere(&self, center: &Point3<f64>, radius: f64) -> bool {
        for plane in &self.planes {
            let dist = plane[0] * center.x + plane[1] * center.y + plane[2] * center.z + plane[3];
            if dist < -radius { return false; }
        }
        true
    }
}

// ────────────────────────────────────────────────────────────────────
//  Instanced rendering
// ────────────────────────────────────────────────────────────────────

/// A single instance transform for instanced rendering
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct InstanceData {
    pub model_matrix: [[f32; 4]; 4],
    pub color_tint: [f32; 4],
    pub id: u32,
}

/// Instanced draw call descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstancedDrawCall {
    pub mesh_id: Uuid,
    pub instances: Vec<InstanceData>,
    pub pipeline_label: String,
}

impl InstancedDrawCall {
    pub fn instance_count(&self) -> usize {
        self.instances.len()
    }
}

// ────────────────────────────────────────────────────────────────────
//  Draw batching
// ────────────────────────────────────────────────────────────────────

/// A batch of draw calls grouped by pipeline and material
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderBatch {
    pub pipeline_label: String,
    pub material_id: Option<Uuid>,
    pub mesh_ids: Vec<Uuid>,
    pub total_triangles: u32,
}

/// Statistics from a render pass
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RenderStats {
    pub draw_calls: u32,
    pub triangles_rendered: u32,
    pub triangles_culled: u32,
    pub instances_rendered: u32,
    pub batches: u32,
    pub frame_time_ms: f64,
    pub gpu_memory_mb: f64,
}

impl RenderStats {
    pub fn culling_ratio(&self) -> f64 {
        let total = self.triangles_rendered + self.triangles_culled;
        if total == 0 { return 0.0; }
        self.triangles_culled as f64 / total as f64
    }
}

// ────────────────────────────────────────────────────────────────────
//  GPU Picking
// ────────────────────────────────────────────────────────────────────

/// GPU color-coding picking info
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PickId {
    pub node_index: u16,
    pub face_index: u16,
}

impl PickId {
    /// Encode as a unique color (RGBA u8)
    pub fn to_color(&self) -> [u8; 4] {
        let n = self.node_index;
        let f = self.face_index;
        [(n >> 8) as u8, (n & 0xFF) as u8, (f >> 8) as u8, (f & 0xFF) as u8]
    }

    /// Decode from pixel color
    pub fn from_color(c: [u8; 4]) -> Self {
        Self {
            node_index: ((c[0] as u16) << 8) | c[1] as u16,
            face_index: ((c[2] as u16) << 8) | c[3] as u16,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lod_selection() {
        let mut lod = LodGroup::new(Uuid::new_v4(), vec![
            LodLevel { level: 0, max_error: 1.0, triangle_count: 10000, mesh_id: Uuid::new_v4() },
            LodLevel { level: 1, max_error: 5.0, triangle_count: 2000, mesh_id: Uuid::new_v4() },
            LodLevel { level: 2, max_error: 20.0, triangle_count: 500, mesh_id: Uuid::new_v4() },
        ]);
        let cam = Point3::new(0.0, 0.0, 5.0);
        let obj = Point3::origin();
        let lvl = lod.select_level(&cam, &obj);
        // Distance is 5, max_error*10 thresholds: 10, 50, 200
        assert_eq!(lvl, 0);

        let cam_far = Point3::new(0.0, 0.0, 100.0);
        let lvl2 = lod.select_level(&cam_far, &obj);
        assert!(lvl2 >= 1);
    }

    #[test]
    fn test_frustum_sphere() {
        let vp = Matrix4::new_perspective(1.0, 45.0_f64.to_radians(), 0.1, 100.0);
        let frustum = Frustum::from_view_proj(&vp);
        // Origin should be inside
        assert!(frustum.intersects_sphere(&Point3::origin(), 1.0));
    }

    #[test]
    fn test_pick_id_roundtrip() {
        let pick = PickId { node_index: 1234, face_index: 5678 };
        let color = pick.to_color();
        let decoded = PickId::from_color(color);
        assert_eq!(pick.node_index, decoded.node_index);
        assert_eq!(pick.face_index, decoded.face_index);
    }

    #[test]
    fn test_render_stats() {
        let stats = RenderStats {
            triangles_rendered: 7000,
            triangles_culled: 3000,
            ..Default::default()
        };
        assert!((stats.culling_ratio() - 0.3).abs() < 0.01);
    }

    #[test]
    fn test_instanced_draw_call() {
        let call = InstancedDrawCall {
            mesh_id: Uuid::new_v4(),
            instances: vec![
                InstanceData {
                    model_matrix: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0],
                                   [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]],
                    color_tint: [1.0; 4],
                    id: 0,
                },
            ],
            pipeline_label: "default".into(),
        };
        assert_eq!(call.instance_count(), 1);
    }
}
