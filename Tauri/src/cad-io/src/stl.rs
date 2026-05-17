use std::{fs::File, io::BufReader, path::Path};

use cad_core::{CadError, HeMesh, Point3f, Result};

// ─── Read ─────────────────────────────────────────────────────────────────

/// Load a binary or ASCII STL file.
/// Format detection (magic bytes) is handled automatically by `stl_io`.
pub fn read(path: &Path) -> Result<HeMesh> {
    let file = File::open(path).map_err(CadError::Io)?;
    let mut reader = BufReader::new(file);

    let stl = stl_io::read_stl(&mut reader)
        .map_err(|e| CadError::Mesh(e.to_string()))?;

    let positions: Vec<Point3f> = stl
        .vertices
        .iter()
        .map(|v| Point3f::new(v[0], v[1], v[2]))
        .collect();

    let indices: Vec<[u32; 3]> = stl
        .faces
        .iter()
        .map(|f| [f.vertices[0] as u32, f.vertices[1] as u32, f.vertices[2] as u32])
        .collect();

    tracing::debug!(
        path = %path.display(),
        vertices = positions.len(),
        faces = indices.len(),
        "STL loaded"
    );

    Ok(HeMesh::from_triangles(&positions, &indices))
}

// ─── Write ────────────────────────────────────────────────────────────────

/// Write a binary STL file from a `HeMesh`.
pub fn write_binary(mesh: &HeMesh, path: &Path) -> Result<()> {
    let vb = mesh.to_vertex_buffer(); // [x,y,z, nx,ny,nz] × n_verts
    let ib = mesh.to_index_buffer();

    let triangles: Vec<stl_io::Triangle> = ib
        .chunks_exact(3)
        .map(|tri| {
            let pos = |i: u32| -> stl_io::Vertex {
                let b = i as usize * 6;
                stl_io::Vector::new([vb[b], vb[b + 1], vb[b + 2]])
            };
            let nor = |i: u32| -> [f32; 3] {
                let b = i as usize * 6 + 3;
                [vb[b], vb[b + 1], vb[b + 2]]
            };
            let [n0, n1, n2] = [nor(tri[0]), nor(tri[1]), nor(tri[2])];
            let face_normal: stl_io::Normal = stl_io::Vector::new([
                (n0[0] + n1[0] + n2[0]) / 3.0,
                (n0[1] + n1[1] + n2[1]) / 3.0,
                (n0[2] + n1[2] + n2[2]) / 3.0,
            ]);
            stl_io::Triangle {
                normal: face_normal,
                vertices: [pos(tri[0]), pos(tri[1]), pos(tri[2])],
            }
        })
        .collect();

    let mut file = File::create(path).map_err(CadError::Io)?;
    stl_io::write_stl(&mut file, triangles.into_iter()).map_err(|e| {
        CadError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    })?;

    tracing::debug!(path = %path.display(), faces = ib.len() / 3, "STL written");
    Ok(())
}

// ─── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn tetrahedron() -> HeMesh {
        let pts = vec![
            Point3f::new(0.0f32, 0.0, 0.0),
            Point3f::new(1.0, 0.0, 0.0),
            Point3f::new(0.5, 1.0, 0.0),
            Point3f::new(0.5, 0.33, 1.0),
        ];
        let idx = vec![[0u32, 1, 2], [0, 2, 3], [0, 3, 1], [1, 3, 2]];
        HeMesh::from_triangles(&pts, &idx)
    }

    #[test]
    fn test_stl_round_trip() {
        let mesh = tetrahedron();
        let path = std::env::temp_dir().join("tlanticad_stl_test.stl");
        write_binary(&mesh, &path).expect("write STL");
        let loaded = read(&path).expect("read STL");
        assert_eq!(loaded.face_count(), mesh.face_count());
    }
}
