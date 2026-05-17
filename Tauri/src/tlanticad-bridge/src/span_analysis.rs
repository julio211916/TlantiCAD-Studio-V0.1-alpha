//! Bridge span analysis: allowable span and deflection estimation

/// Maximum allowable bridge span in mm for a given material and connector cross-section.
///
/// Based on simplified beam theory and published clinical guidelines.
pub fn max_allowed_span(material: &str, connector_cross_section: f64) -> f64 {
    // Approximate flexural stiffness coefficient (mm⁻¹) per material
    let stiffness = match material.to_lowercase().as_str() {
        s if s.contains("zirconia") => 200.0,
        s if s.contains("emax") || s.contains("lithium") => 120.0,
        s if s.contains("cobalt") || s.contains("chrome") || s.contains("metal") => 400.0,
        s if s.contains("titanium") => 300.0,
        s if s.contains("pmma") => 60.0,
        _ => 100.0,
    };
    // Span proportional to (stiffness × cross-section)^(1/3)
    let base = (stiffness * connector_cross_section).cbrt();
    base.min(50.0) // hard cap at 50 mm for any material
}

/// Estimate the midpoint deflection of a bridge under a typical occlusal load.
///
/// Uses a simple beam formula: δ = F·L³ / (48·E·I)
/// where I is approximated from `connector_cross_section` and E from `material`.
/// Returns deflection in µm.
pub fn deflection_estimate(span_mm: f64, load_n: f64, material: &str) -> f64 {
    // Young's modulus (GPa → N/mm²)
    let e_mpa: f64 = match material.to_lowercase().as_str() {
        s if s.contains("zirconia") => 200_000.0,
        s if s.contains("emax") || s.contains("lithium") => 95_000.0,
        s if s.contains("cobalt") || s.contains("chrome") => 220_000.0,
        s if s.contains("titanium") => 110_000.0,
        s if s.contains("pmma") => 3_000.0,
        _ => 70_000.0,
    };

    // Simplified moment of inertia assuming rectangular 3×3 mm cross-section
    let i_mm4 = (3.0_f64.powi(4)) / 12.0; // b·h³/12 for 3mm square

    if e_mpa <= 0.0 || i_mm4 <= 0.0 || span_mm <= 0.0 {
        return 0.0;
    }

    // δ = F·L³ / (48·E·I), result in mm, convert to µm
    let deflection_mm = (load_n * span_mm.powi(3)) / (48.0 * e_mpa * i_mm4);
    deflection_mm * 1000.0 // µm
}
