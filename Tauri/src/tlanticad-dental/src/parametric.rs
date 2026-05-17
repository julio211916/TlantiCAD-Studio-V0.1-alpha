//! Parametric tooth design: generate anatomical tooth shapes from parameters

use nalgebra::Point3;
use serde::{Deserialize, Serialize};

/// FDI tooth numbering quadrant
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Quadrant {
    UpperRight = 1,
    UpperLeft = 2,
    LowerLeft = 3,
    LowerRight = 4,
}

/// Tooth type based on FDI position
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ToothType {
    CentralIncisor,
    LateralIncisor,
    Canine,
    FirstPremolar,
    SecondPremolar,
    FirstMolar,
    SecondMolar,
    ThirdMolar,
}

impl ToothType {
    pub fn from_fdi(number: u8) -> Option<Self> {
        let pos = number % 10;
        match pos {
            1 => Some(ToothType::CentralIncisor),
            2 => Some(ToothType::LateralIncisor),
            3 => Some(ToothType::Canine),
            4 => Some(ToothType::FirstPremolar),
            5 => Some(ToothType::SecondPremolar),
            6 => Some(ToothType::FirstMolar),
            7 => Some(ToothType::SecondMolar),
            8 => Some(ToothType::ThirdMolar),
            _ => None,
        }
    }
}

/// Parameters for generating a parametric tooth
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToothParams {
    pub fdi_number: u8,
    pub mesio_distal_width: f64,   // mm
    pub bucco_lingual_width: f64,  // mm
    pub crown_height: f64,         // mm
    pub cusp_height: f64,          // mm (for premolars/molars)
    pub cusp_angle: f64,           // degrees
    pub fissure_depth: f64,        // mm
    pub contact_tightness: f64,    // 0..1
    pub emergence_angle: f64,      // degrees
}

impl ToothParams {
    /// Default parameters for a given FDI tooth number
    pub fn default_for(fdi: u8) -> Self {
        let tooth_type = ToothType::from_fdi(fdi).unwrap_or(ToothType::FirstMolar);
        match tooth_type {
            ToothType::CentralIncisor => Self {
                fdi_number: fdi,
                mesio_distal_width: 8.5,
                bucco_lingual_width: 7.0,
                crown_height: 10.5,
                cusp_height: 0.0,
                cusp_angle: 0.0,
                fissure_depth: 0.0,
                contact_tightness: 0.5,
                emergence_angle: 12.0,
            },
            ToothType::LateralIncisor => Self {
                fdi_number: fdi,
                mesio_distal_width: 6.5,
                bucco_lingual_width: 6.0,
                crown_height: 9.0,
                cusp_height: 0.0,
                cusp_angle: 0.0,
                fissure_depth: 0.0,
                contact_tightness: 0.5,
                emergence_angle: 14.0,
            },
            ToothType::Canine => Self {
                fdi_number: fdi,
                mesio_distal_width: 7.5,
                bucco_lingual_width: 8.0,
                crown_height: 10.0,
                cusp_height: 2.5,
                cusp_angle: 45.0,
                fissure_depth: 0.0,
                contact_tightness: 0.5,
                emergence_angle: 10.0,
            },
            ToothType::FirstPremolar | ToothType::SecondPremolar => Self {
                fdi_number: fdi,
                mesio_distal_width: 7.0,
                bucco_lingual_width: 9.0,
                crown_height: 8.5,
                cusp_height: 2.0,
                cusp_angle: 33.0,
                fissure_depth: 1.5,
                contact_tightness: 0.5,
                emergence_angle: 8.0,
            },
            ToothType::FirstMolar | ToothType::SecondMolar | ToothType::ThirdMolar => Self {
                fdi_number: fdi,
                mesio_distal_width: 10.0,
                bucco_lingual_width: 11.0,
                crown_height: 7.5,
                cusp_height: 2.5,
                cusp_angle: 30.0,
                fissure_depth: 2.0,
                contact_tightness: 0.5,
                emergence_angle: 6.0,
            },
        }
    }
}

