//! Authentication commands for TlantiStudio
//! Handles onboarding, login with PIN, and user management

use dental_core::enums::UserRole;
use dental_core::models::UserProfile;
use dental_database::rusqlite;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use tauri::State;
use uuid::Uuid;
use chrono::Utc;

use crate::{DentalCommandResult, DentalState};

/// Check if system has been initialized (admin exists)
#[tauri::command]
pub async fn check_system_initialized(
    state: State<'_, DentalState>,
) -> DentalCommandResult<bool> {
    let conn = state.db.get_connection().map_err(|e| crate::DentalCommandError::DatabaseError(e.to_string()))?;
    
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM users WHERE role = 'Admin'",
        [],
        |row| row.get(0),
    ).unwrap_or(0);
    
    Ok(count > 0)
}

/// Onboarding data from frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingData {
    pub clinic_name: String,
    pub owner_name: String,
    pub pin: String,
}

/// Complete system onboarding - creates admin user and clinic
#[tauri::command]
pub async fn complete_onboarding(
    data: OnboardingData,
    state: State<'_, DentalState>,
) -> DentalCommandResult<UserProfile> {
    // Validate PIN is 6 digits
    if data.pin.len() != 6 || !data.pin.chars().all(|c| c.is_ascii_digit()) {
        return Err(crate::DentalCommandError::ValidationError(
            "PIN must be exactly 6 digits".to_string()
        ));
    }
    
    let conn = state.db.get_connection().map_err(|e| crate::DentalCommandError::DatabaseError(e.to_string()))?;
    
    // Check if admin already exists
    let admin_exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM users WHERE role = 'Admin'",
        [],
        |row| row.get(0),
    ).unwrap_or(0);
    
    if admin_exists > 0 {
        return Err(crate::DentalCommandError::ValidationError(
            "System already initialized".to_string()
        ));
    }
    
    // Hash the PIN
    let pin_hash = hash_pin(&data.pin);
    
    // Split owner name into first/last
    let name_parts: Vec<&str> = data.owner_name.splitn(2, ' ').collect();
    let first_name = name_parts.get(0).unwrap_or(&"Admin").to_string();
    let last_name = name_parts.get(1).unwrap_or(&"").to_string();
    
    let now = Utc::now();
    let user_id = Uuid::new_v4();
    let clinic_id = Uuid::new_v4();
    
    // Create clinic first
    conn.execute(
        "INSERT INTO clinics (id, name, address, city, state, postal_code, country, phone, timezone, currency, default_tax_rate, is_main, active, created_at, updated_at)
         VALUES (?1, ?2, '', '', '', '', 'Mexico', '', 'America/Mexico_City', 'MXN', 16.0, 1, 1, ?3, ?3)",
        rusqlite::params![
            clinic_id.to_string(),
            data.clinic_name,
            now.to_rfc3339(),
        ],
    ).map_err(|e| crate::DentalCommandError::DatabaseError(e.to_string()))?;
    
    // Create admin user
    conn.execute(
        "INSERT INTO users (id, username, email, password_hash, pin_hash, first_name, last_name, role, active, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 1, ?9, ?9)",
        rusqlite::params![
            user_id.to_string(),
            "admin",
            format!("admin@{}.local", data.clinic_name.to_lowercase().replace(' ', "")),
            "", // No password, using PIN
            pin_hash,
            first_name,
            last_name,
            "Admin",
            now.to_rfc3339(),
        ],
    ).map_err(|e| crate::DentalCommandError::DatabaseError(e.to_string()))?;
    
    // Return user profile
    Ok(UserProfile {
        id: user_id,
        username: "admin".to_string(),
        email: format!("admin@{}.local", data.clinic_name.to_lowercase().replace(' ', "")),
        first_name: first_name.clone(),
        last_name: last_name.clone(),
        full_name: format!("{} {}", first_name, last_name),
        role: UserRole::Admin,
        phone: None,
        specialty: None,
        license_number: None,
        photo_url: None,
        calendar_color: None,
        active: true,
    })
}

/// Login response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub user: UserProfile,
    pub clinic_name: String,
}

/// Get available users for login
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginUser {
    pub id: String,
    pub name: String,
    pub role: String,
}

