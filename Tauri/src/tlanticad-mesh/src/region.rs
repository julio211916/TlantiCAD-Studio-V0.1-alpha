//! Region selection primitives — BFS over face / vertex adjacency from a seed.
//!
//! Ported from `DentalProcessors/AddRemoveMeshProcessor` selection logic.
//! Used by margin detection, freeform brush, and adapt-to-gingiva.

use crate::Mesh;
use nalgebra::Point3;
use std::collections::{HashMap, VecDeque};

/// A face-based selection — contiguous set of triangle indices.
#[derive(Debug, Clone, Default)]
pub struct FaceRegion {
    pub faces: Vec<usize>,
}

impl FaceRegion {
    pub fn is_empty(&self) -> bool {
        self.faces.is_empty()
    }

    pub fn count(&self) -> usize {
        self.faces.len()
    }
}

fn build_face_adjacency(mesh: &Mesh) -> HashMap<usize, Vec<usize>> {
    let mut edge_to_faces: HashMap<(u32, u32), Vec<usize>> = HashMap::new();
    for (fi, tri) in mesh.indices.iter().enumerate() {
        for i in 0..3 {
            let a = tri[i];
            let b = tri[(i + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_to_faces.entry(key).or_default().push(fi);
        }
    }
    let mut adj: HashMap<usize, Vec<usize>> = HashMap::new();
    for faces in edge_to_faces.values() {
        if faces.len() == 2 {
            adj.entry(faces[0]).or_default().push(faces[1]);
            adj.entry(faces[1]).or_default().push(faces[0]);
        }
    }
    adj
}

/// Find the closest face to a point — used as seed for region growing.
pub fn closest_face(mesh: &Mesh, point: &Point3<f64>) -> Option<usize> {
    let mut best: Option<(usize, f64)> = None;
    for (fi, tri) in mesh.indices.iter().enumerate() {
        let centroid = (mesh.vertices[tri[0] as usize].coords
            + mesh.vertices[tri[1] as usize].coords
            + mesh.vertices[tri[2] as usize].coords)
            / 3.0;
        let d = (centroid - point.coords).norm_squared();
        match best {
            None => best = Some((fi, d)),
            Some((_, bd)) if d < bd => best = Some((fi, d)),
            _ => {}
        }
    }
    best.map(|(fi, _)| fi)
}

/// Region grow by geodesic radius (sum of face-centroid distances) starting from seed face.
///
/// Stops expanding when the cumulative distance would exceed `radius_mm`. This mirrors
/// exocad's "circular" brush selection.
pub fn grow_by_radius(mesh: &Mesh, seed_face: usize, radius_mm: f64) -> FaceRegion {
    if seed_face >= mesh.indices.len() {
        return FaceRegion::default();
    }
    let r2 = radius_mm * radius_mm;
    let adj = build_face_adjacency(mesh);

    let mut visited: Vec<bool> = vec![false; mesh.indices.len()];
    let mut faces = Vec::new();
    let mut queue: VecDeque<usize> = VecDeque::new();

    let seed_centroid = face_centroid(mesh, seed_face);
    visited[seed_face] = true;
    queue.push_back(seed_face);
    faces.push(seed_face);

    while let Some(fi) = queue.pop_front() {
        if let Some(neighbors) = adj.get(&fi) {
            for &nf in neighbors {
                if visited[nf] {
                    continue;
                }
                let c = face_centroid(mesh, nf);
                if (c - seed_centroid.coords).coords.norm_squared() > r2 {
                    continue;
                }
                visited[nf] = true;
                faces.push(nf);
                queue.push_back(nf);
            }
        }
    }

    FaceRegion { faces }
}

/// Region grow by feature label: include face if predicate is true. Stops at boundary.
pub fn grow_with<F>(mesh: &Mesh, seed_face: usize, predicate: F) -> FaceRegion
where
    F: Fn(usize) -> bool,
{
    if seed_face >= mesh.indices.len() || !predicate(seed_face) {
        return FaceRegion::default();
    }
    let adj = build_face_adjacency(mesh);
    let mut visited: Vec<bool> = vec![false; mesh.indices.len()];
    let mut faces = Vec::new();
    let mut queue: VecDeque<usize> = VecDeque::new();
    visited[seed_face] = true;
    queue.push_back(seed_face);
    faces.push(seed_face);
    while let Some(fi) = queue.pop_front() {
        if let Some(neighbors) = adj.get(&fi) {
            for &nf in neighbors {
                if visited[nf] {
                    continue;
                }
                if !predicate(nf) {
                    continue;
                }
                visited[nf] = true;
                faces.push(nf);
                queue.push_back(nf);
            }
        }
    }
    FaceRegion { faces }
}

/// Vertices belonging to a face region (deduped).
pub fn region_vertices(mesh: &Mesh, region: &FaceRegion) -> Vec<u32> {
    let mut seen: HashMap<u32, ()> = HashMap::new();
    for &fi in &region.faces {
        if fi >= mesh.indices.len() {
            continue;
        }
        let tri = mesh.indices[fi];
        seen.insert(tri[0], ());
        seen.insert(tri[1], ());
        seen.insert(tri[2], ());
    }
    let mut out: Vec<u32> = seen.into_keys().collect();
    out.sort_unstable();
    out
}

fn face_centroid(mesh: &Mesh, fi: usize) -> Point3<f64> {
    let tri = mesh.indices[fi];
    let v0 = mesh.vertices[tri[0] as usize];
    let v1 = mesh.vertices[tri[1] as usize];
    let v2 = mesh.vertices[tri[2] as usize];
    Point3::from((v0.coords + v1.coords + v2.coords) / 3.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_box;

    #[test]
    fn grow_by_radius_covers_neighbors() {
        let mesh = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let seed = closest_face(&mesh, &Point3::new(0.5, 0.5, 0.0)).unwrap();
        let region = grow_by_radius(&mesh, seed, 2.0);
        assert!(region.count() >= 2, "small radius should still grab adj face");
    }

    #[test]
    fn grow_by_radius_zero_returns_seed() {
        let mesh = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let seed = closest_face(&mesh, &Point3::new(0.5, 0.5, 0.0)).unwrap();
        let region = grow_by_radius(&mesh, seed, 0.0);
        assert_eq!(region.count(), 1);
    }

    #[test]
    fn region_vertices_dedupes() {
        let mesh = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let region = FaceRegion {
            faces: (0..mesh.indices.len()).collect(),
        };
        let verts = region_vertices(&mesh, &region);
        assert_eq!(verts.len(), 8, "cube has 8 unique vertices");
    }

    #[test]
    fn closest_face_picks_nearest() {
        let mesh = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let f = closest_face(&mesh, &Point3::new(-10.0, 0.5, 0.5)).unwrap();
        let tri = mesh.indices[f];
        let cx = (mesh.vertices[tri[0] as usize].x
            + mesh.vertices[tri[1] as usize].x
            + mesh.vertices[tri[2] as usize].x)
            / 3.0;
        assert!(cx < 0.5, "nearest face should be on -x side");
    }
}