/// Generate a parametric tooth mesh
/// Returns (vertices, indices)
pub fn generate_tooth(params: &ToothParams) -> (Vec<Point3<f64>>, Vec<[u32; 3]>) {
    let tooth_type = ToothType::from_fdi(params.fdi_number)
        .unwrap_or(ToothType::FirstMolar);

    match tooth_type {
        ToothType::CentralIncisor | ToothType::LateralIncisor => {
            generate_incisor(params)
        }
        ToothType::Canine => {
            generate_canine(params)
        }
        ToothType::FirstPremolar | ToothType::SecondPremolar => {
            generate_premolar(params)
        }
        _ => {
            generate_molar(params)
        }
    }
}

fn generate_incisor(params: &ToothParams) -> (Vec<Point3<f64>>, Vec<[u32; 3]>) {
    let w = params.mesio_distal_width / 2.0;
    let d = params.bucco_lingual_width / 2.0;
    let h = params.crown_height;

    let n_radial = 16;
    let n_height = 10;

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for j in 0..=n_height {
        let t = j as f64 / n_height as f64;
        let z = t * h;

        // Cross-section: ellipse that tapers toward incisal edge
        let taper = 1.0 - t * 0.3; // Slight taper
        let rx = w * taper;
        let ry = d * taper * (1.0 - t * 0.4); // Flatten toward incisal
        let emergence = (params.emergence_angle.to_radians() * t).sin() * 0.5;

        for i in 0..n_radial {
            let theta = std::f64::consts::TAU * i as f64 / n_radial as f64;
            let x = rx * theta.cos() + emergence;
            let y = ry * theta.sin();
            vertices.push(Point3::new(x, y, z));
        }
    }

    // Triangulate
    for j in 0..n_height {
        for i in 0..n_radial {
            let a = j * n_radial + i;
            let b = j * n_radial + (i + 1) % n_radial;
            let c = (j + 1) * n_radial + i;
            let d = (j + 1) * n_radial + (i + 1) % n_radial;
            indices.push([a as u32, b as u32, c as u32]);
            indices.push([b as u32, d as u32, c as u32]);
        }
    }

    // Cap bottom
    let bottom_center = vertices.len() as u32;
    vertices.push(Point3::new(0.0, 0.0, 0.0));
    for i in 0..n_radial {
        indices.push([bottom_center, (i + 1) % n_radial as u32, i as u32]);
    }

    // Cap top (incisal)
    let top_center = vertices.len() as u32;
    vertices.push(Point3::new(0.0, 0.0, h));
    let top_ring_start = n_height * n_radial;
    for i in 0..n_radial {
        let a = (top_ring_start + i) as u32;
        let b = (top_ring_start + (i + 1) % n_radial) as u32;
        indices.push([top_center, a, b]);
    }

    (vertices, indices)
}

fn generate_canine(params: &ToothParams) -> (Vec<Point3<f64>>, Vec<[u32; 3]>) {
    let w = params.mesio_distal_width / 2.0;
    let d = params.bucco_lingual_width / 2.0;
    let h = params.crown_height;
    let cusp_h = params.cusp_height;

    let n_radial = 16;
    let n_height = 12;

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for j in 0..=n_height {
        let t = j as f64 / n_height as f64;
        let z = t * h;

        // Canine: tapers to a point with strong cusp
        let taper = 1.0 - t * t * 0.8;
        let rx = w * taper;
        let ry = d * taper;

        for i in 0..n_radial {
            let theta = std::f64::consts::TAU * i as f64 / n_radial as f64;
            let x = rx * theta.cos();
            let y = ry * theta.sin();
            // Add cusp elevation at the tip
            let cusp_offset = if t > 0.7 {
                cusp_h * ((t - 0.7) / 0.3).powi(2) * (1.0 - (theta - std::f64::consts::FRAC_PI_2).abs() / std::f64::consts::PI)
            } else {
                0.0
            };
            vertices.push(Point3::new(x, y, z + cusp_offset));
        }
    }

    for j in 0..n_height {
        for i in 0..n_radial {
            let a = j * n_radial + i;
            let b = j * n_radial + (i + 1) % n_radial;
            let c = (j + 1) * n_radial + i;
            let d = (j + 1) * n_radial + (i + 1) % n_radial;
            indices.push([a as u32, b as u32, c as u32]);
            indices.push([b as u32, d as u32, c as u32]);
        }
    }

    (vertices, indices)
}