/// Get list of users that can login
#[tauri::command]
pub async fn get_login_users(
    state: State<'_, DentalState>,
) -> DentalCommandResult<Vec<LoginUser>> {
    let conn = state.db.get_connection().map_err(|e| crate::DentalCommandError::DatabaseError(e.to_string()))?;
    
    let mut stmt = conn.prepare(
        "SELECT id, first_name, last_name, role FROM users WHERE active = 1 AND pin_hash IS NOT NULL ORDER BY role DESC, first_name"
    ).map_err(|e| crate::DentalCommandError::DatabaseError(e.to_string()))?;
    
    let users = stmt.query_map([], |row| {
        let first_name: String = row.get(1)?;
        let last_name: String = row.get(2)?;
        Ok(LoginUser {
            id: row.get(0)?,
            name: format!("{} {}", first_name, last_name).trim().to_string(),
            role: row.get(3)?,
        })
    }).map_err(|e| crate::DentalCommandError::DatabaseError(e.to_string()))?
    .filter_map(|r| r.ok())
    .collect();
    
    Ok(users)
}

/// Login with PIN
#[tauri::command]
pub async fn login_with_pin(
    user_id: String,
    pin: String,
    state: State<'_, DentalState>,
) -> DentalCommandResult<LoginResponse> {
    // Validate PIN format
    if pin.len() != 6 || !pin.chars().all(|c| c.is_ascii_digit()) {
        return Err(crate::DentalCommandError::ValidationError(
            "PIN must be exactly 6 digits".to_string()
        ));
    }
    
    let conn = state.db.get_connection().map_err(|e| crate::DentalCommandError::DatabaseError(e.to_string()))?;
    
    // Get user and verify PIN
    let pin_hash = hash_pin(&pin);
    
    let user_result = conn.query_row(
        "SELECT id, username, email, first_name, last_name, role, phone, specialty, license_number, photo_url, calendar_color, active, pin_hash
         FROM users WHERE id = ?1 AND active = 1",
        rusqlite::params![user_id],
        |row| {
            let stored_pin_hash: Option<String> = row.get(12)?;
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, Option<String>>(9)?,
                row.get::<_, Option<String>>(10)?,
                row.get::<_, bool>(11)?,
                stored_pin_hash,
            ))
        }
    ).map_err(|e| crate::DentalCommandError::DatabaseError(format!("User not found: {}", e)))?;
    
    let (id, username, email, first_name, last_name, role_str, phone, specialty, license_number, photo_url, calendar_color, active, stored_pin_hash) = user_result;
    
    // Verify PIN
    match stored_pin_hash {
        Some(stored) if stored == pin_hash => {},
        _ => {
            return Err(crate::DentalCommandError::ValidationError(
                "Invalid PIN".to_string()
            ));
        }
    }
    
    // Update last login
    let now = Utc::now();
    let _ = conn.execute(
        "UPDATE users SET last_login = ?1, updated_at = ?1 WHERE id = ?2",
        rusqlite::params![now.to_rfc3339(), user_id],
    );
    
    // Get clinic name
    let clinic_name: String = conn.query_row(
        "SELECT name FROM clinics WHERE is_main = 1 LIMIT 1",
        [],
        |row| row.get(0),
    ).unwrap_or_else(|_| "Clínica".to_string());
    
    let role = match role_str.as_str() {
        "Admin" => UserRole::Admin,
        "Doctor" => UserRole::Doctor,
        "Assistant" => UserRole::Assistant,
        "Receptionist" => UserRole::Receptionist,
        "LabTechnician" => UserRole::LabTech,
        _ => UserRole::Receptionist,
    };
    
    Ok(LoginResponse {
        user: UserProfile {
            id: Uuid::parse_str(&id).unwrap_or_else(|_| Uuid::new_v4()),
            username,
            email,
            first_name: first_name.clone(),
            last_name: last_name.clone(),
            full_name: format!("{} {}", first_name, last_name),
            role,
            phone,
            specialty,
            license_number,
            photo_url,
            calendar_color,
            active,
        },
        clinic_name,
    })
}

/// Create staff user data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateStaffData {
    pub first_name: String,
    pub last_name: String,
    pub pin: String,
    pub role: String, // "Admin" or "Staff"
}

