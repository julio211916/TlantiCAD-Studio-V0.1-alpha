//! Native mesh operations using Rust

use app_core::types::{MeshData, Vec3};
use nalgebra as na;


/// Calculate mesh bounds
pub fn calculate_bounds(mesh: &MeshData) -> (Vec3, Vec3) {
    let mut min = Vec3::new(f64::MAX, f64::MAX, f64::MAX);
    let mut max = Vec3::new(f64::MIN, f64::MIN, f64::MIN);

    for i in 0..mesh.vertex_count() {
        let idx = i * 3;
        let x = mesh.vertices[idx] as f64;
        let y = mesh.vertices[idx + 1] as f64;
        let z = mesh.vertices[idx + 2] as f64;

        min.x = min.x.min(x);
        min.y = min.y.min(y);
        min.z = min.z.min(z);

        max.x = max.x.max(x);
        max.y = max.y.max(y);
        max.z = max.z.max(z);
    }

    (min, max)
}

/// Calculate mesh center
pub fn calculate_center(mesh: &MeshData) -> Vec3 {
    let (min, max) = calculate_bounds(mesh);
    Vec3::new(
        (min.x + max.x) / 2.0,
        (min.y + max.y) / 2.0,
        (min.z + max.z) / 2.0,
    )
}

/// Recalculate normals for mesh
pub fn recalculate_normals(mesh: &mut MeshData) {
    let vertex_count = mesh.vertex_count();
    mesh.normals = vec![0.0; vertex_count * 3];

    // Calculate face normals and accumulate
    for i in 0..mesh.face_count() {
        let idx = i * 3;
        let i0 = mesh.indices[idx] as usize;
        let i1 = mesh.indices[idx + 1] as usize;
        let i2 = mesh.indices[idx + 2] as usize;

        let v0 = na::Vector3::new(
            mesh.vertices[i0 * 3],
            mesh.vertices[i0 * 3 + 1],
            mesh.vertices[i0 * 3 + 2],
        );
        let v1 = na::Vector3::new(
            mesh.vertices[i1 * 3],
            mesh.vertices[i1 * 3 + 1],
            mesh.vertices[i1 * 3 + 2],
        );
        let v2 = na::Vector3::new(
            mesh.vertices[i2 * 3],
            mesh.vertices[i2 * 3 + 1],
            mesh.vertices[i2 * 3 + 2],
        );

        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let normal = edge1.cross(&edge2);

        // Add normal to all vertices of face
        for vi in [i0, i1, i2] {
            mesh.normals[vi * 3] += normal.x;
            mesh.normals[vi * 3 + 1] += normal.y;
            mesh.normals[vi * 3 + 2] += normal.z;
        }
    }

    // Normalize all normals
    for i in 0..vertex_count {
        let idx = i * 3;
        let n = na::Vector3::new(
            mesh.normals[idx],
            mesh.normals[idx + 1],
            mesh.normals[idx + 2],
        );
        let normalized = n.normalize();
        mesh.normals[idx] = normalized.x;
        mesh.normals[idx + 1] = normalized.y;
        mesh.normals[idx + 2] = normalized.z;
    }
}

/// Transform mesh by matrix
pub fn transform_mesh(mesh: &mut MeshData, matrix: &[[f32; 4]; 4]) {
    let transform = na::Matrix4::new(
        matrix[0][0], matrix[0][1], matrix[0][2], matrix[0][3],
        matrix[1][0], matrix[1][1], matrix[1][2], matrix[1][3],
        matrix[2][0], matrix[2][1], matrix[2][2], matrix[2][3],
        matrix[3][0], matrix[3][1], matrix[3][2], matrix[3][3],
    );

    // Transform vertices
    for i in 0..mesh.vertex_count() {
        let idx = i * 3;
        let point = na::Point3::new(
            mesh.vertices[idx],
            mesh.vertices[idx + 1],
            mesh.vertices[idx + 2],
        );
        let transformed = transform.transform_point(&point);
        mesh.vertices[idx] = transformed.x;
        mesh.vertices[idx + 1] = transformed.y;
        mesh.vertices[idx + 2] = transformed.z;
    }

    // Transform normals (using inverse transpose for correct normal transformation)
    if !mesh.normals.is_empty() {
        let normal_matrix = transform.fixed_view::<3, 3>(0, 0).try_inverse().map(|m| m.transpose());
        
        if let Some(nm) = normal_matrix {
            for i in 0..mesh.normals.len() / 3 {
                let idx = i * 3;
                let normal = na::Vector3::new(
                    mesh.normals[idx],
                    mesh.normals[idx + 1],
                    mesh.normals[idx + 2],
                );
                let transformed = (nm * normal).normalize();
                mesh.normals[idx] = transformed.x;
                mesh.normals[idx + 1] = transformed.y;
                mesh.normals[idx + 2] = transformed.z;
            }
        }
    }
}

