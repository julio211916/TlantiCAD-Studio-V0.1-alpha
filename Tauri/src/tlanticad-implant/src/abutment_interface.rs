//! Abutment interface geometry for implant-abutment connections

use crate::library::{ImplantConnection, ImplantDefinition};

/// Get the platform height above the implant shoulder in mm.
pub fn get_platform_height(implant: &ImplantDefinition) -> f64 {
    match implant.connection {
        ImplantConnection::Conical => 0.5,
        ImplantConnection::Morse => 0.8,
        ImplantConnection::InternalHex => 0.7,
        ImplantConnection::ExternalHex => 0.0,
        ImplantConnection::InternalTriangle => 0.6,
    }
}

/// Get the hex or polygon driving feature width in mm.
pub fn get_hex_width(implant: &ImplantDefinition) -> f64 {
    // Approximate hex/polygon widths per diameter class
    let d = implant.diameter;
    match implant.connection {
        ImplantConnection::ExternalHex => {
            if d <= 3.5 { 2.4 } else if d <= 4.1 { 2.7 } else { 3.0 }
        }
        ImplantConnection::InternalHex => {
            if d <= 3.5 { 1.8 } else if d <= 4.1 { 2.0 } else { 2.4 }
        }
        ImplantConnection::InternalTriangle => {
            if d <= 4.1 { 2.5 } else { 3.0 }
        }
        ImplantConnection::Conical | ImplantConnection::Morse => {
            // Conical/Morse: no hex — report internal taper diameter
            d * 0.6
        }
    }
}

/// Return a list of compatible abutment SKU identifiers for the given implant.
pub fn compatible_abutments(implant: &ImplantDefinition) -> Vec<String> {
    let platform = format!("{:.1}", implant.platform_diameter);
    let skus: Vec<String> = match implant.connection {
        ImplantConnection::Conical => vec![
            format!("ABT-CON-ST-{}", platform),
            format!("ABT-CON-ANG15-{}", platform),
            format!("ABT-CON-ANG25-{}", platform),
            format!("ABT-CON-MUA-{}", platform),
            format!("ABT-CON-TI-{}", platform),
        ],
        ImplantConnection::Morse => vec![
            format!("ABT-MORSE-ST-{}", platform),
            format!("ABT-MORSE-ANG17-{}", platform),
            format!("ABT-MORSE-ANG30-{}", platform),
            format!("ABT-MORSE-MUA-{}", platform),
        ],
        ImplantConnection::InternalHex => vec![
            format!("ABT-IH-ST-{}", platform),
            format!("ABT-IH-ANG15-{}", platform),
            format!("ABT-IH-ANG25-{}", platform),
            format!("ABT-IH-BALL-{}", platform),
            format!("ABT-IH-LOCATOR-{}", platform),
        ],
        ImplantConnection::ExternalHex => vec![
            format!("ABT-EH-ST-{}", platform),
            format!("ABT-EH-ANG15-{}", platform),
            format!("ABT-EH-ANG30-{}", platform),
            format!("ABT-EH-MUA-{}", platform),
        ],
        ImplantConnection::InternalTriangle => vec![
            format!("ABT-IT-ST-{}", platform),
            format!("ABT-IT-ANG15-{}", platform),
            format!("ABT-IT-ANG25-{}", platform),
        ],
    };
    skus
}
