use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::Path,
};

use cad_core::{CadError, HeMesh, Point3f, Result};

// ─── Read ─────────────────────────────────────────────────────────────────

pub fn read(path: &Path) -> Result<HeMesh> {
    let reader = BufReader::new(File::open(path).map_err(CadError::Io)?);

    let mut positions: Vec<Point3f> = Vec::new();
    let mut indices: Vec<[u32; 3]> = Vec::new();

    for line in reader.lines() {
        let line = line.map_err(CadError::Io)?;
        let t = line.trim();

        if t.starts_with("v ") {
            let parts: Vec<&str> = t.split_whitespace().collect();
            if parts.len() >= 4 {
                let x: f32 = parts[1].parse().unwrap_or(0.0);
                let y: f32 = parts[2].parse().unwrap_or(0.0);
                let z: f32 = parts[3].parse().unwrap_or(0.0);
                positions.push(Point3f::new(x, y, z));
            }
        } else if t.starts_with("f ") {
            // Face: "f 1 2 3" or "f 1/1/1 2/2/2 3/3/3"
            let parts: Vec<&str> = t.split_whitespace().skip(1).collect();
            let vi: Vec<u32> = parts
                .iter()
                .filter_map(|p| {
                    p.split('/').next()?.parse::<u32>().ok().map(|i| i - 1) // OBJ 1-indexed
                })
                .collect();

            // Fan triangulation for polygonal faces
            if vi.len() >= 3 {
                for i in 1..(vi.len() - 1) {
                    indices.push([vi[0], vi[i], vi[i + 1]]);
                }
            }
        }
    }

    Ok(HeMesh::from_triangles(&positions, &indices))
}

// ─── Write ────────────────────────────────────────────────────────────────

pub fn write(mesh: &HeMesh, path: &Path) -> Result<()> {
    let vb = mesh.to_vertex_buffer(); // [x,y,z,nx,ny,nz] per vertex
    let ib = mesh.to_index_buffer();
    let vert_count = vb.len() / 6;

    let mut out = BufWriter::new(File::create(path).map_err(CadError::Io)?);
    writeln!(out, "# TlantiCAD OBJ export").map_err(CadError::Io)?;

    for i in 0..vert_count {
        let b = i * 6;
        writeln!(out, "v {} {} {}", vb[b], vb[b + 1], vb[b + 2]).map_err(CadError::Io)?;
    }
    for i in 0..vert_count {
        let b = i * 6;
        writeln!(out, "vn {} {} {}", vb[b + 3], vb[b + 4], vb[b + 5]).map_err(CadError::Io)?;
    }

    for tri in ib.chunks_exact(3) {
        // OBJ 1-indexed; "f v//vn" format
        let a = tri[0] + 1;
        let b = tri[1] + 1;
        let c = tri[2] + 1;
        writeln!(out, "f {a}//{a} {b}//{b} {c}//{c}").map_err(CadError::Io)?;
    }

    out.flush().map_err(CadError::Io)?;
    Ok(())
}