/// Create a new staff user (admin only)
#[tauri::command]
pub async fn create_staff_user(
    data: CreateStaffData,
    current_user_id: String,
    state: State<'_, DentalState>,
) -> DentalCommandResult<UserProfile> {
    // Validate PIN
    if data.pin.len() != 6 || !data.pin.chars().all(|c| c.is_ascii_digit()) {
        return Err(crate::DentalCommandError::ValidationError(
            "PIN must be exactly 6 digits".to_string()
        ));
    }
    
    let conn = state.db.get_connection().map_err(|e| crate::DentalCommandError::DatabaseError(e.to_string()))?;
    
    // Verify current user is admin
    let is_admin: bool = conn.query_row(
        "SELECT role = 'Admin' FROM users WHERE id = ?1 AND active = 1",
        rusqlite::params![current_user_id],
        |row| row.get(0),
    ).unwrap_or(false);
    
    if !is_admin {
        return Err(crate::DentalCommandError::ValidationError(
            "Only administrators can create users".to_string()
        ));
    }
    
    let pin_hash = hash_pin(&data.pin);
    let now = Utc::now();
    let user_id = Uuid::new_v4();
    
    // Determine role - staff users get Receptionist role
    let role = if data.role == "Admin" { "Admin" } else { "Receptionist" };
    let username = format!("{}_{}", data.first_name.to_lowercase(), user_id.to_string()[..8].to_string());
    
    conn.execute(
        "INSERT INTO users (id, username, email, password_hash, pin_hash, first_name, last_name, role, active, created_at, updated_at)
         VALUES (?1, ?2, ?3, '', ?4, ?5, ?6, ?7, 1, ?8, ?8)",
        rusqlite::params![
            user_id.to_string(),
            username,
            format!("{}@local", username),
            pin_hash,
            data.first_name,
            data.last_name,
            role,
            now.to_rfc3339(),
        ],
    ).map_err(|e| crate::DentalCommandError::DatabaseError(e.to_string()))?;
    
    Ok(UserProfile {
        id: user_id,
        username: username.clone(),
        email: format!("{}@local", username),
        first_name: data.first_name.clone(),
        last_name: data.last_name.clone(),
        full_name: format!("{} {}", data.first_name, data.last_name),
        role: if role == "Admin" { UserRole::Admin } else { UserRole::Receptionist },
        phone: None,
        specialty: None,
        license_number: None,
        photo_url: None,
        calendar_color: None,
        active: true,
    })
}

/// Get current logged in user info
#[tauri::command]
pub async fn get_current_user(
    user_id: String,
    state: State<'_, DentalState>,
) -> DentalCommandResult<Option<UserProfile>> {
    let conn = state.db.get_connection().map_err(|e| crate::DentalCommandError::DatabaseError(e.to_string()))?;
    
    let result = conn.query_row(
        "SELECT id, username, email, first_name, last_name, role, phone, specialty, license_number, photo_url, calendar_color, active
         FROM users WHERE id = ?1 AND active = 1",
        rusqlite::params![user_id],
        |row| {
            let role_str: String = row.get(5)?;
            let role = match role_str.as_str() {
                "Admin" => UserRole::Admin,
                "Doctor" => UserRole::Doctor,
                "Assistant" => UserRole::Assistant,
                "Receptionist" => UserRole::Receptionist,
                "LabTechnician" => UserRole::LabTech,
                _ => UserRole::Receptionist,
            };
            
            let first_name: String = row.get(3)?;
            let last_name: String = row.get(4)?;
            
            Ok(UserProfile {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_else(|_| Uuid::new_v4()),
                username: row.get(1)?,
                email: row.get(2)?,
                first_name: first_name.clone(),
                last_name: last_name.clone(),
                full_name: format!("{} {}", first_name, last_name),
                role,
                phone: row.get(6)?,
                specialty: row.get(7)?,
                license_number: row.get(8)?,
                photo_url: row.get(9)?,
                calendar_color: row.get(10)?,
                active: row.get(11)?,
            })
        }
    );
    
    Ok(result.ok())
}

/// Hash a PIN using SHA256
fn hash_pin(pin: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(pin.as_bytes());
    hasher.update(b"tlanti_studio_salt_2026"); // Salt
    format!("{:x}", hasher.finalize())
}
