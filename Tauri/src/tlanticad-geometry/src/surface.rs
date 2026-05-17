//! Surface operations: evaluate, normal, sample grid, project

use nalgebra::{Point3, Vector3};
use crate::Surface;

impl Surface {
    /// Evaluate a point on the surface at parameters (u, v) ∈ [0, 1]²
    pub fn evaluate(&self, u: f64, v: f64) -> Point3<f64> {
        match self {
            Surface::Plane { origin, normal } => {
                let (bu, bv) = orthonormal_basis(normal);
                origin + bu * (u - 0.5) + bv * (v - 0.5)
            }
            Surface::Sphere { center, radius } => {
                let theta = u * std::f64::consts::TAU;
                let phi = v * std::f64::consts::PI;
                Point3::new(
                    center.x + radius * phi.sin() * theta.cos(),
                    center.y + radius * phi.sin() * theta.sin(),
                    center.z + radius * phi.cos(),
                )
            }
            Surface::Cylinder { origin, axis, radius } => {
                let theta = u * std::f64::consts::TAU;
                let (bu, bv) = orthonormal_basis(axis);
                let axn = axis.normalize();
                origin + bu * (*radius * theta.cos()) + bv * (*radius * theta.sin()) + axn * v
            }
            Surface::NurbsSurface { u_knots: _, v_knots: _, points } => {
                // Bilinear interpolation over control point grid
                if points.is_empty() || points[0].is_empty() {
                    return Point3::origin();
                }
                let rows = points.len();
                let cols = points[0].len();
                let ui = (u * (rows - 1) as f64).min((rows - 1) as f64);
                let vi = (v * (cols - 1) as f64).min((cols - 1) as f64);
                let r0 = ui.floor() as usize;
                let c0 = vi.floor() as usize;
                let r1 = (r0 + 1).min(rows - 1);
                let c1 = (c0 + 1).min(cols - 1);
                let fu = ui.fract();
                let fv = vi.fract();
                let p00 = &points[r0][c0];
                let p10 = &points[r1][c0];
                let p01 = &points[r0][c1];
                let p11 = &points[r1][c1];
                let top = p00.coords.lerp(&p01.coords, fv);
                let bot = p10.coords.lerp(&p11.coords, fv);
                Point3::from(top.lerp(&bot, fu))
            }
        }
    }

    /// Surface normal at (u, v) via finite differences
    pub fn normal_at(&self, u: f64, v: f64) -> Vector3<f64> {
        let dt = 1e-5;
        let du = self.evaluate(u + dt, v) - self.evaluate(u - dt, v);
        let dv = self.evaluate(u, v + dt) - self.evaluate(u, v - dt);
        du.cross(&dv).normalize()
    }

    /// Sample a grid of points on the surface
    pub fn sample_grid(&self, nu: usize, nv: usize) -> Vec<Vec<Point3<f64>>> {
        (0..=nu)
            .map(|i| {
                let u = i as f64 / nu as f64;
                (0..=nv)
                    .map(|j| self.evaluate(u, j as f64 / nv as f64))
                    .collect()
            })
            .collect()
    }

    /// Project a point onto the surface (brute-force closest parameter)
    pub fn project(&self, point: &Point3<f64>, samples: usize) -> (f64, f64, Point3<f64>) {
        let mut best = (0.0, 0.0, self.evaluate(0.0, 0.0));
        let mut best_dist = f64::MAX;
        for i in 0..=samples {
            let u = i as f64 / samples as f64;
            for j in 0..=samples {
                let v = j as f64 / samples as f64;
                let p = self.evaluate(u, v);
                let d = (p - point).norm();
                if d < best_dist {
                    best_dist = d;
                    best = (u, v, p);
                }
            }
        }
        best
    }
}

fn orthonormal_basis(normal: &Vector3<f64>) -> (Vector3<f64>, Vector3<f64>) {
    let n = normal.normalize();
    let up = if n.x.abs() < 0.9 { Vector3::x() } else { Vector3::y() };
    let u = n.cross(&up).normalize();
    let v = n.cross(&u).normalize();
    (u, v)
}
