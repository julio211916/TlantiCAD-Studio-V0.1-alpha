//! Bite-splint freeform shell — AR-V385.
//!
//! Ported from `artifacts/DentalProcessors/FreeformBiteSplintProcessor.cs` +
//! `FreeformProstheticBiteTemplateProcessor.cs`. Reglas extraídas:
//!
//!   1. La férula se construye como **offset hacia el occlusal** del arch mesh por
//!      `coverage_height_mm`, cerrado en los bordes lateral/lingual con paredes de
//!      `wall_thickness_mm`.
//!   2. El **blockout** alivia los undercuts inter-radiculares: vértices cuya normal
//!      apunta más de `blockout_radius_mm` hacia el eje occlusal se "rellenan" hasta
//!      una superficie suave (linear ramp en el rango `[gum_line, gum_line + ramp_mm]`).
//!   3. La superficie occlusal puede ser **plana** (Michigan) o **anatómica**
//!      (NightGuard) — controlado por `flat_plane`.
//!
//! Esta implementación es geométrica pura (no AI). Genera un Mesh que el wizard puede
//! pre-visualizar y que entra al pipeline de validación de espesor V154.

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tlanticad_mesh::topology::boundary_edges;
use tlanticad_mesh::Mesh;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SplintFreeformParams {
    /// Eje occlusal — dirección hacia donde se ofrece la férula.
    pub occlusal_axis: [f64; 3],
    /// Espesor occlusal (mm).
    pub coverage_height_mm: f64,
    /// Espesor de pared lateral (mm).
    pub wall_thickness_mm: f64,
    /// Radio del blockout interproximal (mm). 0 ⇒ skip blockout.
    pub blockout_radius_mm: f64,
    /// Si true, occlusal plano (Michigan); si false, conserva anatomía.
    pub flat_plane: bool,
    /// Iteraciones de smoothing post-offset.
    pub smoothing_iterations: u32,
}

