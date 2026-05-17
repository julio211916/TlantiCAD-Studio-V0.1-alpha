use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── Patient ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Patient {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub date_of_birth: Option<String>,
    pub patient_number: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub notes: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

impl Patient {
    pub fn new(first_name: String, last_name: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            first_name,
            last_name,
            date_of_birth: None,
            patient_number: None,
            phone: None,
            email: None,
            notes: None,
            created_at: Some(Utc::now()),
        }
    }

    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }
}

// ─── Dentist ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Dentist {
    pub id: String,
    pub name: String,
    pub clinic: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

impl Dentist {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            clinic: None,
            email: None,
            phone: None,
            city: None,
            country: None,
            created_at: Some(Utc::now()),
        }
    }
}

// ─── Case / Project ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum CaseStatus {
    New,
    Design,
    CamSent,
    Completed,
    Delivered,
    OnHold,
}

impl std::fmt::Display for CaseStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::New => write!(f, "new"),
            Self::Design => write!(f, "design"),
            Self::CamSent => write!(f, "cam_sent"),
            Self::Completed => write!(f, "completed"),
            Self::Delivered => write!(f, "delivered"),
            Self::OnHold => write!(f, "on_hold"),
        }
    }
}

impl std::str::FromStr for CaseStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "new" => Ok(Self::New),
            "design" => Ok(Self::Design),
            "cam_sent" => Ok(Self::CamSent),
            "completed" => Ok(Self::Completed),
            "delivered" => Ok(Self::Delivered),
            "on_hold" => Ok(Self::OnHold),
            other => Err(format!("Unknown case status: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Case {
    pub id: String,
    pub case_number: String,
    pub patient_id: String,
    pub dentist_id: String,
    pub work_type: String,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub modified_at: Option<DateTime<Utc>>,
}

impl Case {
    pub fn new(patient_id: String, dentist_id: String, work_type: String) -> Self {
        let id = Uuid::new_v4().to_string();
        let case_number = format!("CASE-{}", &id[..8].to_uppercase());
        Self {
            id,
            case_number,
            patient_id,
            dentist_id,
            work_type,
            status: CaseStatus::New.to_string(),
            notes: None,
            created_at: Some(Utc::now()),
            modified_at: Some(Utc::now()),
        }
    }
}

// ─── Scan ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Scan {
    pub id: String,
    pub case_id: String,
    pub scan_type: String,  // "upper", "lower", "antagonist", "implant_scan"
    pub file_path: String,
    pub file_hash: Option<String>,
    pub transformation: Option<String>, // JSON 4x4 matrix
    pub is_visible: bool,
    pub import_date: Option<DateTime<Utc>>,
}

impl Scan {
    pub fn new(case_id: String, scan_type: String, file_path: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            case_id,
            scan_type,
            file_path,
            file_hash: None,
            transformation: None,
            is_visible: true,
            import_date: Some(Utc::now()),
        }
    }
}

// ─── Design ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Design {
    pub id: String,
    pub case_id: String,
    pub name: String,
    pub design_type: String,  // "crown", "abutment", "bridge", "bar", "splint"
    pub tooth_number: Option<i64>,
    pub mesh_file_path: Option<String>,
    pub parameters: Option<String>, // JSON
    pub created_at: Option<DateTime<Utc>>,
    pub modified_at: Option<DateTime<Utc>>,
}

impl Design {
    pub fn new(case_id: String, name: String, design_type: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            case_id,
            name,
            design_type,
            tooth_number: None,
            mesh_file_path: None,
            parameters: None,
            created_at: Some(Utc::now()),
            modified_at: Some(Utc::now()),
        }
    }
}

// ─── Material ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Material {
    pub id: String,
    pub name: String,
    pub material_type: String,  // "zirconia", "pmma", "titanium", "cobalt_chrome", "emax"
    pub manufacturer: Option<String>,
    pub available_shades: Option<String>, // JSON array e.g. ["A1","A2","A3","B1"]
    pub is_active: bool,
}
