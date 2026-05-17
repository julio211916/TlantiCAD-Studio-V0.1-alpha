use crate::contour::Point;
use crate::error::{MimesisError, Result};

/// Triangulate a 2D polygon using the Earcut algorithm.
///
/// Returns a list of triangle indices (each triplet is one triangle).
pub fn triangulate_polygon(polygon: &[Point]) -> Result<Vec<usize>> {
    if polygon.len() < 3 {
        return Err(MimesisError::PolygonTooSmall(polygon.len()));
    }

    // Flatten to [x0, y0, x1, y1, ...]
    let coords: Vec<f64> = polygon.iter().flat_map(|&(x, y)| [x, y]).collect();

    // No holes
    let hole_indices: Vec<usize> = vec![];

    let triangles = earcutr::earcut(&coords, &hole_indices, 2)
        .map_err(|e| MimesisError::Triangulation(format!("{:?}", e)))?;

    if triangles.is_empty() {
        return Err(MimesisError::Triangulation(
            "Earcut returned zero triangles".into(),
        ));
    }

    Ok(triangles)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triangle() {
        let polygon = vec![(0.0, 0.0), (10.0, 0.0), (5.0, 10.0)];
        let indices = triangulate_polygon(&polygon).unwrap();
        assert_eq!(indices.len(), 3); // 1 triangle = 3 indices
    }

    #[test]
    fn test_square() {
        let polygon = vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)];
        let indices = triangulate_polygon(&polygon).unwrap();
        assert_eq!(indices.len(), 6); // 2 triangles = 6 indices
    }
}
