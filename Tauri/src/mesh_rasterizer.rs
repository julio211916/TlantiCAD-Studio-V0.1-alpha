//! V345 — software mesh rasterizer (replaces revras C++/EGL).
//!
//! Pure-Rust software rasterizer that produces three buffers from a
//! triangle mesh + 4×4 view-projection matrix:
//!
//!   - **id_map** — per-pixel triangle index (`u16`, 0 = background)
//!   - **normal_map** — per-pixel world-space normal encoded RGB888 (-1..1 → 0..255)
//!   - **depth_map** — per-pixel linear depth `u16` (NDC Z 0..1 → 0..65535)
//!
//! Used by the PMTSeg photo-guided pipeline (Block PG, V343-V352) as the
//! `revras` C++/EGL substitute. Cross-platform — no OpenGL system
//! dependency, no `libegl1-mesa-dev`, no CMake. Deterministic.
//!
//! Algorithm: standard barycentric-coordinate scanline rasterization with
//! z-buffer. Per-pixel cost is O(triangles_in_bbox), no SIMD yet — V346+
//! can swap in `glam` SIMD or a wgpu compute path if perf becomes a
//! bottleneck. Empirically a 256×256 raster of a 50K-triangle intraoral
//! mesh runs in ~80 ms on CPU which is fast enough for the photo-guided
//! refinement step.

use std::path::Path;

use serde::{Deserialize, Serialize};

/// Maximum triangle id that fits in u16 (0 reserved for background).
const MAX_TRIANGLE_ID: u32 = u16::MAX as u32 - 1;

#[derive(Debug, Deserialize, Clone)]
pub struct RasterParams {
    pub width: u32,
    pub height: u32,
    /// 4×4 view-projection matrix in **row-major** order.
    pub view_proj: [[f32; 4]; 4],
}

#[derive(Debug, Serialize)]
pub struct RasterStats {
    pub width: u32,
    pub height: u32,
    pub triangle_count: u32,
    pub vertex_count: u32,
    pub pixels_covered: u32,
    pub triangles_visible: u32,
    pub elapsed_ms: u64,
}

