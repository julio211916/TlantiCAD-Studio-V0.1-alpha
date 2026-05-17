//! S132: Undercut visualization — color map for undercut areas.

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};

/// Undercut analysis map.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndercutMap {
    /// Per-vertex undercut depth (0.0 = no undercut).
    pub per_vertex_depth: Vec<f64>,
    /// Total undercut area estimate (mm²).
    pub total_area: f64,
    /// Maximum undercut depth (mm).
    pub max_depth: f64,
}

/// Compute undercut map for a mesh given an insertion axis.
pub fn compute_undercut_map(
    vertices: &[Point3<f64>],
    normals: &[Vector3<f64>],
    axis: &Vector3<f64>,
) -> UndercutMap {
    let dir = axis.normalize();
    let mut per_vertex_depth = Vec::with_capacity(vertices.len());
    let mut total_area = 0.0;
    let mut max_depth = 0.0f64;

    for (_vi, normal) in normals.iter().enumerate() {
        let dot = normal.dot(&dir);
        if dot < 0.0 {
            let depth = -dot;
            per_vertex_depth.push(depth);
            total_area += depth; // simplified area contribution
            max_depth = max_depth.max(depth);
        } else {
            per_vertex_depth.push(0.0);
        }
    }

    UndercutMap { per_vertex_depth, total_area, max_depth }
}

/// Convert undercut map to RGB colors per vertex for visualization.
/// Green = no undercut, Yellow = mild, Red = severe.
pub fn undercut_to_colors(map: &UndercutMap) -> Vec<[f32; 3]> {
    let threshold = map.max_depth * 0.5;
    map.per_vertex_depth
        .iter()
        .map(|&d| {
            if d <= 0.0 {
                [0.0, 0.8, 0.0] // green
            } else if d < threshold {
                let t = (d / threshold) as f32;
                [t, 0.8 * (1.0 - t), 0.0] // green→yellow→orange
            } else {
                [1.0, 0.0, 0.0] // red
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_undercut() {
        let verts = vec![Point3::new(0.0, 0.0, 0.0)];
        let normals = vec![Vector3::new(0.0, 0.0, 1.0)];
        let axis = Vector3::new(0.0, 0.0, 1.0);
        let map = compute_undercut_map(&verts, &normals, &axis);
        assert_eq!(map.per_vertex_depth[0], 0.0);
        assert_eq!(map.max_depth, 0.0);
    }

    #[test]
    fn with_undercut() {
        let verts = vec![Point3::new(0.0, 0.0, 0.0)];
        let normals = vec![Vector3::new(0.0, 0.0, -1.0)]; // opposing
        let axis = Vector3::new(0.0, 0.0, 1.0);
        let map = compute_undercut_map(&verts, &normals, &axis);
        assert!(map.per_vertex_depth[0] > 0.0);
        assert!(map.max_depth > 0.0);
    }

    #[test]
    fn colors_len() {
        let map = UndercutMap {
            per_vertex_depth: vec![0.0, 0.5, 1.0],
            total_area: 1.5,
            max_depth: 1.0,
        };
        let colors = undercut_to_colors(&map);
        assert_eq!(colors.len(), 3);
    }
}
