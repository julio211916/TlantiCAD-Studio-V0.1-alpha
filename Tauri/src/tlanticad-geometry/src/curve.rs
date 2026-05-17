//! Curve operations: evaluate, sample, length, tangent, project

use nalgebra::{Point3, Vector3};
use crate::Curve;

impl Curve {
    /// Evaluate a point on the curve at parameter t ∈ [0, 1]
    pub fn evaluate(&self, t: f64) -> Point3<f64> {
        let t = t.clamp(0.0, 1.0);
        match self {
            Curve::Line { start, end } => {
                Point3::from(start.coords.lerp(&end.coords, t))
            }
            Curve::Circle { center, radius, normal } => {
                let angle = t * std::f64::consts::TAU;
                let (u, v) = orthonormal_basis(normal);
                center + u * (*radius * angle.cos()) + v * (*radius * angle.sin())
            }
            Curve::Ellipse { center, major, minor } => {
                let angle = t * std::f64::consts::TAU;
                Point3::new(
                    center.x + major * angle.cos(),
                    center.y + minor * angle.sin(),
                    center.z,
                )
            }
            Curve::Bezier { control_points } => de_casteljau(control_points, t),
            Curve::Nurbs { knots, points, weights } => {
                nurbs_evaluate(knots, points, weights, t)
            }
        }
    }

    /// Sample N evenly-spaced points along the curve
    pub fn sample(&self, n: usize) -> Vec<Point3<f64>> {
        (0..=n).map(|i| self.evaluate(i as f64 / n as f64)).collect()
    }

    /// Approximate curve length by sampling
    pub fn length(&self, segments: usize) -> f64 {
        let pts = self.sample(segments.max(2));
        pts.windows(2).map(|w| (w[1] - w[0]).norm()).sum()
    }

    /// Tangent vector at parameter t (finite difference)
    pub fn tangent(&self, t: f64) -> Vector3<f64> {
        let dt = 1e-6;
        let p0 = self.evaluate((t - dt).max(0.0));
        let p1 = self.evaluate((t + dt).min(1.0));
        (p1 - p0).normalize()
    }

    /// Project a point onto the curve (closest parameter)
    pub fn project(&self, point: &Point3<f64>, samples: usize) -> (f64, Point3<f64>) {
        let mut best_t = 0.0;
        let mut best_dist = f64::MAX;
        let mut best_pt = self.evaluate(0.0);
        for i in 0..=samples {
            let t = i as f64 / samples as f64;
            let p = self.evaluate(t);
            let d = (p - point).norm();
            if d < best_dist {
                best_dist = d;
                best_t = t;
                best_pt = p;
            }
        }
        (best_t, best_pt)
    }
}

/// De Casteljau Bézier evaluation
fn de_casteljau(points: &[Point3<f64>], t: f64) -> Point3<f64> {
    if points.len() == 1 {
        return points[0];
    }
    let mut tmp: Vec<Point3<f64>> = points.to_vec();
    let n = tmp.len();
    for r in 1..n {
        for i in 0..n - r {
            tmp[i] = Point3::from(tmp[i].coords.lerp(&tmp[i + 1].coords, t));
        }
    }
    tmp[0]
}

/// NURBS curve evaluation (rational B-spline)
fn nurbs_evaluate(knots: &[f64], points: &[Point3<f64>], weights: &[f64], t: f64) -> Point3<f64> {
    let n = points.len();
    let degree = knots.len().saturating_sub(n + 1);
    if degree == 0 || n == 0 {
        return points.first().copied().unwrap_or(Point3::origin());
    }
    let t_clamped = t.clamp(
        knots.first().copied().unwrap_or(0.0),
        knots.last().copied().unwrap_or(1.0) - 1e-10,
    );
    let mut sum = Vector3::zeros();
    let mut w_sum = 0.0;
    for i in 0..n {
        let b = bspline_basis(i, degree, t_clamped, knots);
        let w = weights.get(i).copied().unwrap_or(1.0);
        sum += points[i].coords * b * w;
        w_sum += b * w;
    }
    if w_sum.abs() < 1e-12 {
        points[0]
    } else {
        Point3::from(sum / w_sum)
    }
}

fn bspline_basis(i: usize, degree: usize, t: f64, knots: &[f64]) -> f64 {
    if degree == 0 {
        return if knots.get(i).copied().unwrap_or(0.0) <= t
            && t < knots.get(i + 1).copied().unwrap_or(0.0)
        {
            1.0
        } else {
            0.0
        };
    }
    let left = {
        let denom = knots.get(i + degree).copied().unwrap_or(0.0)
            - knots.get(i).copied().unwrap_or(0.0);
        if denom.abs() < 1e-12 {
            0.0
        } else {
            (t - knots.get(i).copied().unwrap_or(0.0)) / denom
                * bspline_basis(i, degree - 1, t, knots)
        }
    };
    let right = {
        let denom = knots.get(i + degree + 1).copied().unwrap_or(0.0)
            - knots.get(i + 1).copied().unwrap_or(0.0);
        if denom.abs() < 1e-12 {
            0.0
        } else {
            (knots.get(i + degree + 1).copied().unwrap_or(0.0) - t) / denom
                * bspline_basis(i + 1, degree - 1, t, knots)
        }
    };
    left + right
}

/// Build orthonormal basis from a normal vector
fn orthonormal_basis(normal: &Vector3<f64>) -> (Vector3<f64>, Vector3<f64>) {
    let n = normal.normalize();
    let up = if n.x.abs() < 0.9 {
        Vector3::x()
    } else {
        Vector3::y()
    };
    let u = n.cross(&up).normalize();
    let v = n.cross(&u).normalize();
    (u, v)
}
