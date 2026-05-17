use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use cad_core::{CadError, HeMesh, Point3f, Result};

/// Minimal PLY reader — supports ASCII PLY with `x y z` vertex properties
/// and triangular faces. Sufficient for lab scanner output.
pub fn read(path: &Path) -> Result<HeMesh> {
    let reader = BufReader::new(File::open(path).map_err(CadError::Io)?);
    let mut lines = reader.lines();

    // --- Parse header ---
    let mut vertex_count: usize = 0;
    let mut face_count: usize = 0;
    let mut in_vertex_element = false;
    let mut x_idx = 0usize;
    let mut y_idx = 1usize;
    let mut z_idx = 2usize;
    let mut prop_idx = 0usize;
    let mut is_ascii = true;

    loop {
        let line = lines
            .next()
            .ok_or_else(|| CadError::Mesh("PLY: unexpected end of header".into()))?
            .map_err(CadError::Io)?;
        let tokens: Vec<&str> = line.trim().split_whitespace().collect();

        match tokens.as_slice() {
            ["format", "ascii", ..] => is_ascii = true,
            ["format", "binary_little_endian", ..] | ["format", "binary_big_endian", ..] => {
                return Err(CadError::Mesh(
                    "PLY: binary format not yet supported".into(),
                ))
            }
            ["element", "vertex", n] => {
                vertex_count = n.parse().unwrap_or(0);
                in_vertex_element = true;
                prop_idx = 0;
            }
            ["element", ..] => {
                in_vertex_element = false;
            }
            ["property", _, name] if in_vertex_element => {
                match *name {
                    "x" => x_idx = prop_idx,
                    "y" => y_idx = prop_idx,
                    "z" => z_idx = prop_idx,
                    _ => {}
                }
                prop_idx += 1;
            }
            ["element", "face", n] => face_count = n.parse().unwrap_or(0),
            ["end_header"] => break,
            _ => {}
        }
    }

    if !is_ascii {
        return Err(CadError::Mesh("PLY: only ASCII supported".into()));
    }

    // --- Vertices ---
    let mut positions: Vec<Point3f> = Vec::with_capacity(vertex_count);
    for _ in 0..vertex_count {
        let line = lines
            .next()
            .ok_or_else(|| CadError::Mesh("PLY: premature end of vertex data".into()))?
            .map_err(CadError::Io)?;
        let parts: Vec<f32> = line
            .trim()
            .split_whitespace()
            .filter_map(|s| s.parse().ok())
            .collect();
        let x = parts.get(x_idx).copied().unwrap_or(0.0);
        let y = parts.get(y_idx).copied().unwrap_or(0.0);
        let z = parts.get(z_idx).copied().unwrap_or(0.0);
        positions.push(Point3f::new(x, y, z));
    }

    // --- Faces ---
    let mut indices: Vec<[u32; 3]> = Vec::with_capacity(face_count * 1);
    for _ in 0..face_count {
        let line = lines
            .next()
            .ok_or_else(|| CadError::Mesh("PLY: premature end of face data".into()))?
            .map_err(CadError::Io)?;
        let parts: Vec<u32> = line
            .trim()
            .split_whitespace()
            .filter_map(|s| s.parse().ok())
            .collect();
        if parts.is_empty() {
            continue;
        }
        let n = parts[0] as usize;
        if parts.len() < n + 1 {
            continue;
        }
        let vi = &parts[1..=n];
        // Fan triangulation
        for i in 1..(vi.len() - 1) {
            indices.push([vi[0], vi[i], vi[i + 1]]);
        }
    }

    Ok(HeMesh::from_triangles(&positions, &indices))
}