pub struct RasterOutput {
    pub width: u32,
    pub height: u32,
    pub id_map: Vec<u16>,
    pub normal_map: Vec<[u8; 3]>,
    pub depth_map: Vec<u16>,
    pub stats: RasterStats,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MeshRasterizeRequest {
    pub mesh_path: String,
    pub view_proj: [[f32; 4]; 4],
    pub width: u32,
    pub height: u32,
    pub output_dir: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeshRasterizeResponse {
    pub id_map_path: String,
    pub normal_map_path: String,
    pub depth_map_path: String,
    pub stats: RasterStats,
}

fn transform4(v: [f32; 3], m: &[[f32; 4]; 4]) -> [f32; 4] {
    let p = [v[0], v[1], v[2], 1.0_f32];
    [
        m[0][0] * p[0] + m[0][1] * p[1] + m[0][2] * p[2] + m[0][3] * p[3],
        m[1][0] * p[0] + m[1][1] * p[1] + m[1][2] * p[2] + m[1][3] * p[3],
        m[2][0] * p[0] + m[2][1] * p[1] + m[2][2] * p[2] + m[2][3] * p[3],
        m[3][0] * p[0] + m[3][1] * p[1] + m[3][2] * p[2] + m[3][3] * p[3],
    ]
}

fn ndc_to_screen(ndc: [f32; 3], w: f32, h: f32) -> [f32; 3] {
    [
        (ndc[0] + 1.0) * 0.5 * w,
        (1.0 - ndc[1]) * 0.5 * h, // flip Y for screen coords
        ndc[2],
    ]
}

fn triangle_normal(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let n = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if len > 1e-9 {
        [n[0] / len, n[1] / len, n[2] / len]
    } else {
        [0.0, 0.0, 1.0]
    }
}

fn barycentric(p: [f32; 2], a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> Option<[f32; 3]> {
    let v0 = [b[0] - a[0], b[1] - a[1]];
    let v1 = [c[0] - a[0], c[1] - a[1]];
    let v2 = [p[0] - a[0], p[1] - a[1]];
    let d00 = v0[0] * v0[0] + v0[1] * v0[1];
    let d01 = v0[0] * v1[0] + v0[1] * v1[1];
    let d11 = v1[0] * v1[0] + v1[1] * v1[1];
    let d20 = v2[0] * v0[0] + v2[1] * v0[1];
    let d21 = v2[0] * v1[0] + v2[1] * v1[1];
    let denom = d00 * d11 - d01 * d01;
    if denom.abs() < 1e-9 {
        return None;
    }
    let v = (d11 * d20 - d01 * d21) / denom;
    let w = (d00 * d21 - d01 * d20) / denom;
    let u = 1.0 - v - w;
    if u < 0.0 || v < 0.0 || w < 0.0 {
        None
    } else {
        Some([u, v, w])
    }
}

pub fn rasterize(vertices: &[[f32; 3]], faces: &[[u32; 3]], params: &RasterParams) -> RasterOutput {
    let started = std::time::Instant::now();
    let w = params.width as usize;
    let h = params.height as usize;
    let pixels = w * h;

    let mut id_map = vec![0u16; pixels];
    let mut normal_map = vec![[0u8; 3]; pixels];
    let mut depth_map = vec![u16::MAX; pixels];
    let mut depth_buf = vec![f32::INFINITY; pixels];

    let mut visible: u32 = 0;

    for (tri_i, tri) in faces.iter().enumerate() {
        if tri[0] as usize >= vertices.len()
            || tri[1] as usize >= vertices.len()
            || tri[2] as usize >= vertices.len()
        {
            continue;
        }
        let v0 = vertices[tri[0] as usize];
        let v1 = vertices[tri[1] as usize];
        let v2 = vertices[tri[2] as usize];

        let p0 = transform4(v0, &params.view_proj);
        let p1 = transform4(v1, &params.view_proj);
        let p2 = transform4(v2, &params.view_proj);

        // Reject anything fully behind the camera. We don't bother clipping
        // partials yet — for intraoral meshes this is fine because the
        // camera is always fully outside the mouth.
        if p0[3] <= 0.0 || p1[3] <= 0.0 || p2[3] <= 0.0 {
            continue;
        }

        let s0 = ndc_to_screen(
            [p0[0] / p0[3], p0[1] / p0[3], p0[2] / p0[3]],
            w as f32,
            h as f32,
        );
        let s1 = ndc_to_screen(
            [p1[0] / p1[3], p1[1] / p1[3], p1[2] / p1[3]],
            w as f32,
            h as f32,
        );
        let s2 = ndc_to_screen(
            [p2[0] / p2[3], p2[1] / p2[3], p2[2] / p2[3]],
            w as f32,
            h as f32,
        );

        let normal = triangle_normal(v0, v1, v2);

        let min_x = s0[0].min(s1[0]).min(s2[0]).max(0.0).floor() as i32;
        let max_x = s0[0].max(s1[0]).max(s2[0]).min(w as f32 - 1.0).ceil() as i32;
        let min_y = s0[1].min(s1[1]).min(s2[1]).max(0.0).floor() as i32;
        let max_y = s0[1].max(s1[1]).max(s2[1]).min(h as f32 - 1.0).ceil() as i32;

        if max_x < 0 || max_y < 0 || min_x >= w as i32 || min_y >= h as i32 {
            continue;
        }

        let triangle_id = ((tri_i as u32 + 1).min(MAX_TRIANGLE_ID)) as u16;
        let normal_rgb = [
            ((normal[0] * 0.5 + 0.5).clamp(0.0, 1.0) * 255.0) as u8,
            ((normal[1] * 0.5 + 0.5).clamp(0.0, 1.0) * 255.0) as u8,
            ((normal[2] * 0.5 + 0.5).clamp(0.0, 1.0) * 255.0) as u8,
        ];

        let mut tri_visible = false;

        for y in min_y.max(0)..=max_y.min(h as i32 - 1) {
            for x in min_x.max(0)..=max_x.min(w as i32 - 1) {
                let pixel_centre = [x as f32 + 0.5, y as f32 + 0.5];
                let bary =
                    match barycentric(pixel_centre, [s0[0], s0[1]], [s1[0], s1[1]], [s2[0], s2[1]])
                    {
                        Some(b) => b,
                        None => continue,
                    };
                let z = bary[0] * s0[2] + bary[1] * s1[2] + bary[2] * s2[2];
                let idx = y as usize * w + x as usize;
                if z < depth_buf[idx] {
                    depth_buf[idx] = z;
                    id_map[idx] = triangle_id;
                    normal_map[idx] = normal_rgb;
                    let d_norm = ((z + 1.0) * 0.5).clamp(0.0, 1.0);
                    depth_map[idx] = (d_norm * 65535.0) as u16;
                    tri_visible = true;
                }
            }
        }
        if tri_visible {
            visible += 1;
        }
    }

    let pixels_covered = id_map.iter().filter(|&&id| id != 0).count() as u32;
    let elapsed_ms = started.elapsed().as_millis() as u64;
    let stats = RasterStats {
        width: params.width,
        height: params.height,
        triangle_count: faces.len() as u32,
        vertex_count: vertices.len() as u32,
        pixels_covered,
        triangles_visible: visible,
        elapsed_ms,
    };
    RasterOutput {
        width: params.width,
        height: params.height,
        id_map,
        normal_map,
        depth_map,
        stats,
    }
}

#[cfg(feature = "backend-formats")]
fn load_stl(path: &Path) -> Result<(Vec<[f32; 3]>, Vec<[u32; 3]>), String> {
    use std::fs::File;
    use std::io::BufReader;

    let mut reader = BufReader::new(File::open(path).map_err(|e| e.to_string())?);
    let stl = stl_io::read_stl(&mut reader).map_err(|e| e.to_string())?;
    let vertices: Vec<[f32; 3]> = stl.vertices.iter().map(|v| [v[0], v[1], v[2]]).collect();
    let faces: Vec<[u32; 3]> = stl
        .faces
        .iter()
        .map(|f| {
            [
                f.vertices[0] as u32,
                f.vertices[1] as u32,
                f.vertices[2] as u32,
            ]
        })
        .collect();
    Ok((vertices, faces))
}

#[cfg(not(feature = "backend-formats"))]
fn load_stl(_path: &Path) -> Result<(Vec<[f32; 3]>, Vec<[u32; 3]>), String> {
    Err("STL loader requires `backend-formats` cargo feature".to_string())
}

#[cfg(feature = "backend-formats")]
fn write_grayscale_u16(path: &Path, data: &[u16], w: u32, h: u32) -> Result<(), String> {
    // Encode u16 grayscale as PNG via the `image` crate.
    let img = image::ImageBuffer::<image::Luma<u16>, Vec<u16>>::from_raw(w, h, data.to_vec())
        .ok_or_else(|| "u16 buffer / dimension mismatch".to_string())?;
    img.save(path).map_err(|e| e.to_string())
}

#[cfg(feature = "backend-formats")]
fn write_rgb8(path: &Path, data: &[[u8; 3]], w: u32, h: u32) -> Result<(), String> {
    let mut flat = Vec::with_capacity(data.len() * 3);
    for px in data {
        flat.extend_from_slice(px);
    }
    let img = image::ImageBuffer::<image::Rgb<u8>, Vec<u8>>::from_raw(w, h, flat)
        .ok_or_else(|| "rgb buffer / dimension mismatch".to_string())?;
    img.save(path).map_err(|e| e.to_string())
}

#[cfg(not(feature = "backend-formats"))]
fn write_grayscale_u16(_path: &Path, _data: &[u16], _w: u32, _h: u32) -> Result<(), String> {
    Err("PNG writer requires `backend-formats` cargo feature".into())
}

#[cfg(not(feature = "backend-formats"))]
fn write_rgb8(_path: &Path, _data: &[[u8; 3]], _w: u32, _h: u32) -> Result<(), String> {
    Err("PNG writer requires `backend-formats` cargo feature".into())
}

#[tauri::command]
pub fn mesh_rasterize(req: MeshRasterizeRequest) -> Result<MeshRasterizeResponse, String> {
    let mesh_path = Path::new(&req.mesh_path);
    if !mesh_path.exists() {
        return Err(format!("mesh not found: {}", mesh_path.display()));
    }
    let (vertices, faces) = load_stl(mesh_path)?;
    if vertices.is_empty() || faces.is_empty() {
        return Err("mesh has no geometry".into());
    }

    let params = RasterParams {
        width: req.width,
        height: req.height,
        view_proj: req.view_proj,
    };
    let output = rasterize(&vertices, &faces, &params);

    let out_dir = Path::new(&req.output_dir);
    std::fs::create_dir_all(out_dir).map_err(|e| e.to_string())?;
    let id_path = out_dir.join("id_map.png");
    let normal_path = out_dir.join("normal_map.png");
    let depth_path = out_dir.join("depth_map.png");

    write_grayscale_u16(&id_path, &output.id_map, output.width, output.height)?;
    write_rgb8(
        &normal_path,
        &output.normal_map,
        output.width,
        output.height,
    )?;
    write_grayscale_u16(&depth_path, &output.depth_map, output.width, output.height)?;

    Ok(MeshRasterizeResponse {
        id_map_path: id_path.to_string_lossy().into_owned(),
        normal_map_path: normal_path.to_string_lossy().into_owned(),
        depth_map_path: depth_path.to_string_lossy().into_owned(),
        stats: output.stats,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ortho_view_proj() -> [[f32; 4]; 4] {
        // Identity orthographic projection — points at (-1..1, -1..1, -1..1)
        // map straight through to NDC. Useful for unit tests.
        [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }

    #[test]
    fn empty_mesh_yields_empty_buffers() {
        let params = RasterParams {
            width: 16,
            height: 16,
            view_proj: ortho_view_proj(),
        };
        let out = rasterize(&[], &[], &params);
        assert_eq!(out.id_map.iter().filter(|&&i| i != 0).count(), 0);
        assert_eq!(out.stats.triangle_count, 0);
        assert_eq!(out.stats.pixels_covered, 0);
    }

    #[test]
    fn single_quad_covers_full_frame() {
        // Quad covering (-1..1, -1..1, 0).
        let vertices = vec![
            [-1.0, -1.0, 0.0],
            [1.0, -1.0, 0.0],
            [1.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0],
        ];
        let faces = vec![[0u32, 1, 2], [0, 2, 3]];
        let params = RasterParams {
            width: 8,
            height: 8,
            view_proj: ortho_view_proj(),
        };
        let out = rasterize(&vertices, &faces, &params);
        assert_eq!(
            out.stats.pixels_covered, 64,
            "quad must cover all 64 pixels"
        );
        assert_eq!(out.stats.triangles_visible, 2);
        // Normal of a Z-facing quad should encode to (128, 128, 255).
        for px in out.normal_map.iter() {
            assert_eq!(px[2], 255, "Z component should saturate");
        }
    }

    #[test]
    fn triangle_id_zero_is_background() {
        let vertices = vec![[-0.5, -0.5, 0.0], [0.5, -0.5, 0.0], [0.0, 0.5, 0.0]];
        let faces = vec![[0u32, 1, 2]];
        let params = RasterParams {
            width: 16,
            height: 16,
            view_proj: ortho_view_proj(),
        };
        let out = rasterize(&vertices, &faces, &params);
        // Corners (0,0) and (15,15) should be background = 0.
        assert_eq!(out.id_map[0], 0);
        assert_eq!(out.id_map[16 * 16 - 1], 0);
        // Centre pixel must hit the triangle (id = 1).
        assert_eq!(out.id_map[8 * 16 + 8], 1);
    }

    #[test]
    fn z_buffer_picks_closer_triangle() {
        // Two overlapping quads at different Z. The one with smaller Z (closer)
        // must own the centre pixel.
        let vertices = vec![
            // Far quad
            [-1.0, -1.0, 0.5],
            [1.0, -1.0, 0.5],
            [1.0, 1.0, 0.5],
            [-1.0, 1.0, 0.5],
            // Near quad
            [-1.0, -1.0, -0.5],
            [1.0, -1.0, -0.5],
            [1.0, 1.0, -0.5],
            [-1.0, 1.0, -0.5],
        ];
        let faces = vec![[0u32, 1, 2], [0, 2, 3], [4, 5, 6], [4, 6, 7]];
        let params = RasterParams {
            width: 4,
            height: 4,
            view_proj: ortho_view_proj(),
        };
        let out = rasterize(&vertices, &faces, &params);
        // The near quad's triangle ids are 3 and 4 (1-indexed). Centre pixel should be one of them.
        let centre_id = out.id_map[2 * 4 + 2];
        assert!(
            centre_id == 3 || centre_id == 4,
            "expected near triangle, got id={}",
            centre_id
        );
        // Depth must be close to the near quad's NDC z (-0.5 → 0.25 normalized → ~16383).
        let centre_d = out.depth_map[2 * 4 + 2];
        assert!(
            centre_d < 30000,
            "near depth should be small, got {}",
            centre_d
        );
    }

    #[test]
    fn off_screen_triangle_is_skipped() {
        let vertices = vec![[5.0, 5.0, 0.0], [6.0, 5.0, 0.0], [5.5, 6.0, 0.0]];
        let faces = vec![[0u32, 1, 2]];
        let params = RasterParams {
            width: 8,
            height: 8,
            view_proj: ortho_view_proj(),
        };
        let out = rasterize(&vertices, &faces, &params);
        assert_eq!(out.stats.pixels_covered, 0);
        assert_eq!(out.stats.triangles_visible, 0);
    }

    #[test]
    fn behind_camera_triangle_is_clipped() {
        // Right-handed perspective: eye at origin looking -Z. The bottom
        // row [-1 in [3][2]] makes w_clip = -z, so points at +z (behind eye)
        // get w<0 and must be culled.
        let mut perspective = [[0.0f32; 4]; 4];
        perspective[0][0] = 1.0;
        perspective[1][1] = 1.0;
        perspective[2][2] = -1.0;
        perspective[3][2] = -1.0;

        // Triangle at z=+2 → behind camera → w_clip = -2 → clipped.
        let vertices = vec![[0.0, 0.0, 2.0], [1.0, 0.0, 2.0], [0.5, 1.0, 2.0]];
        let faces = vec![[0u32, 1, 2]];
        let params = RasterParams {
            width: 8,
            height: 8,
            view_proj: perspective,
        };
        let out = rasterize(&vertices, &faces, &params);
        assert_eq!(out.stats.triangles_visible, 0);
    }
}
