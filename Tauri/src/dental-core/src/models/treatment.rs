//! Treatment domain models

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::enums::{TreatmentStatus, TreatmentPlanStatus, ToothSurface};

/// Individual treatment/procedure performed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Treatment {
    pub id: Uuid,
    
    /// Reference to patient
    pub patient_id: Uuid,
    
    /// Reference to appointment (when performed)
    pub appointment_id: Option<Uuid>,
    
    /// Reference to treatment plan
    pub treatment_plan_id: Option<Uuid>,
    
    /// Reference to procedure catalog
    pub procedure_id: Uuid,
    
    /// Doctor who performed the treatment
    pub doctor_id: Uuid,
    
    /// Tooth number (FDI notation: 11-48)
    pub tooth_number: Option<i32>,
    
    /// Affected surfaces
    pub surfaces: Option<Vec<ToothSurface>>,
    
    /// Quadrant (1-4) for quadrant-based treatments
    pub quadrant: Option<i32>,
    
    /// Status of the treatment
    pub status: TreatmentStatus,
    
    /// Price charged
    pub price: Decimal,
    
    /// Discount applied
    pub discount: Decimal,
    
    /// Final price after discount
    pub final_price: Decimal,
    
    /// Treatment notes
    pub notes: Option<String>,
    
    /// Date planned
    pub planned_date: Option<DateTime<Utc>>,
    
    /// Date completed
    pub completed_at: Option<DateTime<Utc>>,
    
    /// Warranty expiration (for restorative work)
    pub warranty_until: Option<DateTime<Utc>>,
    
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Treatment {
    pub fn new(
        patient_id: Uuid,
        procedure_id: Uuid,
        doctor_id: Uuid,
        price: Decimal,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            patient_id,
            appointment_id: None,
            treatment_plan_id: None,
            procedure_id,
            doctor_id,
            tooth_number: None,
            surfaces: None,
            quadrant: None,
            status: TreatmentStatus::Planned,
            price,
            discount: Decimal::ZERO,
            final_price: price,
            notes: None,
            planned_date: None,
            completed_at: None,
            warranty_until: None,
            created_at: now,
            updated_at: now,
        }
    }
    
    pub fn calculate_final_price(&mut self) {
        self.final_price = self.price - self.discount;
    }
}

/// Treatment plan - group of planned treatments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreatmentPlan {
    pub id: Uuid,
    
    /// Reference to patient
    pub patient_id: Uuid,
    
    /// Doctor who created the plan
    pub created_by: Uuid,
    
    /// Plan name/title
    pub name: String,
    
    /// Description
    pub description: Option<String>,
    
    /// Status
    pub status: TreatmentPlanStatus,
    
    /// Total estimated cost
    pub total_estimated: Decimal,
    
    /// Discount on total
    pub total_discount: Decimal,
    
    /// Final total after discount
    pub total_final: Decimal,
    
    /// Patient approved date
    pub approved_at: Option<DateTime<Utc>>,
    
    /// Patient signature file path
    pub signature_path: Option<String>,
    
    /// Notes
    pub notes: Option<String>,
    
    /// Valid until date
    pub valid_until: Option<DateTime<Utc>>,
    
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TreatmentPlan {
    pub fn new(patient_id: Uuid, name: String, created_by: Uuid) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            patient_id,
            created_by,
            name,
            description: None,
            status: TreatmentPlanStatus::Draft,
            total_estimated: Decimal::ZERO,
            total_discount: Decimal::ZERO,
            total_final: Decimal::ZERO,
            approved_at: None,
            signature_path: None,
            notes: None,
            valid_until: None,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Treatment plan with items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreatmentPlanDetails {
    pub plan: TreatmentPlan,
    pub items: Vec<TreatmentPlanItem>,
    pub patient_name: String,
    pub doctor_name: String,
}

/// Item in a treatment plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreatmentPlanItem {
    pub id: Uuid,
    pub treatment_plan_id: Uuid,
    pub procedure_id: Uuid,
    pub procedure_name: String,
    pub tooth_number: Option<i32>,
    pub surfaces: Option<Vec<ToothSurface>>,
    pub quadrant: Option<i32>,
    pub quantity: i32,
    pub unit_price: Decimal,
    pub discount: Decimal,
    pub total: Decimal,
    pub priority: i32, // Order of treatment
    pub notes: Option<String>,
    pub status: TreatmentStatus,
}

/// Create treatment DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTreatment {
    pub patient_id: Uuid,
    pub appointment_id: Option<Uuid>,
    pub treatment_plan_id: Option<Uuid>,
    pub procedure_id: Uuid,
    pub doctor_id: Uuid,
    pub tooth_number: Option<i32>,
    pub surfaces: Option<Vec<ToothSurface>>,
    pub quadrant: Option<i32>,
    pub price: Decimal,
    pub discount: Option<Decimal>,
    pub notes: Option<String>,
    pub planned_date: Option<DateTime<Utc>>,
}

/// Update treatment DTO
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateTreatment {
    pub status: Option<TreatmentStatus>,
    pub tooth_number: Option<i32>,
    pub surfaces: Option<Vec<ToothSurface>>,
    pub price: Option<Decimal>,
    pub discount: Option<Decimal>,
    pub notes: Option<String>,
    pub planned_date: Option<DateTime<Utc>>,
}

/// Create treatment plan DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTreatmentPlan {
    pub patient_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub items: Vec<CreateTreatmentPlanItem>,
    pub discount: Option<Decimal>,
    pub notes: Option<String>,
    pub valid_until: Option<DateTime<Utc>>,
}

/// Create treatment plan item DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTreatmentPlanItem {
    pub procedure_id: Uuid,
    pub tooth_number: Option<i32>,
    pub surfaces: Option<Vec<ToothSurface>>,
    pub quadrant: Option<i32>,
    pub quantity: i32,
    pub unit_price: Decimal,
    pub discount: Option<Decimal>,
    pub priority: i32,
    pub notes: Option<String>,
}

/// Treatment with procedure details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreatmentWithDetails {
    pub treatment: Treatment,
    pub procedure_name: String,
    pub procedure_code: String,
    pub patient_name: String,
    pub doctor_name: String,
}

/// Treatment filters
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TreatmentFilters {
    pub patient_id: Option<Uuid>,
    pub doctor_id: Option<Uuid>,
    pub appointment_id: Option<Uuid>,
    pub treatment_plan_id: Option<Uuid>,
    pub status: Option<Vec<TreatmentStatus>>,
    pub tooth_number: Option<i32>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
}
