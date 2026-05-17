use crate::contour::Point;

/// Ramer-Douglas-Peucker polygon simplification.
///
/// Reduces the number of vertices in a polygon by removing points
/// within `tolerance` distance of the simplified line.
pub fn simplify_polygon(points: &[Point], tolerance: f64) -> Vec<Point> {
    if points.len() < 3 {
        return points.to_vec();
    }

    let result = rdp(points, tolerance);

    // Ensure we still have a valid polygon
    if result.len() < 3 {
        return points.to_vec();
    }

    result
}

fn rdp(points: &[Point], epsilon: f64) -> Vec<Point> {
    let n = points.len();
    if n < 3 {
        return points.to_vec();
    }

    // Find the point with maximum distance from the line (first → last)
    let mut max_dist = 0.0;
    let mut max_idx = 0;

    for i in 1..n - 1 {
        let d = perpendicular_distance(points[i], points[0], points[n - 1]);
        if d > max_dist {
            max_dist = d;
            max_idx = i;
        }
    }

    if max_dist > epsilon {
        // Recurse on both halves
        let mut left = rdp(&points[..=max_idx], epsilon);
        let right = rdp(&points[max_idx..], epsilon);
        left.pop(); // Remove duplicate at split point
        left.extend(right);
        left
    } else {
        // Only keep endpoints
        vec![points[0], points[n - 1]]
    }
}

/// Perpendicular distance from point `p` to line segment `a`–`b`.
fn perpendicular_distance(p: Point, a: Point, b: Point) -> f64 {
    let dx = b.0 - a.0;
    let dy = b.1 - a.1;
    let len_sq = dx * dx + dy * dy;

    if len_sq < 1e-12 {
        // a and b are the same point
        let ex = p.0 - a.0;
        let ey = p.1 - a.1;
        return (ex * ex + ey * ey).sqrt();
    }

    let numerator = ((b.0 - a.0) * (a.1 - p.1) - (a.0 - p.0) * (b.1 - a.1)).abs();
    numerator / len_sq.sqrt()
}

/// Chaikin corner-cutting smoothing.
///
/// Each iteration replaces each pair of adjacent points with two new points
/// at 25% and 75% along the segment, producing a smoother curve.
pub fn smooth_polygon(points: &[Point], iterations: u32) -> Vec<Point> {
    let mut current = points.to_vec();

    for _ in 0..iterations {
        if current.len() < 3 {
            break;
        }

        let n = current.len();
        let mut smoothed = Vec::with_capacity(n * 2);

        for i in 0..n {
            let j = (i + 1) % n;
            let (ax, ay) = current[i];
            let (bx, by) = current[j];

            // 25% point
            smoothed.push((0.75 * ax + 0.25 * bx, 0.75 * ay + 0.25 * by));
            // 75% point
            smoothed.push((0.25 * ax + 0.75 * bx, 0.25 * ay + 0.75 * by));
        }

        current = smoothed;
    }

    current
}
