//! User domain model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::enums::UserRole;

/// User entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    
    /// Username for login
    pub username: String,
    
    /// Email address
    pub email: String,
    
    /// Password hash (never expose)
    #[serde(skip_serializing)]
    pub password_hash: String,
    
    /// PIN hash for quick login (6 digits)
    #[serde(skip_serializing)]
    pub pin_hash: Option<String>,
    
    /// First name
    pub first_name: String,
    
    /// Last name
    pub last_name: String,
    
    /// User role
    pub role: UserRole,
    
    /// Phone number
    pub phone: Option<String>,
    
    /// Professional specialty (for doctors)
    pub specialty: Option<String>,
    
    /// License/certificate number (for doctors)
    pub license_number: Option<String>,
    
    /// Professional ID (cedula profesional in Mexico)
    pub professional_id: Option<String>,
    
    /// Assigned clinic(s)
    pub clinic_ids: Vec<Uuid>,
    
    /// Profile photo URL
    pub photo_url: Option<String>,
    
    /// Signature image path (for documents)
    pub signature_path: Option<String>,
    
    /// Calendar color (for appointments)
    pub calendar_color: Option<String>,
    
    /// Is active
    pub active: bool,
    
    /// Email verified
    pub email_verified: bool,
    
    /// Last login time
    pub last_login: Option<DateTime<Utc>>,
    
    /// Two-factor authentication enabled
    pub two_factor_enabled: bool,
    
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(
        username: String,
        email: String,
        password_hash: String,
        first_name: String,
        last_name: String,
        role: UserRole,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            username,
            email,
            password_hash,
            pin_hash: None,
            first_name,
            last_name,
            role,
            phone: None,
            specialty: None,
            license_number: None,
            professional_id: None,
            clinic_ids: Vec::new(),
            photo_url: None,
            signature_path: None,
            calendar_color: None,
            active: true,
            email_verified: false,
            last_login: None,
            two_factor_enabled: false,
            created_at: now,
            updated_at: now,
        }
    }
    
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }
    
    pub fn is_doctor(&self) -> bool {
        self.role == UserRole::Doctor
    }
    
    pub fn is_admin(&self) -> bool {
        self.role == UserRole::Admin
    }
}

/// User without sensitive data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub full_name: String,
    pub role: UserRole,
    pub phone: Option<String>,
    pub specialty: Option<String>,
    pub license_number: Option<String>,
    pub photo_url: Option<String>,
    pub calendar_color: Option<String>,
    pub active: bool,
}

impl From<User> for UserProfile {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            first_name: user.first_name.clone(),
            last_name: user.last_name.clone(),
            full_name: format!("{} {}", user.first_name, user.last_name),
            role: user.role,
            phone: user.phone,
            specialty: user.specialty,
            license_number: user.license_number,
            photo_url: user.photo_url,
            calendar_color: user.calendar_color,
            active: user.active,
        }
    }
}

/// User list item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserListItem {
    pub id: Uuid,
    pub username: String,
    pub full_name: String,
    pub email: String,
    pub role: UserRole,
    pub specialty: Option<String>,
    pub active: bool,
    pub last_login: Option<DateTime<Utc>>,
}

/// Doctor list item (for appointment scheduling)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorListItem {
    pub id: Uuid,
    pub full_name: String,
    pub specialty: Option<String>,
    pub license_number: Option<String>,
    pub calendar_color: Option<String>,
}

/// Create user DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub role: UserRole,
    pub phone: Option<String>,
    pub specialty: Option<String>,
    pub license_number: Option<String>,
    pub professional_id: Option<String>,
    pub clinic_ids: Option<Vec<Uuid>>,
    pub calendar_color: Option<String>,
}

/// Update user DTO
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateUser {
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub specialty: Option<String>,
    pub license_number: Option<String>,
    pub professional_id: Option<String>,
    pub clinic_ids: Option<Vec<Uuid>>,
    pub photo_url: Option<String>,
    pub calendar_color: Option<String>,
    pub active: Option<bool>,
}

/// Change password DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangePassword {
    pub current_password: String,
    pub new_password: String,
}

/// User session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// User filters
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserFilters {
    pub query: Option<String>,
    pub role: Option<UserRole>,
    pub clinic_id: Option<Uuid>,
    pub active_only: Option<bool>,
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: Uuid,
    pub user_id: Uuid,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Option<Uuid>,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub ip_address: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl AuditLog {
    pub fn new(user_id: Uuid, action: String, entity_type: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            action,
            entity_type,
            entity_id: None,
            old_value: None,
            new_value: None,
            ip_address: None,
            timestamp: Utc::now(),
        }
    }
}

/// Permission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub module: String,
}

/// Role permissions mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RolePermissions {
    pub role: UserRole,
    pub permissions: Vec<String>,
}
