//! Document repository

use chrono::{DateTime, Utc};
use rusqlite::params;
use uuid::Uuid;

use dental_core::models::{Document, DocumentTemplate, CreateDocument, CreateDocumentTemplate, DocumentListItem};
use dental_core::DocumentType;

use crate::{DbError, DbPool, DbResult};

/// Document repository
pub struct DocumentRepository {
    pool: DbPool,
}

impl DocumentRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
    
    /// Create a new document
    pub fn create(&self, data: CreateDocument, created_by: Uuid) -> DbResult<Document> {
        let conn = self.pool.get()?;
        let now = Utc::now();
        let id = Uuid::new_v4();
        
        conn.execute(
            r#"
            INSERT INTO documents (
                id, patient_id, template_id, appointment_id, document_type,
                title, content, patient_visible, notes, created_by, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            "#,
            params![
                id.to_string(),
                data.patient_id.to_string(),
                data.template_id.map(|t| t.to_string()),
                data.appointment_id.map(|a| a.to_string()),
                data.document_type.to_string(),
                data.title,
                data.content,
                data.patient_visible.unwrap_or(false),
                data.notes,
                created_by.to_string(),
                now.to_rfc3339(),
                now.to_rfc3339(),
            ],
        )?;
        
        self.find_by_id(id)
    }
    
    /// Find document by ID
    pub fn find_by_id(&self, id: Uuid) -> DbResult<Document> {
        let conn = self.pool.get()?;
        
        conn.query_row(
            r#"
            SELECT id, patient_id, template_id, appointment_id, document_type,
                   title, content, file_path, mime_type, file_size,
                   signed, signature_path, signature_date, signed_by,
                   patient_visible, notes, created_by, created_at, updated_at
            FROM documents
            WHERE id = ?1
            "#,
            [id.to_string()],
            |row| self.map_row(row),
        ).map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => DbError::NotFound(format!("Document {}", id)),
            _ => DbError::QueryError(e.to_string()),
        })
    }
    
    /// Sign document
    pub fn sign(&self, id: Uuid, signature_path: String, signed_by: String) -> DbResult<()> {
        let conn = self.pool.get()?;
        let now = Utc::now();
        
        conn.execute(
            r#"
            UPDATE documents SET
                signed = 1, signature_path = ?2, signature_date = ?3, signed_by = ?4, updated_at = ?5
            WHERE id = ?1
            "#,
            params![id.to_string(), signature_path, now.to_rfc3339(), signed_by, now.to_rfc3339()],
        )?;
        
        Ok(())
    }
    
    /// List documents by patient
    pub fn list_by_patient(&self, patient_id: Uuid) -> DbResult<Vec<DocumentListItem>> {
        let conn = self.pool.get()?;
        
        let mut stmt = conn.prepare(
            r#"
            SELECT d.id, d.patient_id, p.first_name || ' ' || p.last_name,
                   d.document_type, d.title, d.signed, d.created_at
            FROM documents d
            JOIN patients p ON p.id = d.patient_id
            WHERE d.patient_id = ?1
            ORDER BY d.created_at DESC
            "#
        )?;
        
        let rows = stmt.query_map([patient_id.to_string()], |row| {
            let id: String = row.get(0)?;
            let pid: String = row.get(1)?;
            let doc_type_str: String = row.get(3)?;
            let created_at_str: String = row.get(6)?;
            
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            
            Ok(DocumentListItem {
                id: Uuid::parse_str(&id).unwrap_or_default(),
                patient_id: Uuid::parse_str(&pid).unwrap_or_default(),
                patient_name: row.get(2)?,
                document_type: doc_type_str.parse().unwrap_or(DocumentType::Other),
                title: row.get(4)?,
                signed: row.get(5)?,
                created_at,
            })
        })?;
        
        Ok(rows.filter_map(|r| r.ok()).collect())
    }
    
    /// Create a document template
    pub fn create_template(&self, data: CreateDocumentTemplate, created_by: Uuid) -> DbResult<DocumentTemplate> {
        let conn = self.pool.get()?;
        let now = Utc::now();
        let id = Uuid::new_v4();
        
        let variables_json = data.variables.map(|v| serde_json::to_string(&v).ok()).flatten();
        
        conn.execute(
            r#"
            INSERT INTO document_templates (
                id, name, description, document_type, category, content,
                variables, header, footer, styles, active, is_system,
                created_by, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
            "#,
            params![
                id.to_string(),
                data.name,
                data.description,
                data.document_type.to_string(),
                data.category,
                data.content,
                variables_json,
                data.header,
                data.footer,
                data.styles,
                true,
                false,
                created_by.to_string(),
                now.to_rfc3339(),
                now.to_rfc3339(),
            ],
        )?;
        
        self.find_template_by_id(id)
    }
    
    /// Find template by ID
    pub fn find_template_by_id(&self, id: Uuid) -> DbResult<DocumentTemplate> {
        let conn = self.pool.get()?;
        
        conn.query_row(
            "SELECT * FROM document_templates WHERE id = ?1",
            [id.to_string()],
            |row| self.map_template_row(row),
        ).map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => DbError::NotFound(format!("Template {}", id)),
            _ => DbError::QueryError(e.to_string()),
        })
    }
    
    /// List templates by type
    pub fn list_templates(&self, document_type: Option<DocumentType>) -> DbResult<Vec<DocumentTemplate>> {
        let conn = self.pool.get()?;
        
        let sql = match document_type {
            Some(dt) => format!(
                "SELECT * FROM document_templates WHERE active = 1 AND document_type = '{}' ORDER BY name",
                dt.to_string()
            ),
            None => "SELECT * FROM document_templates WHERE active = 1 ORDER BY name".to_string(),
        };
        
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([], |row| self.map_template_row(row))?;
        
        Ok(rows.filter_map(|r| r.ok()).collect())
    }
    
    fn map_row(&self, row: &rusqlite::Row) -> Result<Document, rusqlite::Error> {
        let id: String = row.get(0)?;
        let patient_id: String = row.get(1)?;
        let doc_type_str: String = row.get(4)?;
        let created_by: String = row.get(16)?;
        let created_at_str: String = row.get(17)?;
        let updated_at_str: String = row.get(18)?;
        
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        
        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        
        Ok(Document {
            id: Uuid::parse_str(&id).unwrap_or_default(),
            patient_id: Uuid::parse_str(&patient_id).unwrap_or_default(),
            template_id: row.get::<_, Option<String>>(2)?.and_then(|s| Uuid::parse_str(&s).ok()),
            appointment_id: row.get::<_, Option<String>>(3)?.and_then(|s| Uuid::parse_str(&s).ok()),
            document_type: doc_type_str.parse().unwrap_or(DocumentType::Other),
            title: row.get(5)?,
            content: row.get(6)?,
            file_path: row.get(7)?,
            mime_type: row.get(8)?,
            file_size: row.get(9)?,
            signed: row.get(10)?,
            signature_path: row.get(11)?,
            signature_date: None,
            signed_by: row.get(13)?,
            patient_visible: row.get(14)?,
            notes: row.get(15)?,
            created_by: Uuid::parse_str(&created_by).unwrap_or_default(),
            created_at,
            updated_at,
        })
    }
    
    fn map_template_row(&self, row: &rusqlite::Row) -> Result<DocumentTemplate, rusqlite::Error> {
        let id: String = row.get(0)?;
        let doc_type_str: String = row.get(3)?;
        let created_by: String = row.get(12)?;
        let created_at_str: String = row.get(13)?;
        let updated_at_str: String = row.get(14)?;
        
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        
        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        
        Ok(DocumentTemplate {
            id: Uuid::parse_str(&id).unwrap_or_default(),
            name: row.get(1)?,
            description: row.get(2)?,
            document_type: doc_type_str.parse().unwrap_or(DocumentType::Other),
            category: row.get(4)?,
            content: row.get(5)?,
            variables: Vec::new(), // Parse from JSON if needed
            header: row.get(7)?,
            footer: row.get(8)?,
            styles: row.get(9)?,
            active: row.get(10)?,
            is_system: row.get(11)?,
            created_by: Uuid::parse_str(&created_by).unwrap_or_default(),
            created_at,
            updated_at,
        })
    }
}
