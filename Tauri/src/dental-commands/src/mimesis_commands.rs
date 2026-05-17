//! Tauri IPC Commands for Mimesis — 3D mesh generation from images

use crate::{CommandResult, DentalCommandError};
use mimesis::{MimesisConfig, MeshStats, ProcessResult};
use std::path::PathBuf;
use tracing::info;

/// Process an image file through the full mimesis pipeline (contour → extrude → OBJ).
#[tauri::command]
pub async fn mimesis_process_image(
    input_path: String,
    output_dir: String,
    config: Option<MimesisConfig>,
    mask_path: Option<String>,
) -> CommandResult<ProcessResult> {
    let cfg = config.unwrap_or_default();
    let input = PathBuf::from(&input_path);
    let output = PathBuf::from(&output_dir);
    let mask = mask_path.map(PathBuf::from);

    info!("Mimesis: processing {:?} → {:?}", input, output);

    let result = mimesis::process_image(
        &input,
        &output,
        &cfg,
        mask.as_deref(),
    )
    .map_err(|e| DentalCommandError::Internal(format!("Mimesis error: {}", e)))?;

    Ok(result)
}

/// Generate mesh preview from base64-encoded image data (no file I/O).
#[tauri::command]
pub fn mimesis_generate_mesh_preview(
    image_base64: String,
    config: Option<MimesisConfig>,
) -> CommandResult<MeshStats> {
    let cfg = config.unwrap_or_default();

    // Decode base64
    let bytes = simple_base64_decode(&image_base64)
        .map_err(|e| DentalCommandError::Validation(format!("Invalid base64: {}", e)))?;

    let img = image::load_from_memory(&bytes)
        .map_err(|e| DentalCommandError::Internal(format!("Image decode error: {}", e)))?;

    let (_mesh, stats) = mimesis::generate_mesh(&img, &cfg)
        .map_err(|e| DentalCommandError::Internal(format!("Mimesis error: {}", e)))?;

    Ok(stats)
}

/// Return the default mimesis configuration.
#[tauri::command]
pub fn mimesis_default_config() -> CommandResult<MimesisConfig> {
    Ok(MimesisConfig::default())
}

// Simple base64 decoder (avoids extra dependency)
fn simple_base64_decode(input: &str) -> Result<Vec<u8>, String> {
    // Strip data URI prefix if present
    let data = if let Some(pos) = input.find(",") {
        &input[pos + 1..]
    } else {
        input
    };

    // Standard base64 decode
    let chars: Vec<u8> = data.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    let mut output = Vec::with_capacity(chars.len() * 3 / 4);

    let table = |c: u8| -> Result<u8, String> {
        match c {
            b'A'..=b'Z' => Ok(c - b'A'),
            b'a'..=b'z' => Ok(c - b'a' + 26),
            b'0'..=b'9' => Ok(c - b'0' + 52),
            b'+' => Ok(62),
            b'/' => Ok(63),
            b'=' => Ok(0),
            _ => Err(format!("Invalid base64 character: {}", c as char)),
        }
    };

    let mut i = 0;
    while i + 3 < chars.len() {
        let a = table(chars[i])?;
        let b = table(chars[i + 1])?;
        let c = table(chars[i + 2])?;
        let d = table(chars[i + 3])?;

        output.push((a << 2) | (b >> 4));
        if chars[i + 2] != b'=' {
            output.push((b << 4) | (c >> 2));
        }
        if chars[i + 3] != b'=' {
            output.push((c << 6) | d);
        }
        i += 4;
    }

    Ok(output)
}
