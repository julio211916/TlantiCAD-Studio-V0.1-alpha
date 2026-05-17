//! TlantiCAD Alignment Engine.
//!
//! Native Rust port of the Alignment3 workflow concepts:
//! moving object (`mvg_`), destination object (`dstn_`), selected refinement
//! points, remesh/cleanup intent, and ICP registration. Blender object names
//! become typed inputs and deterministic transform artifacts.

use nalgebra::{Matrix3, Matrix4, Point3, Vector3};
use serde::{Deserialize, Serialize};
use tlanticad_mesh::Mesh;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlignmentMode {
    LandmarkRigid,
    MeshCentroid,
    IterativeClosestPoint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlignmentParams {
    pub mode: AlignmentMode,
    pub max_iterations: usize,
    pub tolerance_mm: f64,
    pub sample_limit: usize,
    pub max_correspondence_distance_mm: Option<f64>,
}

impl Default for AlignmentParams {
    fn default() -> Self {
        Self {
            mode: AlignmentMode::IterativeClosestPoint,
            max_iterations: 32,
            tolerance_mm: 1.0e-4,
            sample_limit: 2_000,
            max_correspondence_distance_mm: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlignmentResult {
    pub matrix: [[f64; 4]; 4],
    pub rms_mm: f64,
    pub iterations: usize,
    pub converged: bool,
    pub correspondence_count: usize,
    pub warnings: Vec<String>,
}

pub fn align_landmarks(
    moving_points: &[[f64; 3]],
    fixed_points: &[[f64; 3]],
) -> Result<AlignmentResult, String> {
    if moving_points.len() != fixed_points.len() {
        return Err("movingPoints and fixedPoints must have the same length".to_string());
    }
    if moving_points.len() < 3 {
        return Err("at least 3 point pairs are required for rigid alignment".to_string());
    }

    let moving = points_from_arrays(moving_points);
    let fixed = points_from_arrays(fixed_points);
    let transform = estimate_rigid_transform(&moving, &fixed)?;
    let transformed = transform_points(&moving, &transform);
    let rms = rms_error(&transformed, &fixed);

    Ok(AlignmentResult {
        matrix: matrix_to_array(&transform),
        rms_mm: rms,
        iterations: 1,
        converged: rms.is_finite(),
        correspondence_count: moving.len(),
        warnings: Vec::new(),
    })
}

pub fn align_meshes(
    moving_mesh: &Mesh,
    fixed_mesh: &Mesh,
    params: &AlignmentParams,
) -> Result<AlignmentResult, String> {
    if moving_mesh.vertices.is_empty() {
        return Err("moving mesh has no vertices".to_string());
    }
    if fixed_mesh.vertices.is_empty() {
        return Err("fixed mesh has no vertices".to_string());
    }

    match params.mode {
        AlignmentMode::LandmarkRigid => {
            Err("LandmarkRigid needs explicit point pairs; use align_landmarks".to_string())
        }
        AlignmentMode::MeshCentroid => align_centroids(moving_mesh, fixed_mesh),
        AlignmentMode::IterativeClosestPoint => align_icp(moving_mesh, fixed_mesh, params),
    }
}

pub fn apply_transform_to_mesh(mesh: &Mesh, transform: &[[f64; 4]; 4]) -> Mesh {
    let matrix = array_to_matrix(transform);
    let mut output = mesh.clone();
    output.vertices = transform_points(&mesh.vertices, &matrix);
    output.calculate_normals();
    output
}

fn align_centroids(moving_mesh: &Mesh, fixed_mesh: &Mesh) -> Result<AlignmentResult, String> {
    let moving_center = centroid(&moving_mesh.vertices);
    let fixed_center = centroid(&fixed_mesh.vertices);
    let translation = fixed_center - moving_center;
    let transform = translation_matrix(translation);
    let moved = transform_points(&sample_points(&moving_mesh.vertices, 2_000), &transform);
    let fixed = sample_points(&fixed_mesh.vertices, 2_000);
    let (_, rms) = closest_correspondences(&moved, &fixed, None)?;

    Ok(AlignmentResult {
        matrix: matrix_to_array(&transform),
        rms_mm: rms,
        iterations: 1,
        converged: true,
        correspondence_count: moved.len(),
        warnings: vec!["centroid-only alignment: use ICP or landmarks for clinical precision".to_string()],
    })
}

fn align_icp(
    moving_mesh: &Mesh,
    fixed_mesh: &Mesh,
    params: &AlignmentParams,
) -> Result<AlignmentResult, String> {
    let sample_limit = params.sample_limit.clamp(3, 25_000);
    let fixed = sample_points(&fixed_mesh.vertices, sample_limit);
    let moving_seed = sample_points(&moving_mesh.vertices, sample_limit);
    let initial = translation_matrix(centroid(&fixed) - centroid(&moving_seed));
    let mut current = transform_points(&moving_seed, &initial);
    let mut total = initial;
    let mut previous_rms = f64::MAX;
    let mut final_rms = previous_rms;
    let mut final_count = 0usize;
    let max_iterations = params.max_iterations.clamp(1, 256);
    let tolerance = params.tolerance_mm.max(1.0e-9);
    let mut converged = false;

    for iteration in 0..max_iterations {
        let (pairs, rms) = closest_correspondences(
            &current,
            &fixed,
            params.max_correspondence_distance_mm,
        )?;
        if pairs.len() < 3 {
            return Err("ICP found fewer than 3 correspondences".to_string());
        }
        final_rms = rms;
        final_count = pairs.len();

        if (previous_rms - rms).abs() <= tolerance {
            converged = true;
            return Ok(AlignmentResult {
                matrix: matrix_to_array(&total),
                rms_mm: rms,
                iterations: iteration + 1,
                converged,
                correspondence_count: pairs.len(),
                warnings: icp_warnings(rms, pairs.len(), sample_limit),
            });
        }
        previous_rms = rms;

        let moving_pairs: Vec<Point3<f64>> = pairs.iter().map(|(source, _)| *source).collect();
        let fixed_pairs: Vec<Point3<f64>> = pairs.iter().map(|(_, target)| *target).collect();
        let step = estimate_rigid_transform(&moving_pairs, &fixed_pairs)?;
        current = transform_points(&current, &step);
        total = step * total;
    }

    Ok(AlignmentResult {
        matrix: matrix_to_array(&total),
        rms_mm: final_rms,
        iterations: max_iterations,
        converged,
        correspondence_count: final_count,
        warnings: icp_warnings(final_rms, final_count, sample_limit),
    })
}

fn estimate_rigid_transform(
    moving: &[Point3<f64>],
    fixed: &[Point3<f64>],
) -> Result<Matrix4<f64>, String> {
    if moving.len() != fixed.len() || moving.len() < 3 {
        return Err("rigid transform requires at least 3 matched point pairs".to_string());
    }

    let moving_center = centroid(moving);
    let fixed_center = centroid(fixed);
    let mut covariance = Matrix3::<f64>::zeros();

    for (m, f) in moving.iter().zip(fixed.iter()) {
        let mv = m.coords - moving_center;
        let fv = f.coords - fixed_center;
        covariance += mv * fv.transpose();
    }

    let svd = covariance.svd(true, true);
    let u = svd.u.ok_or_else(|| "SVD failed to produce U".to_string())?;
    let v_t = svd.v_t.ok_or_else(|| "SVD failed to produce Vt".to_string())?;
    let mut v = v_t.transpose();
    let mut rotation = v * u.transpose();

    if rotation.determinant() < 0.0 {
        v[(0, 2)] *= -1.0;
        v[(1, 2)] *= -1.0;
        v[(2, 2)] *= -1.0;
        rotation = v * u.transpose();
    }

    let translation = fixed_center - rotation * moving_center;
    let mut transform = Matrix4::<f64>::identity();
    for row in 0..3 {
        for col in 0..3 {
            transform[(row, col)] = rotation[(row, col)];
        }
        transform[(row, 3)] = translation[row];
    }
    Ok(transform)
}

fn closest_correspondences(
    moving: &[Point3<f64>],
    fixed: &[Point3<f64>],
    max_distance: Option<f64>,
) -> Result<(Vec<(Point3<f64>, Point3<f64>)>, f64), String> {
    if moving.is_empty() || fixed.is_empty() {
        return Err("correspondence search needs non-empty point sets".to_string());
    }

    let max_distance_sq = max_distance.map(|d| d * d);
    let mut pairs = Vec::with_capacity(moving.len());
    let mut sum = 0.0;

    for source in moving {
        let mut best = fixed[0];
        let mut best_distance_sq = (source - best).norm_squared();
        for candidate in fixed.iter().skip(1) {
            let distance_sq = (source - candidate).norm_squared();
            if distance_sq < best_distance_sq {
                best_distance_sq = distance_sq;
                best = *candidate;
            }
        }
        if max_distance_sq.is_some_and(|limit| best_distance_sq > limit) {
            continue;
        }
        pairs.push((*source, best));
        sum += best_distance_sq;
    }

    if pairs.is_empty() {
        return Err("all correspondences were rejected by max distance".to_string());
    }
    let pair_count = pairs.len();
    Ok((pairs, (sum / pair_count as f64).sqrt()))
}

fn rms_error(moving: &[Point3<f64>], fixed: &[Point3<f64>]) -> f64 {
    let sum: f64 = moving
        .iter()
        .zip(fixed.iter())
        .map(|(m, f)| (m - f).norm_squared())
        .sum();
    (sum / moving.len().max(1) as f64).sqrt()
}

fn centroid(points: &[Point3<f64>]) -> Vector3<f64> {
    if points.is_empty() {
        return Vector3::zeros();
    }
    points.iter().map(|point| point.coords).sum::<Vector3<f64>>() / points.len() as f64
}

fn points_from_arrays(points: &[[f64; 3]]) -> Vec<Point3<f64>> {
    points
        .iter()
        .map(|point| Point3::new(point[0], point[1], point[2]))
        .collect()
}

fn sample_points(points: &[Point3<f64>], limit: usize) -> Vec<Point3<f64>> {
    if points.len() <= limit {
        return points.to_vec();
    }
    let stride = (points.len() as f64 / limit as f64).ceil() as usize;
    points.iter().step_by(stride).take(limit).copied().collect()
}

fn transform_points(points: &[Point3<f64>], transform: &Matrix4<f64>) -> Vec<Point3<f64>> {
    points
        .iter()
        .map(|point| {
            let p = transform * point.to_homogeneous();
            Point3::new(p.x / p.w, p.y / p.w, p.z / p.w)
        })
        .collect()
}

fn translation_matrix(translation: Vector3<f64>) -> Matrix4<f64> {
    let mut transform = Matrix4::<f64>::identity();
    transform[(0, 3)] = translation.x;
    transform[(1, 3)] = translation.y;
    transform[(2, 3)] = translation.z;
    transform
}

fn matrix_to_array(matrix: &Matrix4<f64>) -> [[f64; 4]; 4] {
    let mut out = [[0.0; 4]; 4];
    for row in 0..4 {
        for col in 0..4 {
            out[row][col] = matrix[(row, col)];
        }
    }
    out
}

fn array_to_matrix(array: &[[f64; 4]; 4]) -> Matrix4<f64> {
    Matrix4::from_row_slice(&[
        array[0][0],
        array[0][1],
        array[0][2],
        array[0][3],
        array[1][0],
        array[1][1],
        array[1][2],
        array[1][3],
        array[2][0],
        array[2][1],
        array[2][2],
        array[2][3],
        array[3][0],
        array[3][1],
        array[3][2],
        array[3][3],
    ])
}

fn icp_warnings(rms: f64, correspondence_count: usize, sample_limit: usize) -> Vec<String> {
    let mut warnings = Vec::new();
    if rms > 0.25 {
        warnings.push(format!("RMS {:.3} mm exceeds precision target; inspect manually", rms));
    }
    if correspondence_count < sample_limit / 2 {
        warnings.push("low correspondence count; check scan overlap or max distance".to_string());
    }
    warnings
}

#[cfg(test)]
mod tests {
    use super::*;
    use tlanticad_mesh::Mesh;

    fn tetra_points() -> Vec<[f64; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ]
    }

    fn mesh_from_points(name: &str, points: &[[f64; 3]]) -> Mesh {
        let mut mesh = Mesh::new(name);
        mesh.vertices = points_from_arrays(points);
        mesh.indices = vec![[0, 1, 2], [0, 1, 3], [0, 2, 3], [1, 2, 3]];
        mesh.calculate_normals();
        mesh
    }

    #[test]
    fn landmarks_recover_translation() {
        let moving = tetra_points();
        let fixed: Vec<[f64; 3]> = moving
            .iter()
            .map(|point| [point[0] + 3.0, point[1] - 2.0, point[2] + 1.0])
            .collect();
        let result = align_landmarks(&moving, &fixed).unwrap();
        assert!(result.rms_mm < 1.0e-9);
        assert!((result.matrix[0][3] - 3.0).abs() < 1.0e-9);
        assert!((result.matrix[1][3] + 2.0).abs() < 1.0e-9);
        assert!((result.matrix[2][3] - 1.0).abs() < 1.0e-9);
    }

    #[test]
    fn landmarks_recover_rotation() {
        let moving = tetra_points();
        let fixed: Vec<[f64; 3]> = moving
            .iter()
            .map(|point| [-point[1] + 2.0, point[0] + 1.0, point[2] - 0.5])
            .collect();
        let result = align_landmarks(&moving, &fixed).unwrap();
        assert!(result.rms_mm < 1.0e-9);
        assert!(result.converged);
    }

    #[test]
    fn centroid_alignment_moves_mesh_near_target() {
        let moving = mesh_from_points("moving", &tetra_points());
        let fixed_points: Vec<[f64; 3]> = tetra_points()
            .iter()
            .map(|point| [point[0] + 10.0, point[1], point[2]])
            .collect();
        let fixed = mesh_from_points("fixed", &fixed_points);
        let result = align_meshes(
            &moving,
            &fixed,
            &AlignmentParams {
                mode: AlignmentMode::MeshCentroid,
                ..AlignmentParams::default()
            },
        )
        .unwrap();
        assert!((result.matrix[0][3] - 10.0).abs() < 1.0e-9);
    }

    #[test]
    fn icp_aligns_translated_mesh() {
        let moving_points: Vec<[f64; 3]> = tetra_points()
            .iter()
            .map(|point| [point[0] + 2.0, point[1], point[2]])
            .collect();
        let moving = mesh_from_points("moving", &moving_points);
        let fixed = mesh_from_points("fixed", &tetra_points());
        let result = align_meshes(
            &moving,
            &fixed,
            &AlignmentParams {
                sample_limit: 16,
                max_iterations: 16,
                tolerance_mm: 1.0e-8,
                ..AlignmentParams::default()
            },
        )
        .unwrap();
        assert!(result.rms_mm < 1.0e-6);
        assert!((result.matrix[0][3] + 2.0).abs() < 1.0e-6);
    }
}
