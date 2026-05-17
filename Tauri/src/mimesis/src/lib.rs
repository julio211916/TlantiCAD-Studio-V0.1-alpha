//! # Mimesis
//!
//! Generate 3D meshes from 2D images using contour tracing and polygon extrusion.
//!
//! ## Pipeline
//! 1. Extract binary mask from image (alpha / luminance / channel)
//! 2. Trace polygon contours (Theo Pavlidis algorithm)
//! 3. Simplify polygons (Ramer-Douglas-Peucker)
//! 4. Smooth polygons (Chaikin corner-cutting)
//! 5. Triangulate (Earcut)
//! 6. Extrude into 3D
//! 7. Export to OBJ/MTL

pub mod config;
pub mod contour;
pub mod error;
pub mod export;
pub mod extrude;
pub mod simplify;
pub mod triangulate;

pub use config::{MaskMethod, MimesisConfig};
pub use error::{MimesisError, Result};
pub use extrude::{Mesh3D, TexCoord, Vertex3D};

use image::DynamicImage;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::info;

/// Result of processing a single image.
#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessResult {
    /// Path to the generated OBJ file.
    pub obj_path: PathBuf,
    /// Path to the generated MTL file.
    pub mtl_path: PathBuf,
    /// Path to the texture copy.
    pub texture_path: PathBuf,
    /// Number of vertices in the mesh.
    pub vertex_count: usize,
    /// Number of triangles in the mesh.
    pub triangle_count: usize,
    /// Number of contours found.
    pub contour_count: usize,
}

/// Mesh statistics (for the frontend).
#[derive(Debug, Serialize, Deserialize)]
pub struct MeshStats {
    pub vertex_count: usize,
    pub triangle_count: usize,
    pub contour_count: usize,
    pub polygon_vertices: usize,
    pub simplified_vertices: usize,
    pub extrude_height: f64,
    pub image_width: u32,
    pub image_height: u32,
}

/// Process a single image through the full pipeline.
pub fn process_image(
    input_path: &Path,
    output_dir: &Path,
    config: &MimesisConfig,
    mask_path: Option<&Path>,
) -> Result<ProcessResult> {
    // Load input image
    let img = image::open(input_path)?;
    let (img_w, img_h) = (img.width(), img.height());

    info!(
        "Processing image: {}x{} from {:?}",
        img_w, img_h, input_path
    );

    // Extract or load mask
    let mask = if let Some(mp) = mask_path {
        let mask_img = image::open(mp)?;
        contour::extract_mask(&mask_img, MaskMethod::Alpha, config.threshold)
    } else {
        contour::extract_mask(&img, config.mask_method, config.threshold)
    };

    // Trace contours
    let contours = contour::find_all_contours(&mask, config.min_polygon_dimension);
    if contours.is_empty() {
        return Err(MimesisError::NoContours);
    }

    info!("Found {} contour(s)", contours.len());

    // Use the largest contour
    let raw_polygon = &contours[0];

    // Simplify
    let simplified = simplify::simplify_polygon(raw_polygon, config.simplify_tolerance);
    info!(
        "Simplified: {} → {} vertices",
        raw_polygon.len(),
        simplified.len()
    );

    // Smooth
    let smoothed = simplify::smooth_polygon(&simplified, config.smooth_iterations);

    // Triangulate
    let triangles = triangulate::triangulate_polygon(&smoothed)?;
    info!("Triangulated: {} triangles", triangles.len() / 3);

    // Extrude
    let mesh = extrude::extrude_polygon(&smoothed, &triangles, config.extrude_height, img_w, img_h);
    info!(
        "Extruded: {} vertices, {} triangles",
        mesh.vertex_count, mesh.triangle_count
    );

    // Prepare output
    std::fs::create_dir_all(output_dir)?;
    let textures_dir = output_dir.join("textures");
    std::fs::create_dir_all(&textures_dir)?;

    let stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("mesh");

    let obj_path = output_dir.join(format!("{}_0.obj", stem));
    let mtl_path = output_dir.join(format!("{}_0.mtl", stem));
    let texture_name = format!("{}.png", stem);
    let texture_path = textures_dir.join(&texture_name);

    // Copy texture
    img.save(&texture_path)?;

    // Write OBJ
    export::write_obj(&mesh, &obj_path, Some(&format!("{}_0", stem)))?;

    // Write MTL
    export::write_mtl(&mtl_path, &format!("{}_0", stem), &texture_name)?;

    info!("Exported: {:?}", obj_path);

    Ok(ProcessResult {
        obj_path,
        mtl_path,
        texture_path,
        vertex_count: mesh.vertex_count,
        triangle_count: mesh.triangle_count,
        contour_count: contours.len(),
    })
}

/// Process a single image and return just the mesh data (no file I/O).
/// Useful for preview in the frontend.
pub fn generate_mesh(
    img: &DynamicImage,
    config: &MimesisConfig,
) -> Result<(Mesh3D, MeshStats)> {
    let (img_w, img_h) = (img.width(), img.height());
    let mask = contour::extract_mask(img, config.mask_method, config.threshold);

    let contours = contour::find_all_contours(&mask, config.min_polygon_dimension);
    if contours.is_empty() {
        return Err(MimesisError::NoContours);
    }

    let raw_polygon = &contours[0];
    let simplified = simplify::simplify_polygon(raw_polygon, config.simplify_tolerance);
    let smoothed = simplify::smooth_polygon(&simplified, config.smooth_iterations);
    let triangles = triangulate::triangulate_polygon(&smoothed)?;
    let mesh = extrude::extrude_polygon(&smoothed, &triangles, config.extrude_height, img_w, img_h);

    let stats = MeshStats {
        vertex_count: mesh.vertex_count,
        triangle_count: mesh.triangle_count,
        contour_count: contours.len(),
        polygon_vertices: raw_polygon.len(),
        simplified_vertices: smoothed.len(),
        extrude_height: config.extrude_height,
        image_width: img_w,
        image_height: img_h,
    };

    Ok((mesh, stats))
}

/// Generate the default configuration (for --generate-config).
pub fn default_config_json() -> String {
    serde_json::to_string_pretty(&MimesisConfig::default()).unwrap_or_default()
}