fn generate_premolar(params: &ToothParams) -> (Vec<Point3<f64>>, Vec<[u32; 3]>) {
    generate_multicusp_tooth(params, 2)
}

fn generate_molar(params: &ToothParams) -> (Vec<Point3<f64>>, Vec<[u32; 3]>) {
    let tooth_type = ToothType::from_fdi(params.fdi_number);
    let cusps = match tooth_type {
        Some(ToothType::FirstMolar) => 4,
        Some(ToothType::SecondMolar) => 4,
        _ => 5, // Third molars often have 5 cusps
    };
    generate_multicusp_tooth(params, cusps)
}

fn generate_multicusp_tooth(
    params: &ToothParams,
    num_cusps: usize,
) -> (Vec<Point3<f64>>, Vec<[u32; 3]>) {
    let w = params.mesio_distal_width / 2.0;
    let d = params.bucco_lingual_width / 2.0;
    let h = params.crown_height;
    let cusp_h = params.cusp_height;
    let fissure_d = params.fissure_depth;

    let n_radial = 24;
    let n_height = 12;

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // Generate cusp positions
    let cusp_angles: Vec<f64> = (0..num_cusps)
        .map(|i| std::f64::consts::TAU * i as f64 / num_cusps as f64)
        .collect();

    for j in 0..=n_height {
        let t = j as f64 / n_height as f64;
        let z = t * h;

        // Body tapers slightly upward
        let body_taper = 1.0 - t * 0.2;
        let rx = w * body_taper;
        let ry = d * body_taper;

        for i in 0..n_radial {
            let theta = std::f64::consts::TAU * i as f64 / n_radial as f64;
            let base_x = rx * theta.cos();
            let base_y = ry * theta.sin();

            // Add cusp modulation in the upper portion
            let cusp_offset = if t > 0.6 {
                let cusp_factor = (t - 0.6) / 0.4;
                let mut max_cusp = 0.0f64;
                let mut fissure = 1.0f64;
                for &ca in &cusp_angles {
                    let dist = angle_distance(theta, ca);
                    let cusp_influence = (-dist * dist * 4.0).exp();
                    max_cusp = max_cusp.max(cusp_influence);
                    fissure = fissure.min(1.0 - cusp_influence * 0.5);
                }
                cusp_factor * (cusp_h * max_cusp - fissure_d * (1.0 - max_cusp) * fissure)
            } else {
                0.0
            };

            vertices.push(Point3::new(base_x, base_y, z + cusp_offset));
        }
    }

    for j in 0..n_height {
        for i in 0..n_radial {
            let a = j * n_radial + i;
            let b = j * n_radial + (i + 1) % n_radial;
            let c = (j + 1) * n_radial + i;
            let d_idx = (j + 1) * n_radial + (i + 1) % n_radial;
            indices.push([a as u32, b as u32, c as u32]);
            indices.push([b as u32, d_idx as u32, c as u32]);
        }
    }

    (vertices, indices)
}

fn angle_distance(a: f64, b: f64) -> f64 {
    let diff = (a - b).rem_euclid(std::f64::consts::TAU);
    if diff > std::f64::consts::PI { std::f64::consts::TAU - diff } else { diff }
}
