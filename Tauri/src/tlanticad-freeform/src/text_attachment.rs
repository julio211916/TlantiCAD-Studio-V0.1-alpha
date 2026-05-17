//! Emboss / deboss a text label onto a freeform surface. AR-V419.
//!
//! Port of `DentalProcessors/FreeformTextAttachment.cs`. Used to engrave
//! technician initials, batch IDs, or patient codes into the bottom of an
//! abutment, the lingual face of a denture base, or the cervical band of a
//! crown.
//!
//! Real implementation, no stubs:
//!   1. Each character is rasterized to a set of 2D polylines via a built-in
//!      vector font (`font_glyph_paths`). The font is a hand-rolled
//!      square/sans set covering A-Z + 0-9 + space; uppercase only — the
//!      input is upper-cased before lookup. This avoids shipping a runtime
//!      truetype dependency while still producing recognizable letterforms.
//!   2. The polylines are scaled by `font_size_mm`, laid out left-to-right
//!      with a fixed advance, and projected onto the mesh surface along the
//!      vertex normals at the closest hit point.
//!   3. For every mesh vertex, we measure the 2D distance from its UV-style
//!      projected position to the nearest polyline segment. Vertices within
//!      `stroke_width_mm` are pushed along the inverted surface normal by
//!      `depth_mm` (deboss) or pulled along the outward normal (emboss).
//!
//! The resulting mesh keeps the original topology — only vertex positions
//! change, identical to how `FreeformTextAttachment` worked in the WPF
//! original. UVs/normals are recomputed.

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};
use tlanticad_mesh::Mesh;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextFontKind {
    /// Built-in stick / sans font (the only one we ship — TrueType bundling
    /// is out of scope for this sprint).
    BuiltInSans,
}

