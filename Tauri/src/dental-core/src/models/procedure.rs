//! Procedure/Service catalog domain model

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::enums::ProcedureCategory;

/// Procedure/Service entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Procedure {
    pub id: Uuid,
    
    /// Procedure code (ADA code or custom)
    pub code: String,
    
    /// Procedure name
    pub name: String,
    
    /// Description
    pub description: Option<String>,
    
    /// Category
    pub category: ProcedureCategory,
    
    /// Default price
    pub default_price: Decimal,
    
    /// Minimum price (for discounting)
    pub min_price: Option<Decimal>,
    
    /// Duration in minutes
    pub duration_minutes: i32,
    
    /// Is per tooth
    pub per_tooth: bool,
    
    /// Is per quadrant
    pub per_quadrant: bool,
    
    /// Is per arch (upper/lower)
    pub per_arch: bool,
    
    /// Required products (JSON array of product IDs)
    pub required_products: Option<Vec<Uuid>>,
    
    /// Typical warranty period in months
    pub warranty_months: Option<i32>,
    
    /// Notes/instructions
    pub notes: Option<String>,
    
    /// Is active
    pub active: bool,
    
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Procedure {
    pub fn new(code: String, name: String, category: ProcedureCategory, default_price: Decimal) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            code,
            name,
            description: None,
            category,
            default_price,
            min_price: None,
            duration_minutes: 30,
            per_tooth: false,
            per_quadrant: false,
            per_arch: false,
            required_products: None,
            warranty_months: None,
            notes: None,
            active: true,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Procedure list item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureListItem {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub category: ProcedureCategory,
    pub default_price: Decimal,
    pub duration_minutes: i32,
    pub active: bool,
}

/// Create procedure DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProcedure {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub category: ProcedureCategory,
    pub default_price: Decimal,
    pub min_price: Option<Decimal>,
    pub duration_minutes: i32,
    pub per_tooth: Option<bool>,
    pub per_quadrant: Option<bool>,
    pub per_arch: Option<bool>,
    pub required_products: Option<Vec<Uuid>>,
    pub warranty_months: Option<i32>,
    pub notes: Option<String>,
}

/// Update procedure DTO
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateProcedure {
    pub code: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub category: Option<ProcedureCategory>,
    pub default_price: Option<Decimal>,
    pub min_price: Option<Decimal>,
    pub duration_minutes: Option<i32>,
    pub per_tooth: Option<bool>,
    pub per_quadrant: Option<bool>,
    pub per_arch: Option<bool>,
    pub required_products: Option<Vec<Uuid>>,
    pub warranty_months: Option<i32>,
    pub notes: Option<String>,
    pub active: Option<bool>,
}

/// Procedure filters
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProcedureFilters {
    pub query: Option<String>,
    pub code: Option<String>,
    pub category: Option<ProcedureCategory>,
    pub active_only: Option<bool>,
    pub price_min: Option<Decimal>,
    pub price_max: Option<Decimal>,
}

/// Procedure with usage stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureWithStats {
    pub procedure: Procedure,
    pub times_performed: i64,
    pub total_revenue: Decimal,
    pub last_performed: Option<DateTime<Utc>>,
}

/// Common dental procedures (seed data)
pub struct CommonProcedures;

impl CommonProcedures {
    pub fn diagnostic() -> Vec<(&'static str, &'static str, i32)> {
        vec![
            ("D0120", "Periodic Oral Evaluation", 15),
            ("D0140", "Limited Oral Evaluation", 15),
            ("D0150", "Comprehensive Oral Evaluation", 30),
            ("D0210", "Full Mouth X-rays", 20),
            ("D0220", "Periapical X-ray", 5),
            ("D0272", "Bitewing X-rays (2)", 10),
            ("D0274", "Bitewing X-rays (4)", 15),
            ("D0330", "Panoramic X-ray", 15),
        ]
    }
    
    pub fn preventive() -> Vec<(&'static str, &'static str, i32)> {
        vec![
            ("D1110", "Prophylaxis - Adult", 45),
            ("D1120", "Prophylaxis - Child", 30),
            ("D1206", "Topical Fluoride Varnish", 10),
            ("D1351", "Sealant - Per Tooth", 15),
        ]
    }
    
    pub fn restorative() -> Vec<(&'static str, &'static str, i32)> {
        vec![
            ("D2140", "Amalgam - 1 Surface", 30),
            ("D2150", "Amalgam - 2 Surfaces", 40),
            ("D2160", "Amalgam - 3 Surfaces", 50),
            ("D2330", "Composite - 1 Surface Anterior", 30),
            ("D2331", "Composite - 2 Surfaces Anterior", 40),
            ("D2391", "Composite - 1 Surface Posterior", 35),
            ("D2392", "Composite - 2 Surfaces Posterior", 45),
            ("D2393", "Composite - 3 Surfaces Posterior", 55),
            ("D2740", "Porcelain Crown", 90),
            ("D2750", "Metal-Porcelain Crown", 90),
            ("D2790", "Full Gold Crown", 90),
        ]
    }
    
    pub fn endodontic() -> Vec<(&'static str, &'static str, i32)> {
        vec![
            ("D3220", "Root Canal - Anterior", 60),
            ("D3230", "Root Canal - Premolar", 75),
            ("D3240", "Root Canal - Molar", 90),
            ("D3310", "Root Canal Treatment - Anterior", 60),
            ("D3320", "Root Canal Treatment - Premolar", 75),
            ("D3330", "Root Canal Treatment - Molar", 90),
        ]
    }
    
    pub fn periodontic() -> Vec<(&'static str, &'static str, i32)> {
        vec![
            ("D4341", "Scaling & Root Planing (Per Quadrant)", 45),
            ("D4342", "Scaling & Root Planing (1-3 Teeth)", 30),
            ("D4910", "Periodontal Maintenance", 45),
        ]
    }
    
    pub fn oral_surgery() -> Vec<(&'static str, &'static str, i32)> {
        vec![
            ("D7140", "Extraction - Simple", 20),
            ("D7210", "Extraction - Surgical", 30),
            ("D7220", "Impacted Tooth Removal - Soft Tissue", 45),
            ("D7230", "Impacted Tooth Removal - Partial Bony", 60),
            ("D7240", "Impacted Tooth Removal - Full Bony", 75),
        ]
    }
    
    pub fn prosthodontic() -> Vec<(&'static str, &'static str, i32)> {
        vec![
            ("D5110", "Complete Denture - Upper", 120),
            ("D5120", "Complete Denture - Lower", 120),
            ("D5211", "Partial Denture - Resin Upper", 90),
            ("D5212", "Partial Denture - Resin Lower", 90),
            ("D6240", "Pontic - Porcelain/High Noble Metal", 60),
        ]
    }
}