/// Scale mesh uniformly
pub fn scale_mesh(mesh: &mut MeshData, scale: f32) {
    for v in mesh.vertices.iter_mut() {
        *v *= scale;
    }
}

/// Center mesh at origin
pub fn center_mesh(mesh: &mut MeshData) {
    let center = calculate_center(mesh);

    for i in 0..mesh.vertex_count() {
        let idx = i * 3;
        mesh.vertices[idx] -= center.x as f32;
        mesh.vertices[idx + 1] -= center.y as f32;
        mesh.vertices[idx + 2] -= center.z as f32;
    }
}

/// Flip mesh normals
pub fn flip_normals(mesh: &mut MeshData) {
    for n in mesh.normals.iter_mut() {
        *n = -*n;
    }

    // Also reverse face winding
    for i in 0..mesh.face_count() {
        let idx = i * 3;
        mesh.indices.swap(idx + 1, idx + 2);
    }
}

/// Calculate mesh volume (for closed meshes)
pub fn calculate_volume(mesh: &MeshData) -> f64 {
    let mut volume = 0.0;

    for i in 0..mesh.face_count() {
        let idx = i * 3;
        let i0 = mesh.indices[idx] as usize;
        let i1 = mesh.indices[idx + 1] as usize;
        let i2 = mesh.indices[idx + 2] as usize;

        let v0 = na::Vector3::new(
            mesh.vertices[i0 * 3] as f64,
            mesh.vertices[i0 * 3 + 1] as f64,
            mesh.vertices[i0 * 3 + 2] as f64,
        );
        let v1 = na::Vector3::new(
            mesh.vertices[i1 * 3] as f64,
            mesh.vertices[i1 * 3 + 1] as f64,
            mesh.vertices[i1 * 3 + 2] as f64,
        );
        let v2 = na::Vector3::new(
            mesh.vertices[i2 * 3] as f64,
            mesh.vertices[i2 * 3 + 1] as f64,
            mesh.vertices[i2 * 3 + 2] as f64,
        );

        volume += v0.dot(&v1.cross(&v2));
    }

    (volume / 6.0).abs()
}

/// Calculate mesh surface area
pub fn calculate_surface_area(mesh: &MeshData) -> f64 {
    let mut area = 0.0;

    for i in 0..mesh.face_count() {
        let idx = i * 3;
        let i0 = mesh.indices[idx] as usize;
        let i1 = mesh.indices[idx + 1] as usize;
        let i2 = mesh.indices[idx + 2] as usize;

        let v0 = na::Vector3::new(
            mesh.vertices[i0 * 3] as f64,
            mesh.vertices[i0 * 3 + 1] as f64,
            mesh.vertices[i0 * 3 + 2] as f64,
        );
        let v1 = na::Vector3::new(
            mesh.vertices[i1 * 3] as f64,
            mesh.vertices[i1 * 3 + 1] as f64,
            mesh.vertices[i1 * 3 + 2] as f64,
        );
        let v2 = na::Vector3::new(
            mesh.vertices[i2 * 3] as f64,
            mesh.vertices[i2 * 3 + 1] as f64,
            mesh.vertices[i2 * 3 + 2] as f64,
        );

        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        area += edge1.cross(&edge2).magnitude() / 2.0;
    }

    area
}
