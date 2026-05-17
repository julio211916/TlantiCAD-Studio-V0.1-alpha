//! Odontogram Tauri commands
//! 
//! Commands for managing dental charts (odontograms and periodontograms)

use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

use dental_core::models::{Odontogram, OdontogramEntry, OdontogramHistory, SurfaceCondition, UpdateOdontogramEntry};
use dental_core::ToothCondition;
use dental_database::{OdontogramRepository, PeriodontogramRepository};

use crate::{CommandResult, DentalCommandError, DentalState};

/// Input for saving odontogram
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveOdontogramInput {
    pub patient_id: String,
    pub dentition_type: String,
    pub teeth: Vec<ToothRecordInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToothRecordInput {
    pub tooth_number: String,
    pub surfaces: Option<serde_json::Value>,
    pub general_condition: Option<String>,
    pub notes: Option<String>,
}

/// Periodontogram data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Periodontogram {
    pub patient_id: String,
    pub teeth: Vec<PeriodontogramEntry>,
    pub last_updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodontogramEntry {
    pub tooth_number: i32,
    pub pocket_depths: Vec<i32>,
    pub bleeding_points: Vec<bool>,
    pub furcation: Option<i32>,
    pub mobility: Option<i32>,
}

/// Get odontogram for a patient
#[tauri::command]
pub fn odontogram_get(
    state: State<'_, DentalState>,
    patient_id: String,
) -> CommandResult<Odontogram> {
    let uuid = Uuid::parse_str(&patient_id)
        .map_err(|_| DentalCommandError::Validation("Invalid patient ID".into()))?;

    let repo = OdontogramRepository::new(state.db.pool().clone());
    let odontogram = repo.get_odontogram(uuid)?;
    Ok(odontogram)
}

/// Save odontogram for a patient
#[tauri::command]
pub fn odontogram_save(
    state: State<'_, DentalState>,
    data: SaveOdontogramInput,
) -> CommandResult<Odontogram> {
    let patient_uuid = Uuid::parse_str(&data.patient_id)
        .map_err(|_| DentalCommandError::Validation("Invalid patient ID".into()))?;
    
    let user_id = state.get_current_user()
        .ok_or_else(|| DentalCommandError::PermissionDenied("Not logged in".into()))?;
    
    let repo = OdontogramRepository::new(state.db.pool().clone());
    let mut odontogram = Odontogram::new(patient_uuid);
    
    for tooth in data.teeth {
        let tooth_num: i32 = tooth.tooth_number.parse()
            .map_err(|_| DentalCommandError::Validation("Invalid tooth number".into()))?;
        
        let condition = tooth.general_condition
            .as_ref()
            .map(|c| match c.as_str() {
                "healthy" => ToothCondition::Healthy,
                "caries" | "cavity" => ToothCondition::Caries,
                "filling" | "filled" => ToothCondition::Filling,
                "crown" => ToothCondition::Crown,
                "missing" => ToothCondition::Missing,
                "extraction" => ToothCondition::Extraction,
                "root_canal" => ToothCondition::RootCanal,
                "implant" => ToothCondition::Implant,
                "bridge" => ToothCondition::Bridge,
                _ => ToothCondition::Healthy,
            })
            .unwrap_or(ToothCondition::Healthy);
        
        let mut entry = OdontogramEntry::new(patient_uuid, tooth_num, user_id);
        entry.primary_condition = condition;
        entry.notes = tooth.notes;

        if let Some(surfaces) = tooth.surfaces {
            let parsed: Result<Vec<SurfaceCondition>, _> = serde_json::from_value(surfaces);
            if let Ok(surface_conditions) = parsed {
                entry.surface_conditions = surface_conditions;
            }
        }

        let saved = repo.upsert_entry(entry, None)?;
        odontogram.entries.push(saved);
    }
    
    odontogram.last_updated = Utc::now();

    Ok(odontogram)
}

/// Get odontogram history for a patient
#[tauri::command]
pub fn odontogram_history(
    state: State<'_, DentalState>,
    patient_id: String,
) -> CommandResult<Vec<OdontogramHistory>> {
    let _uuid = Uuid::parse_str(&patient_id)
        .map_err(|_| DentalCommandError::Validation("Invalid patient ID".into()))?;

    let repo = OdontogramRepository::new(state.db.pool().clone());
    let history = repo.list_history(Uuid::parse_str(&patient_id).unwrap_or_default())?;
    Ok(history)
}

/// Update a single tooth in the odontogram
#[tauri::command]
pub fn odontogram_update_tooth(
    state: State<'_, DentalState>,
    patient_id: String,
    tooth_number: i32,
    update: UpdateOdontogramEntry,
) -> CommandResult<OdontogramEntry> {
    let patient_uuid = Uuid::parse_str(&patient_id)
        .map_err(|_| DentalCommandError::Validation("Invalid patient ID".into()))?;
    
    let user_id = state.get_current_user()
        .ok_or_else(|| DentalCommandError::PermissionDenied("Not logged in".into()))?;
    
    let repo = OdontogramRepository::new(state.db.pool().clone());
    let entry = repo.update_tooth(patient_uuid, tooth_number, update, user_id)?;
    Ok(entry)
}

// ============ Periodontogram Commands ============

/// Get periodontogram for a patient
#[tauri::command]
pub fn periodontogram_get(
    state: State<'_, DentalState>,
    patient_id: String,
) -> CommandResult<Periodontogram> {
    let _uuid = Uuid::parse_str(&patient_id)
        .map_err(|_| DentalCommandError::Validation("Invalid patient ID".into()))?;

    let repo = PeriodontogramRepository::new(state.db.pool().clone());
    let patient_uuid = Uuid::parse_str(&patient_id).unwrap_or_default();
    let data = repo.get_by_patient(patient_uuid)?;

    if let Some(json) = data {
        let parsed: Result<Periodontogram, _> = serde_json::from_value(json);
        if let Ok(periodontogram) = parsed {
            return Ok(periodontogram);
        }
    }

    Ok(Periodontogram {
        patient_id,
        teeth: Vec::new(),
        last_updated: Utc::now().to_rfc3339(),
    })
}

/// Save periodontogram for a patient
#[tauri::command]
pub fn periodontogram_save(
    state: State<'_, DentalState>,
    data: Periodontogram,
) -> CommandResult<Periodontogram> {
    let _uuid = Uuid::parse_str(&data.patient_id)
        .map_err(|_| DentalCommandError::Validation("Invalid patient ID".into()))?;
    
    let _user_id = state.get_current_user()
        .ok_or_else(|| DentalCommandError::PermissionDenied("Not logged in".into()))?;
    
    let repo = PeriodontogramRepository::new(state.db.pool().clone());
    let patient_uuid = Uuid::parse_str(&data.patient_id).unwrap_or_default();

    let json = serde_json::to_value(&data)
        .map_err(|e| DentalCommandError::Internal(e.to_string()))?;

    let saved = repo.save(patient_uuid, json, _user_id)?;
    let parsed: Periodontogram = serde_json::from_value(saved)
        .map_err(|e| DentalCommandError::Internal(e.to_string()))?;

    Ok(parsed)
}
