//! Enums for TlantiStudio Dental

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

/// Gender options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Gender {
    Male,
    Female,
    Other,
    PreferNotToSay,
}

/// ID Document types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum IdDocumentType {
    Ine,           // Mexico INE
    Passport,
    DriverLicense,
    Curp,          // Mexico CURP
    Rfc,           // Mexico RFC
    SocialSecurity,
    Other,
}

/// Appointment status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum AppointmentStatus {
    Scheduled,
    Confirmed,
    CheckedIn,
    InProgress,
    Completed,
    Cancelled,
    NoShow,
    Rescheduled,
}

/// Treatment status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum TreatmentStatus {
    Planned,
    InProgress,
    Completed,
    Cancelled,
    OnHold,
}

/// Treatment plan status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum TreatmentPlanStatus {
    Draft,
    Proposed,
    Approved,
    InProgress,
    Completed,
    Cancelled,
}

/// Tooth condition in odontogram
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ToothCondition {
    Healthy,
    Caries,
    Filling,
    Crown,
    Bridge,
    Implant,
    RootCanal,
    Extraction,
    Missing,
    Fractured,
    Mobility,
    Abscess,
    Sensitivity,
}

/// Tooth surfaces (for charting)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ToothSurface {
    Mesial,      // M
    Distal,      // D
    Occlusal,    // O (for posterior teeth)
    Incisal,     // I (for anterior teeth)
    Buccal,      // B
    Lingual,     // L
    Facial,      // F
    Palatal,     // P
}

/// Procedure categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ProcedureCategory {
    Diagnostic,
    Preventive,
    Restorative,
    Endodontic,
    Periodontic,
    Prosthodontic,
    OralSurgery,
    Orthodontic,
    Pediatric,
    Cosmetic,
    Implant,
    Emergency,
    Other,
}

/// Invoice status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum InvoiceStatus {
    Draft,
    Pending,
    PartiallyPaid,
    Paid,
    Overdue,
    Cancelled,
    Refunded,
}

/// Payment methods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum PaymentMethod {
    Cash,
    CreditCard,
    DebitCard,
    BankTransfer,
    Check,
    Insurance,
    Financing,
    Other,
}

/// Stock movement types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum StockMovementType {
    Purchase,      // IN: Purchase order received
    Return,        // IN: Customer return
    Adjustment,    // IN/OUT: Inventory adjustment
    Consumption,   // OUT: Used in treatment
    Sale,          // OUT: Sold to patient
    Expired,       // OUT: Expired items removed
    Transfer,      // IN/OUT: Transfer between locations
}

/// Product categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ProductCategory {
    Material,
    Instrument,
    Medication,
    Consumable,
    Equipment,
    Implant,
    Laboratory,
    Hygiene,
    Office,
    Other,
}

/// Product unit of measure
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ProductUnit {
    Unit,
    Box,
    Pack,
    Bottle,
    Tube,
    Syringe,
    Kit,
    Roll,
    Gram,
    Milliliter,
}

/// User roles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum UserRole {
    Admin,
    Doctor,
    Receptionist,
    Assistant,
    Hygienist,
    Accountant,
    LabTech,
    Manager,
}

/// Document types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum DocumentType {
    ConsentForm,
    MedicalHistory,
    Prescription,
    LabOrder,
    Referral,
    XrayReport,
    TreatmentPlan,
    Invoice,
    Receipt,
    Certificate,
    Other,
}

/// Clinical note types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ClinicalNoteType {
    Examination,
    Treatment,
    Progress,
    PostOp,
    Consultation,
    Emergency,
    FollowUp,
}

/// Reminder channels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ReminderChannel {
    Sms,
    Email,
    WhatsApp,
    Phone,
    Push,
}