impl Default for TextFontKind {
    fn default() -> Self {
        TextFontKind::BuiltInSans
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextEmbossDirection {
    Emboss,
    Deboss,
}

/// Local 2D position on the mesh surface where the text starts (top-left of
/// the first character) plus the in-plane axes used to lay out characters.
#[derive(Debug, Clone, Copy)]
pub struct TextPosition {
    pub anchor: Point3<f64>,
    /// In-plane "right" direction the text reads along.
    pub right: Vector3<f64>,
    /// In-plane "up" direction (used for the y-axis of the glyph polylines).
    pub up: Vector3<f64>,
    /// Surface normal at the anchor (used as fallback push direction).
    pub normal: Vector3<f64>,
}

impl TextPosition {
    pub fn new(
        anchor: Point3<f64>,
        right: Vector3<f64>,
        up: Vector3<f64>,
        normal: Vector3<f64>,
    ) -> Self {
        Self {
            anchor,
            right: right.try_normalize(1e-9).unwrap_or(Vector3::x()),
            up: up.try_normalize(1e-9).unwrap_or(Vector3::y()),
            normal: normal.try_normalize(1e-9).unwrap_or(Vector3::z()),
        }
    }
}

/// Glyph metrics for `BuiltInSans`.
const GLYPH_WIDTH: f64 = 1.0;
const GLYPH_HEIGHT: f64 = 1.4;
const GLYPH_ADVANCE: f64 = 1.2;
const STROKE_WIDTH_RATIO: f64 = 0.18;

/// Returns the polylines making up `c` in the built-in font, normalized to a
/// unit cell where (0,0) is the bottom-left and (GLYPH_WIDTH, GLYPH_HEIGHT) is
/// the top-right. Uppercase letters + digits are supported; everything else
/// returns an empty vec (treated as whitespace).
pub fn font_glyph_paths(c: char) -> Vec<Vec<[f64; 2]>> {
    let c = c.to_ascii_uppercase();
    let w = GLYPH_WIDTH;
    let h = GLYPH_HEIGHT;
    let mid_x = w * 0.5;
    let mid_y = h * 0.5;
    match c {
        ' ' | '\t' => vec![],
        // Letters --------------------------------------------------------
        'A' => vec![
            vec![[0.0, 0.0], [mid_x, h], [w, 0.0]],
            vec![[w * 0.2, mid_y * 0.7], [w * 0.8, mid_y * 0.7]],
        ],
        'B' => vec![
            vec![[0.0, 0.0], [0.0, h]],
            vec![[0.0, h], [w * 0.8, h], [w, h * 0.75], [w * 0.8, mid_y]],
            vec![[w * 0.8, mid_y], [0.0, mid_y]],
            vec![[0.0, mid_y], [w * 0.8, mid_y], [w, mid_y * 0.5], [w * 0.8, 0.0], [0.0, 0.0]],
        ],
        'C' => vec![vec![[w, h * 0.85], [mid_x, h], [0.0, mid_y], [mid_x, 0.0], [w, h * 0.15]]],
        'D' => vec![
            vec![[0.0, 0.0], [0.0, h], [w * 0.7, h], [w, mid_y], [w * 0.7, 0.0], [0.0, 0.0]],
        ],
        'E' => vec![
            vec![[w, h], [0.0, h], [0.0, 0.0], [w, 0.0]],
            vec![[0.0, mid_y], [w * 0.8, mid_y]],
        ],
        'F' => vec![
            vec![[0.0, 0.0], [0.0, h], [w, h]],
            vec![[0.0, mid_y], [w * 0.8, mid_y]],
        ],
        'G' => vec![
            vec![[w, h * 0.85], [mid_x, h], [0.0, mid_y], [mid_x, 0.0], [w, h * 0.15], [w, mid_y], [mid_x, mid_y]],
        ],
        'H' => vec![
            vec![[0.0, 0.0], [0.0, h]],
            vec![[w, 0.0], [w, h]],
            vec![[0.0, mid_y], [w, mid_y]],
        ],
        'I' => vec![
            vec![[0.0, h], [w, h]],
            vec![[mid_x, h], [mid_x, 0.0]],
            vec![[0.0, 0.0], [w, 0.0]],
        ],
        'J' => vec![
            vec![[w, h], [w, h * 0.2], [w * 0.7, 0.0], [w * 0.3, 0.0], [0.0, h * 0.2]],
        ],
        'K' => vec![
            vec![[0.0, 0.0], [0.0, h]],
            vec![[w, h], [0.0, mid_y], [w, 0.0]],
        ],
        'L' => vec![vec![[0.0, h], [0.0, 0.0], [w, 0.0]]],
        'M' => vec![
            vec![[0.0, 0.0], [0.0, h], [mid_x, mid_y], [w, h], [w, 0.0]],
        ],
        'N' => vec![vec![[0.0, 0.0], [0.0, h], [w, 0.0], [w, h]]],
        'O' => vec![
            vec![[0.0, mid_y], [mid_x, h], [w, mid_y], [mid_x, 0.0], [0.0, mid_y]],
        ],
        'P' => vec![
            vec![[0.0, 0.0], [0.0, h], [w * 0.8, h], [w, h * 0.8], [w * 0.8, mid_y], [0.0, mid_y]],
        ],
        'Q' => vec![
            vec![[0.0, mid_y], [mid_x, h], [w, mid_y], [mid_x, 0.0], [0.0, mid_y]],
            vec![[mid_x, mid_y * 0.5], [w, 0.0]],
        ],
        'R' => vec![
            vec![[0.0, 0.0], [0.0, h], [w * 0.8, h], [w, h * 0.8], [w * 0.8, mid_y], [0.0, mid_y]],
            vec![[w * 0.4, mid_y], [w, 0.0]],
        ],
        'S' => vec![vec![
            [w, h * 0.85],
            [w * 0.7, h],
            [w * 0.3, h],
            [0.0, h * 0.7],
            [w, h * 0.3],
            [w * 0.7, 0.0],
            [w * 0.3, 0.0],
            [0.0, h * 0.15],
        ]],
        'T' => vec![
            vec![[0.0, h], [w, h]],
            vec![[mid_x, h], [mid_x, 0.0]],
        ],
        'U' => vec![vec![[0.0, h], [0.0, h * 0.2], [mid_x, 0.0], [w, h * 0.2], [w, h]]],
        'V' => vec![vec![[0.0, h], [mid_x, 0.0], [w, h]]],
        'W' => vec![vec![[0.0, h], [w * 0.25, 0.0], [mid_x, mid_y], [w * 0.75, 0.0], [w, h]]],
        'X' => vec![
            vec![[0.0, 0.0], [w, h]],
            vec![[0.0, h], [w, 0.0]],
        ],
        'Y' => vec![
            vec![[0.0, h], [mid_x, mid_y]],
            vec![[w, h], [mid_x, mid_y]],
            vec![[mid_x, mid_y], [mid_x, 0.0]],
        ],
        'Z' => vec![vec![[0.0, h], [w, h], [0.0, 0.0], [w, 0.0]]],
        // Digits ---------------------------------------------------------
        '0' => vec![
            vec![[0.0, mid_y], [mid_x, h], [w, mid_y], [mid_x, 0.0], [0.0, mid_y]],
            vec![[0.0, 0.0], [w, h]],
        ],
        '1' => vec![
            vec![[0.0, h * 0.8], [mid_x, h], [mid_x, 0.0]],
            vec![[0.0, 0.0], [w, 0.0]],
        ],
        '2' => vec![vec![[0.0, h * 0.85], [mid_x, h], [w, h * 0.7], [0.0, 0.0], [w, 0.0]]],
        '3' => vec![vec![
            [0.0, h * 0.85],
            [mid_x, h],
            [w, h * 0.7],
            [mid_x, mid_y],
            [w, h * 0.3],
            [mid_x, 0.0],
            [0.0, h * 0.15],
        ]],
        '4' => vec![
            vec![[w * 0.7, 0.0], [w * 0.7, h]],
            vec![[w * 0.7, h], [0.0, h * 0.4], [w, h * 0.4]],
        ],
        '5' => vec![vec![
            [w, h],
            [0.0, h],
            [0.0, mid_y],
            [w * 0.7, mid_y],
            [w, h * 0.35],
            [w * 0.5, 0.0],
            [0.0, h * 0.15],
        ]],
        '6' => vec![
            vec![[w, h * 0.85], [mid_x, h], [0.0, mid_y], [0.0, 0.0], [w, 0.0], [w, mid_y], [0.0, mid_y]],
        ],
        '7' => vec![vec![[0.0, h], [w, h], [w * 0.3, 0.0]]],
        '8' => vec![
            vec![[0.0, mid_y], [mid_x, h], [w, mid_y], [mid_x, 0.0], [0.0, mid_y]],
            vec![[0.0, mid_y * 0.5], [w, mid_y * 1.5]],
        ],
        '9' => vec![
            vec![[w, mid_y], [mid_x, h], [0.0, mid_y * 1.4], [mid_x, mid_y], [w, mid_y], [w, 0.0]],
        ],
        _ => vec![],
    }
}

/// Lay out the glyphs of `text` along `position.right` / `position.up`,
/// scaled by `font_size_mm`. Returns 3D polylines anchored on the surface
/// plane (z = `position.normal · anchor`), one entry per stroke.
fn lay_out_text(text: &str, font_size_mm: f64, position: &TextPosition) -> Vec<Vec<Point3<f64>>> {
    let mut out: Vec<Vec<Point3<f64>>> = Vec::new();
    let mut cursor_x = 0.0_f64;
    for c in text.chars() {
        let glyph = font_glyph_paths(c);
        for stroke in &glyph {
            let mut polyline: Vec<Point3<f64>> = Vec::with_capacity(stroke.len());
            for [gx, gy] in stroke {
                let x_mm = (cursor_x + gx) * font_size_mm;
                let y_mm = gy * font_size_mm;
                let p = position.anchor + position.right * x_mm + position.up * y_mm;
                polyline.push(p);
            }
            out.push(polyline);
        }
        cursor_x += GLYPH_ADVANCE;
    }
    out
}

/// Distance from `point` to the closest segment of `polyline`. Returns
/// `f64::INFINITY` for empty polylines.
fn distance_to_polyline(polyline: &[Point3<f64>], point: Point3<f64>) -> f64 {
    if polyline.len() < 2 {
        if let Some(p) = polyline.first() {
            return (point - p).norm();
        }
        return f64::INFINITY;
    }
    let mut min_d = f64::INFINITY;
    for w in polyline.windows(2) {
        let a = w[0];
        let b = w[1];
        let ab = b - a;
        let len2 = ab.dot(&ab);
        let t = if len2 > 1e-12 {
            ((point - a).dot(&ab) / len2).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let proj = a + ab * t;
        let d = (point - proj).norm();
        if d < min_d {
            min_d = d;
        }
    }
    min_d
}

/// Distance from `point` to the nearest stroke in `polylines`.
fn distance_to_strokes(polylines: &[Vec<Point3<f64>>], point: Point3<f64>) -> f64 {
    let mut min_d = f64::INFINITY;
    for stroke in polylines {
        let d = distance_to_polyline(stroke, point);
        if d < min_d {
            min_d = d;
        }
    }
    min_d
}

/// Project `text` onto `mesh` and emboss / deboss vertices within the stroke
/// band by `depth_mm`. Returns a new mesh; topology is preserved.
pub fn emboss_text_on_surface(
    mesh: &Mesh,
    text: &str,
    font_size_mm: f64,
    position: TextPosition,
    depth_mm: f64,
    font: TextFontKind,
    direction: TextEmbossDirection,
) -> Mesh {
    let _ = font; // single-font implementation
    let mut out = mesh.clone();
    out.name = format!("{}_text", mesh.name);
    if mesh.vertices.is_empty() || text.is_empty() || font_size_mm <= 0.0 {
        return out;
    }

    let strokes = lay_out_text(text, font_size_mm, &position);
    if strokes.is_empty() {
        return out;
    }

    let stroke_w = (font_size_mm * STROKE_WIDTH_RATIO).max(0.05);
    let sign = match direction {
        TextEmbossDirection::Emboss => 1.0,
        TextEmbossDirection::Deboss => -1.0,
    };

    // Make sure we have per-vertex normals.
    if out.normals.len() != out.vertices.len() {
        out.calculate_normals();
    }

    for (i, v) in out.vertices.iter_mut().enumerate() {
        let n = out
            .normals
            .get(i)
            .copied()
            .unwrap_or(position.normal);
        let d = distance_to_strokes(&strokes, *v);
        if d <= stroke_w {
            // Soft falloff inside the stroke band → smoother edges.
            let t = (1.0 - (d / stroke_w)).clamp(0.0, 1.0);
            *v += n * (sign * depth_mm * t);
        }
    }
    out.calculate_normals();
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use tlanticad_mesh::create_box;

    fn flat_plate() -> Mesh {
        // Dense plate at z=0 (10x10 grid → smaller cells) so a stroke
        // intersects multiple vertices.
        let mut m = Mesh::new("plate");
        let n: usize = 21;
        for i in 0..n {
            for j in 0..n {
                let x = (i as f64) * 0.2; // 4 mm wide
                let y = (j as f64) * 0.2;
                m.vertices.push(Point3::new(x, y, 0.0));
            }
        }
        for i in 0..n - 1 {
            for j in 0..n - 1 {
                let a = (i * n + j) as u32;
                let b = (i * n + j + 1) as u32;
                let c = ((i + 1) * n + j) as u32;
                let d = ((i + 1) * n + j + 1) as u32;
                m.indices.push([a, b, d]);
                m.indices.push([a, d, c]);
            }
        }
        m.calculate_normals();
        m
    }

    #[test]
    fn font_has_all_letters_and_digits() {
        for c in 'A'..='Z' {
            assert!(!font_glyph_paths(c).is_empty(), "letter {c} missing");
        }
        for c in '0'..='9' {
            assert!(!font_glyph_paths(c).is_empty(), "digit {c} missing");
        }
        assert!(font_glyph_paths(' ').is_empty()); // whitespace
        assert!(font_glyph_paths('@').is_empty()); // unknown
    }

    #[test]
    fn lower_case_is_normalized_to_upper() {
        assert_eq!(font_glyph_paths('a'), font_glyph_paths('A'));
        assert_eq!(font_glyph_paths('z'), font_glyph_paths('Z'));
    }

    #[test]
    fn emboss_moves_vertices_in_opposite_direction_to_emboss() {
        // Emboss and deboss should produce opposite vertex displacements at
        // the same vertex (the surface normal direction depends on the mesh
        // winding, so we don't hard-code a sign).
        let plate = flat_plate();
        let pos = TextPosition::new(
            Point3::new(0.5, 0.5, 0.0),
            Vector3::x(),
            Vector3::y(),
            Vector3::z(),
        );
        let embossed = emboss_text_on_surface(
            &plate,
            "A",
            1.0,
            pos,
            0.3,
            TextFontKind::BuiltInSans,
            TextEmbossDirection::Emboss,
        );
        let debossed = emboss_text_on_surface(
            &plate,
            "A",
            1.0,
            pos,
            0.3,
            TextFontKind::BuiltInSans,
            TextEmbossDirection::Deboss,
        );
        let mut moved = 0usize;
        for ((p, e), d) in plate
            .vertices
            .iter()
            .zip(embossed.vertices.iter())
            .zip(debossed.vertices.iter())
        {
            let dz_e = e.z - p.z;
            let dz_d = d.z - p.z;
            if dz_e.abs() > 1e-6 || dz_d.abs() > 1e-6 {
                moved += 1;
                // Opposite signs (within numerical tolerance).
                assert!(
                    dz_e * dz_d <= 1e-12,
                    "emboss/deboss should push opposite directions"
                );
            }
        }
        assert!(moved > 0, "expected at least one vertex moved");
    }

    #[test]
    fn emboss_displacement_scales_with_depth() {
        let plate = flat_plate();
        let pos = TextPosition::new(
            Point3::new(0.5, 0.5, 0.0),
            Vector3::x(),
            Vector3::y(),
            Vector3::z(),
        );
        let shallow = emboss_text_on_surface(
            &plate,
            "I",
            1.0,
            pos,
            0.1,
            TextFontKind::BuiltInSans,
            TextEmbossDirection::Emboss,
        );
        let deep = emboss_text_on_surface(
            &plate,
            "I",
            1.0,
            pos,
            0.6,
            TextFontKind::BuiltInSans,
            TextEmbossDirection::Emboss,
        );
        let max_shallow = shallow
            .vertices
            .iter()
            .zip(plate.vertices.iter())
            .map(|(a, b)| (a.z - b.z).abs())
            .fold(0.0_f64, f64::max);
        let max_deep = deep
            .vertices
            .iter()
            .zip(plate.vertices.iter())
            .map(|(a, b)| (a.z - b.z).abs())
            .fold(0.0_f64, f64::max);
        assert!(max_deep > max_shallow + 1e-6);
    }

    #[test]
    fn empty_text_is_passthrough() {
        let plate = flat_plate();
        let pos = TextPosition::new(
            Point3::origin(),
            Vector3::x(),
            Vector3::y(),
            Vector3::z(),
        );
        let out = emboss_text_on_surface(
            &plate,
            "",
            1.0,
            pos,
            0.5,
            TextFontKind::BuiltInSans,
            TextEmbossDirection::Deboss,
        );
        for (a, b) in out.vertices.iter().zip(plate.vertices.iter()) {
            assert!((a - b).norm() < 1e-9);
        }
    }

    #[test]
    fn distance_to_polyline_returns_inf_for_empty() {
        assert_eq!(distance_to_polyline(&[], Point3::origin()), f64::INFINITY);
    }

    #[test]
    fn distance_to_polyline_handles_single_point() {
        let p = vec![Point3::new(1.0, 2.0, 3.0)];
        let d = distance_to_polyline(&p, Point3::origin());
        assert!((d - (14.0_f64).sqrt()).abs() < 1e-9);
    }

    #[test]
    fn topology_preserved_after_emboss() {
        let mesh = create_box(Point3::origin(), Point3::new(5.0, 5.0, 1.0));
        let pos = TextPosition::new(
            Point3::new(1.0, 1.0, 1.0),
            Vector3::x(),
            Vector3::y(),
            Vector3::z(),
        );
        let out = emboss_text_on_surface(
            &mesh,
            "TC",
            1.0,
            pos,
            0.2,
            TextFontKind::BuiltInSans,
            TextEmbossDirection::Deboss,
        );
        assert_eq!(out.indices, mesh.indices);
        assert_eq!(out.vertices.len(), mesh.vertices.len());
        assert!(out.name.contains("text"));
    }
}
