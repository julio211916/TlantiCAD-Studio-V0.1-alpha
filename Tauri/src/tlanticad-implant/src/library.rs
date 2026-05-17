//! Real implant catalog with manufacturer and connection type data

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Implant manufacturer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ImplantManufacturer {
    NobelBiocare,
    Straumann,
    Osstem,
    Zimmer,
    Dentsply,
    MegaGen,
    Neodent,
    Bicon,
    Ankylos,
}

/// Implant-abutment connection type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ImplantConnection {
    InternalHex,
    ExternalHex,
    InternalTriangle,
    Conical,
    Morse,
}

/// Implant body profile
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ImplantProfile {
    Regular,
    Narrow,
    Wide,
    TaperLock,
}

/// A specific implant SKU with all geometric and clinical parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplantDefinition {
    pub id: Uuid,
    pub manufacturer: ImplantManufacturer,
    pub name: String,
    pub sku: String,
    /// Body diameter in mm
    pub diameter: f64,
    /// Implant length in mm
    pub length: f64,
    pub connection: ImplantConnection,
    pub profile: ImplantProfile,
    /// Prosthetic platform diameter in mm
    pub platform_diameter: f64,
    /// Recommended insertion torque in Ncm
    pub torque_insertion: f64,
    /// Final seating torque in Ncm
    pub torque_final: f64,
}

impl ImplantDefinition {
    /// Nobel Active 4.3 × 10 mm — conical connection, regular platform
    pub fn nobel_active_4_3_10() -> Self {
        Self {
            id: Uuid::new_v4(),
            manufacturer: ImplantManufacturer::NobelBiocare,
            name: "Nobel Active".into(),
            sku: "NA-4310".into(),
            diameter: 4.3,
            length: 10.0,
            connection: ImplantConnection::Conical,
            profile: ImplantProfile::Regular,
            platform_diameter: 4.3,
            torque_insertion: 35.0,
            torque_final: 35.0,
        }
    }

    /// Nobel Active 4.3 × 13 mm — conical connection, regular platform
    pub fn nobel_active_4_3_13() -> Self {
        Self {
            id: Uuid::new_v4(),
            manufacturer: ImplantManufacturer::NobelBiocare,
            name: "Nobel Active".into(),
            sku: "NA-4313".into(),
            diameter: 4.3,
            length: 13.0,
            connection: ImplantConnection::Conical,
            profile: ImplantProfile::Regular,
            platform_diameter: 4.3,
            torque_insertion: 35.0,
            torque_final: 35.0,
        }
    }

    /// Straumann RC 4.1 × 10 mm — Morse taper connection
    pub fn straumann_rc_4_1_10() -> Self {
        Self {
            id: Uuid::new_v4(),
            manufacturer: ImplantManufacturer::Straumann,
            name: "Straumann BL RC".into(),
            sku: "SBL-RC-4110".into(),
            diameter: 4.1,
            length: 10.0,
            connection: ImplantConnection::Morse,
            profile: ImplantProfile::Regular,
            platform_diameter: 4.8,
            torque_insertion: 35.0,
            torque_final: 35.0,
        }
    }

    /// Straumann RC 4.1 × 12 mm — Morse taper connection
    pub fn straumann_rc_4_1_12() -> Self {
        Self {
            id: Uuid::new_v4(),
            manufacturer: ImplantManufacturer::Straumann,
            name: "Straumann BL RC".into(),
            sku: "SBL-RC-4112".into(),
            diameter: 4.1,
            length: 12.0,
            connection: ImplantConnection::Morse,
            profile: ImplantProfile::Regular,
            platform_diameter: 4.8,
            torque_insertion: 35.0,
            torque_final: 35.0,
        }
    }

    /// Osstem TS III 4.0 × 10 mm — internal hex connection
    pub fn osstem_ts3_4_0_10() -> Self {
        Self {
            id: Uuid::new_v4(),
            manufacturer: ImplantManufacturer::Osstem,
            name: "Osstem TS III".into(),
            sku: "OTS3-4010".into(),
            diameter: 4.0,
            length: 10.0,
            connection: ImplantConnection::InternalHex,
            profile: ImplantProfile::Regular,
            platform_diameter: 4.5,
            torque_insertion: 30.0,
            torque_final: 30.0,
        }
    }

    /// Return the complete built-in catalog
    pub fn full_catalog() -> Vec<ImplantDefinition> {
        vec![
            Self::nobel_active_4_3_10(),
            Self::nobel_active_4_3_13(),
            // Nobel Active 3.5 narrow
            Self {
                id: Uuid::new_v4(),
                manufacturer: ImplantManufacturer::NobelBiocare,
                name: "Nobel Active NP".into(),
                sku: "NA-3510".into(),
                diameter: 3.5,
                length: 10.0,
                connection: ImplantConnection::Conical,
                profile: ImplantProfile::Narrow,
                platform_diameter: 3.5,
                torque_insertion: 25.0,
                torque_final: 25.0,
            },
            Self::straumann_rc_4_1_10(),
            Self::straumann_rc_4_1_12(),
            // Straumann Wide 4.8
            Self {
                id: Uuid::new_v4(),
                manufacturer: ImplantManufacturer::Straumann,
                name: "Straumann BL WN".into(),
                sku: "SBL-WN-4810".into(),
                diameter: 4.8,
                length: 10.0,
                connection: ImplantConnection::Morse,
                profile: ImplantProfile::Wide,
                platform_diameter: 6.5,
                torque_insertion: 40.0,
                torque_final: 40.0,
            },
            Self::osstem_ts3_4_0_10(),
            // Osstem TS III 4.0 × 13
            Self {
                id: Uuid::new_v4(),
                manufacturer: ImplantManufacturer::Osstem,
                name: "Osstem TS III".into(),
                sku: "OTS3-4013".into(),
                diameter: 4.0,
                length: 13.0,
                connection: ImplantConnection::InternalHex,
                profile: ImplantProfile::Regular,
                platform_diameter: 4.5,
                torque_insertion: 30.0,
                torque_final: 30.0,
            },
            // Zimmer TSV 4.1 × 10
            Self {
                id: Uuid::new_v4(),
                manufacturer: ImplantManufacturer::Zimmer,
                name: "Zimmer TSV".into(),
                sku: "ZTS-4110".into(),
                diameter: 4.1,
                length: 10.0,
                connection: ImplantConnection::InternalTriangle,
                profile: ImplantProfile::TaperLock,
                platform_diameter: 4.5,
                torque_insertion: 35.0,
                torque_final: 35.0,
            },
            // MegaGen AnyRidge 4.0 × 10
            Self {
                id: Uuid::new_v4(),
                manufacturer: ImplantManufacturer::MegaGen,
                name: "MegaGen AnyRidge".into(),
                sku: "MAR-4010".into(),
                diameter: 4.0,
                length: 10.0,
                connection: ImplantConnection::Conical,
                profile: ImplantProfile::Regular,
                platform_diameter: 4.5,
                torque_insertion: 40.0,
                torque_final: 40.0,
            },
        ]
    }
}