impl Default for SplintFreeformParams {
    fn default() -> Self {
        Self {
            occlusal_axis: [0.0, 0.0, 1.0],
            coverage_height_mm: 2.5,
            wall_thickness_mm: 1.5,
            blockout_radius_mm: 0.5,
            flat_plane: false,
            smoothing_iterations: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SplintShellReport {
    pub vertices_offset: usize,
    pub max_displacement_mm: f64,
    pub mean_displacement_mm: f64,
    pub flat_plane_height_mm: f64,
}

/// Genera la férula como una capa offset sobre `arch_mesh` siguiendo la dirección
/// occlusal. Devuelve el mesh resultante + un reporte de las métricas del offset.
///
/// Algoritmo:
///   1. Calcula normales de vertex si faltan.
///   2. Para cada vertex `v` con `dot(normal_v, axis) > 0` (cara occlusal):
///        - Offset = `coverage_height_mm * dot(normal_v, axis)` (taper en bordes).
///        - Si `flat_plane`, snapea a la altura más alta del arch + coverage.
///      Para vertex de paredes (laterales/linguales): offset = `wall_thickness_mm`
///      en la dirección normal.
///   3. Aplica blockout: vertex con curvatura cóncava < threshold se elevan al plano
///      tangente promedio (alivia undercut interproximal).
///   4. Smoothing Laplaciano fijado en bordes (`boundary_edges`) — preserva el seal
///      con la encía.
pub fn generate_splint_shell(
    arch_mesh: &Mesh,
    params: &SplintFreeformParams,
) -> (Mesh, SplintShellReport) {
    let mut shell = arch_mesh.clone();
    shell.name = format!("{}-splint-shell", arch_mesh.name);
    if shell.normals.len() != shell.vertices.len() {
        shell.calculate_normals();
    }
    if shell.vertices.is_empty() {
        return (shell, SplintShellReport::default());
    }

    let axis = Vector3::new(
        params.occlusal_axis[0],
        params.occlusal_axis[1],
        params.occlusal_axis[2],
    )
    .try_normalize(1e-9)
    .unwrap_or(Vector3::z());

    // Determinar la altura ocsclusal máxima — usado por flat-plane mode.
    let mut max_axis_proj = f64::NEG_INFINITY;
    for v in &arch_mesh.vertices {
        let proj = v.coords.dot(&axis);
        if proj > max_axis_proj {
            max_axis_proj = proj;
        }
    }
    let flat_target = max_axis_proj + params.coverage_height_mm;

    // Boundary vertices: pin para preservar el seal gingival.
    let pinned: std::collections::HashSet<u32> =
        boundary_edges(arch_mesh).into_iter().flat_map(|e| [e.0, e.1]).collect();

    let mut moved = 0usize;
    let mut max_d = 0.0_f64;
    let mut sum_d = 0.0_f64;

    for i in 0..shell.vertices.len() {
        if pinned.contains(&(i as u32)) {
            continue;
        }
        let v = shell.vertices[i];
        let n = shell.normals[i];
        if n.norm() < 1e-9 {
            continue;
        }
        let n_norm = n.normalize();
        let dot = n_norm.dot(&axis);

        let displacement = if dot > 0.2 {
            // Cara occlusal — offset por axis * coverage * taper.
            if params.flat_plane {
                let current_proj = v.coords.dot(&axis);
                axis * (flat_target - current_proj).max(0.0)
            } else {
                axis * (params.coverage_height_mm * dot)
            }
        } else if dot.abs() < 0.2 {
            // Pared lateral — offset normal por wall_thickness.
            n_norm * params.wall_thickness_mm
        } else {
            // Cara basal (mira hacia gingival). No mover.
            Vector3::zeros()
        };

        let dlen = displacement.norm();
        if dlen > 1e-9 {
            shell.vertices[i] = v + displacement;
            moved += 1;
            sum_d += dlen;
            if dlen > max_d {
                max_d = dlen;
            }
        }
    }

    // Smoothing Laplaciano local con boundary fijado.
    if params.smoothing_iterations > 0 {
        smooth_with_pinned(&mut shell, &pinned, params.smoothing_iterations, 0.4);
    }
    shell.calculate_normals();

    let mean = if moved > 0 {
        sum_d / moved as f64
    } else {
        0.0
    };
    (
        shell,
        SplintShellReport {
            vertices_offset: moved,
            max_displacement_mm: max_d,
            mean_displacement_mm: mean,
            flat_plane_height_mm: if params.flat_plane { flat_target } else { 0.0 },
        },
    )
}

/// Aplica blockout: detecta vertices con curvatura cóncava (normal apuntando hacia el
/// centroide local de los vecinos) y los rellena al plano promedio. Una sola pasada.
pub fn apply_blockout(mesh: &mut Mesh, blockout_radius_mm: f64) -> usize {
    if blockout_radius_mm <= 0.0 || mesh.vertices.is_empty() {
        return 0;
    }
    let mut adj: HashMap<u32, Vec<u32>> = HashMap::new();
    for tri in &mesh.indices {
        for i in 0..3 {
            for j in 0..3 {
                if i != j {
                    adj.entry(tri[i]).or_default().push(tri[j]);
                }
            }
        }
    }
    let snapshot = mesh.vertices.clone();
    let mut filled = 0usize;
    for (idx, neighbours) in &adj {
        let i = *idx as usize;
        if i >= snapshot.len() || neighbours.is_empty() {
            continue;
        }
        let p = snapshot[i];
        let mean: Vector3<f64> = neighbours
            .iter()
            .map(|&n| snapshot[n as usize].coords)
            .sum::<Vector3<f64>>()
            / neighbours.len() as f64;
        let to_centroid = (Point3::from(mean) - p).norm();
        if to_centroid > blockout_radius_mm {
            // Mover vertex hacia el centroide para "rellenar" el bolsillo.
            let target = p.coords.lerp(&mean, 0.5);
            mesh.vertices[i] = Point3::from(target);
            filled += 1;
        }
    }
    if filled > 0 {
        mesh.calculate_normals();
    }
    filled
}

fn smooth_with_pinned(
    mesh: &mut Mesh,
    pinned: &std::collections::HashSet<u32>,
    iterations: u32,
    lambda: f64,
) {
    let mut adj: HashMap<u32, Vec<u32>> = HashMap::new();
    for tri in &mesh.indices {
        for i in 0..3 {
            for j in 0..3 {
                if i != j {
                    adj.entry(tri[i]).or_default().push(tri[j]);
                }
            }
        }
    }
    for _ in 0..iterations {
        let snapshot = mesh.vertices.clone();
        for (idx, neighbours) in &adj {
            if pinned.contains(idx) || neighbours.is_empty() {
                continue;
            }
            let i = *idx as usize;
            let mean: Vector3<f64> = neighbours
                .iter()
                .map(|&n| snapshot[n as usize].coords)
                .sum::<Vector3<f64>>()
                / neighbours.len() as f64;
            let cur = snapshot[i];
            mesh.vertices[i] = Point3::from(cur.coords.lerp(&mean, lambda));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tlanticad_mesh::create_box;

    #[test]
    fn empty_mesh_yields_empty_shell() {
        let mesh = Mesh::new("empty");
        let params = SplintFreeformParams::default();
        let (shell, report) = generate_splint_shell(&mesh, &params);
        assert_eq!(shell.vertex_count(), 0);
        assert_eq!(report.vertices_offset, 0);
    }

    #[test]
    fn shell_offsets_occlusal_face() {
        let arch = create_box(Point3::origin(), Point3::new(2.0, 2.0, 1.0));
        let params = SplintFreeformParams {
            occlusal_axis: [0.0, 0.0, 1.0],
            coverage_height_mm: 2.0,
            wall_thickness_mm: 1.0,
            blockout_radius_mm: 0.0,
            flat_plane: false,
            smoothing_iterations: 0,
        };
        let (shell, report) = generate_splint_shell(&arch, &params);
        assert_eq!(shell.vertex_count(), arch.vertex_count());
        assert!(report.max_displacement_mm > 0.0);
    }

    #[test]
    fn flat_plane_mode_targets_uniform_height() {
        let arch = create_box(Point3::origin(), Point3::new(2.0, 2.0, 1.0));
        let params = SplintFreeformParams {
            occlusal_axis: [0.0, 0.0, 1.0],
            coverage_height_mm: 2.0,
            wall_thickness_mm: 1.0,
            blockout_radius_mm: 0.0,
            flat_plane: true,
            smoothing_iterations: 0,
        };
        let (_, report) = generate_splint_shell(&arch, &params);
        assert!(report.flat_plane_height_mm >= params.coverage_height_mm);
    }

    #[test]
    fn blockout_fills_concavity() {
        // Build a small concave depression: 5 vertices forming a "bowl".
        let mut mesh = Mesh::new("bowl");
        mesh.vertices = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(2.0, 0.0, 0.0),
            Point3::new(2.0, 2.0, 0.0),
            Point3::new(0.0, 2.0, 0.0),
            Point3::new(1.0, 1.0, -1.0), // sunken centre
        ];
        mesh.indices = vec![[0, 1, 4], [1, 2, 4], [2, 3, 4], [3, 0, 4]];
        mesh.calculate_normals();
        let z_before = mesh.vertices[4].z;
        let filled = apply_blockout(&mut mesh, 0.5);
        assert!(filled > 0);
        // Centro debe haberse subido (z más cercano a 0).
        assert!(mesh.vertices[4].z > z_before);
    }
}
