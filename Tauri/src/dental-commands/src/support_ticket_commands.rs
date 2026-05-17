//! Support ticketing commands (SQLite-backed)

use chrono::{DateTime, Utc};
use dental_database::rusqlite::{self, params};
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

use crate::{DentalCommandError, DentalCommandResult, DentalState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportTicket {
    pub id: String,
    pub subject: String,
    pub description: String,
    pub status: String,
    pub priority: String,
    pub customer_name: String,
    pub customer_email: String,
    pub assigned_to: Option<String>,
    pub assigned_to_name: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub first_response_at: Option<String>,
    pub last_response_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportTicketMessage {
    pub id: String,
    pub ticket_id: String,
    pub body: String,
    pub author_type: String,
    pub author_id: Option<String>,
    pub author_name: String,
    pub created_at: String,
    pub is_internal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportTicketAgent {
    pub id: String,
    pub full_name: String,
    pub email: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CannedResponse {
    pub id: String,
    pub title: String,
    pub body: String,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewSupportTicketInput {
    pub customer_name: String,
    pub customer_email: String,
    pub subject: String,
    pub description: String,
    pub priority: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewSupportTicketMessageInput {
    pub ticket_id: String,
    pub body: String,
    pub author_type: String,
    pub author_id: Option<String>,
    pub author_name: String,
    pub is_internal: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportTicketFilters {
    pub status: Option<String>,
    pub priority: Option<String>,
    pub search: Option<String>,
}

const RATE_LIMIT_SECONDS: i64 = 60;

fn ensure_status(value: &str) -> DentalCommandResult<()> {
    match value {
        "open" | "in_progress" | "resolved" => Ok(()),
        _ => Err(DentalCommandError::ValidationError("Estado inválido".to_string())),
    }
}

fn ensure_priority(value: &str) -> DentalCommandResult<()> {
    match value {
        "low" | "medium" | "high" | "urgent" => Ok(()),
        _ => Err(DentalCommandError::ValidationError("Prioridad inválida".to_string())),
    }
}

fn can_transition_status(current: &str, next: &str) -> bool {
    matches!((current, next), ("open", "in_progress") | ("in_progress", "resolved"))
}

#[tauri::command]
pub fn support_ticket_create(
    state: State<'_, DentalState>,
    data: NewSupportTicketInput,
) -> DentalCommandResult<SupportTicket> {
    let customer_name = data.customer_name.trim().to_string();
    let customer_email = data.customer_email.trim().to_lowercase();
    let subject = data.subject.trim().to_string();
    let description = data.description.trim().to_string();
    let priority = data.priority.trim().to_string();

    if customer_name.len() < 2 {
        return Err(DentalCommandError::ValidationError(
            "El nombre del cliente es obligatorio".to_string(),
        ));
    }
    if !customer_email.contains('@') || !customer_email.contains('.') {
        return Err(DentalCommandError::ValidationError("Email inválido".to_string()));
    }
    if subject.len() < 6 {
        return Err(DentalCommandError::ValidationError(
            "El asunto debe tener al menos 6 caracteres".to_string(),
        ));
    }
    if description.len() < 10 {
        return Err(DentalCommandError::ValidationError(
            "La descripción debe tener al menos 10 caracteres".to_string(),
        ));
    }
    ensure_priority(&priority)?;

    let conn = state
        .db
        .get_connection()
        .map_err(|e| DentalCommandError::DatabaseError(e.to_string()))?;

    let latest_created: Option<String> = conn
        .query_row(
            "SELECT created_at FROM support_tickets WHERE customer_email = ?1 ORDER BY created_at DESC LIMIT 1",
            params![customer_email],
            |row| row.get(0),
        )
        .ok();

    if let Some(last) = latest_created {
        let last_dt = DateTime::parse_from_rfc3339(&last)
            .map(|d| d.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        let elapsed = Utc::now().signed_duration_since(last_dt).num_seconds();
        if elapsed < RATE_LIMIT_SECONDS {
            return Err(DentalCommandError::ValidationError(
                "Espera unos segundos antes de crear otro ticket".to_string(),
            ));
        }
    }

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO support_tickets (
            id, subject, description, status, priority,
            customer_name, customer_email, created_at, updated_at
        ) VALUES (?1, ?2, ?3, 'open', ?4, ?5, ?6, ?7, ?7)",
        params![id, subject, description, priority, customer_name, customer_email, now],
    )
    .map_err(|e| DentalCommandError::DatabaseError(e.to_string()))?;

    support_ticket_get(state, id)
}

#[tauri::command]
pub fn support_ticket_list(
    state: State<'_, DentalState>,
    filters: Option<SupportTicketFilters>,
) -> DentalCommandResult<Vec<SupportTicket>> {
    let conn = state
        .db
        .get_connection()
        .map_err(|e| DentalCommandError::DatabaseError(e.to_string()))?;

    let mut sql = String::from(
        "SELECT id, subject, description, status, priority, customer_name, customer_email, assigned_to, assigned_to_name, created_at, updated_at, first_response_at, last_response_at FROM support_tickets WHERE 1 = 1",
    );
    let mut bind_values: Vec<rusqlite::types::Value> = Vec::new();

    if let Some(f) = filters {
        if let Some(status) = f.status {
            if status != "all" {
                sql.push_str(" AND status = ?");
                bind_values.push(rusqlite::types::Value::Text(status));
            }
        }
        if let Some(priority) = f.priority {
            if priority != "all" {
                sql.push_str(" AND priority = ?");
                bind_values.push(rusqlite::types::Value::Text(priority));
            }
        }
        if let Some(search) = f.search {
            let term = search.trim();
            if !term.is_empty() {
                sql.push_str(" AND (subject LIKE ? OR customer_name LIKE ? OR customer_email LIKE ?)");
                let like = format!("%{}%", term);
                bind_values.push(rusqlite::types::Value::Text(like.clone()));
                bind_values.push(rusqlite::types::Value::Text(like.clone()));
                bind_values.push(rusqlite::types::Value::Text(like));
            }
        }
    }

    sql.push_str(" ORDER BY updated_at DESC");

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| DentalCommandError::DatabaseError(e.to_string()))?;

    let rows = stmt
        .query_map(rusqlite::params_from_iter(bind_values.iter()), |row| {
            Ok(SupportTicket {
                id: row.get(0)?,
                subject: row.get(1)?,
                description: row.get(2)?,
                status: row.get(3)?,
                priority: row.get(4)?,
                customer_name: row.get(5)?,
                customer_email: row.get(6)?,
                assigned_to: row.get(7)?,
                assigned_to_name: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
                first_response_at: row.get(11)?,
                last_response_at: row.get(12)?,
            })
        })
        .map_err(|e| DentalCommandError::DatabaseError(e.to_string()))?;

    let mut list = Vec::new();
    for item in rows {
        list.push(item.map_err(|e| DentalCommandError::DatabaseError(e.to_string()))?);
    }

    Ok(list)
}

#[tauri::command]
pub fn support_ticket_get(
    state: State<'_, DentalState>,
    ticket_id: String,
) -> DentalCommandResult<SupportTicket> {
    let conn = state
        .db
        .get_connection()
        .map_err(|e| DentalCommandError::DatabaseError(e.to_string()))?;

    conn.query_row(
        "SELECT id, subject, description, status, priority, customer_name, customer_email, assigned_to, assigned_to_name, created_at, updated_at, first_response_at, last_response_at FROM support_tickets WHERE id = ?1",
        params![ticket_id],
        |row| {
            Ok(SupportTicket {
                id: row.get(0)?,
                subject: row.get(1)?,
                description: row.get(2)?,
                status: row.get(3)?,
                priority: row.get(4)?,
                customer_name: row.get(5)?,
                customer_email: row.get(6)?,
                assigned_to: row.get(7)?,
                assigned_to_name: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
                first_response_at: row.get(11)?,
                last_response_at: row.get(12)?,
            })
        },
    )
    .map_err(|_| DentalCommandError::NotFound("Ticket no encontrado".to_string()))
}

#[tauri::command]
pub fn support_ticket_history_by_email(
    state: State<'_, DentalState>,
    customer_email: String,
) -> DentalCommandResult<Vec<SupportTicket>> {
    let conn = state
        .db
        .get_connection()
        .map_err(|e| DentalCommandError::DatabaseError(e.to_string()))?;

    let mut stmt = conn
        .prepare(
            "SELECT id, subject, description, status, priority, customer_name, customer_email, assigned_to, assigned_to_name, created_at, updated_at, first_response_at, last_response_at
             FROM support_tickets WHERE customer_email = ?1 ORDER BY created_at DESC",
        )
        .map_err(|e| DentalCommandError::DatabaseError(e.to_string()))?;

    let rows = stmt
        .query_map(params![customer_email.to_lowercase()], |row| {
            Ok(SupportTicket {
                id: row.get(0)?,
                subject: row.get(1)?,
                description: row.get(2)?,
                status: row.get(3)?,
                priority: row.get(4)?,
                customer_name: row.get(5)?,
                customer_email: row.get(6)?,
                assigned_to: row.get(7)?,
                assigned_to_name: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
                first_response_at: row.get(11)?,
                last_response_at: row.get(12)?,
            })
        })
        .map_err(|e| DentalCommandError::DatabaseError(e.to_string()))?;

    let mut list = Vec::new();
    for item in rows {
        list.push(item.map_err(|e| DentalCommandError::DatabaseError(e.to_string()))?);
    }
    Ok(list)
}

#[tauri::command]
pub fn support_ticket_list_messages(
    state: State<'_, DentalState>,
    ticket_id: String,
) -> DentalCommandResult<Vec<SupportTicketMessage>> {
    let conn = state
        .db
        .get_connection()
        .map_err(|e| DentalCommandError::DatabaseError(e.to_string()))?;

    let mut stmt = conn
        .prepare(
            "SELECT id, ticket_id, body, author_type, author_id, author_name, created_at, is_internal
             FROM support_ticket_messages WHERE ticket_id = ?1 ORDER BY created_at ASC",
        )
        .map_err(|e| DentalCommandError::DatabaseError(e.to_string()))?;

    let rows = stmt
        .query_map(params![ticket_id], |row| {
            let is_internal: i64 = row.get(7)?;
            Ok(SupportTicketMessage {
                id: row.get(0)?,
                ticket_id: row.get(1)?,
                body: row.get(2)?,
                author_type: row.get(3)?,
                author_id: row.get(4)?,
                author_name: row.get(5)?,
                created_at: row.get(6)?,
                is_internal: is_internal == 1,
            })
        })
        .map_err(|e| DentalCommandError::DatabaseError(e.to_string()))?;

    let mut list = Vec::new();
    for item in rows {
        list.push(item.map_err(|e| DentalCommandError::DatabaseError(e.to_string()))?);
    }
    Ok(list)
}

#[tauri::command]
pub fn support_ticket_add_message(
    state: State<'_, DentalState>,
    data: NewSupportTicketMessageInput,
) -> DentalCommandResult<SupportTicketMessage> {
    let ticket_id = data.ticket_id.clone();
    let author_type = data.author_type.trim().to_string();
    let author_name = data.author_name.trim().to_string();
    let author_id = data.author_id.clone();
    let body = data.body.trim().to_string();
    if body.len() < 2 {
        return Err(DentalCommandError::ValidationError(
            "El mensaje no puede estar vacío".to_string(),
        ));
    }

    if !matches!(author_type.as_str(), "customer" | "agent" | "system") {
        return Err(DentalCommandError::ValidationError("Autor inválido".to_string()));
    }

    let conn = state
        .db
        .get_connection()
        .map_err(|e| DentalCommandError::DatabaseError(e.to_string()))?;

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let is_internal = if data.is_internal.unwrap_or(false) { 1 } else { 0 };

    conn.execute(
        "INSERT INTO support_ticket_messages (id, ticket_id, body, author_type, author_id, author_name, created_at, is_internal)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            id,
            ticket_id,
            body,
            author_type,
            author_id,
            author_name,
            now,
            is_internal,
        ],
    )
    .map_err(|e| DentalCommandError::DatabaseError(e.to_string()))?;

    if data.author_type == "agent" {
        let first_response: Option<String> = conn
            .query_row(
                "SELECT first_response_at FROM support_tickets WHERE id = ?1",
                params![ticket_id.clone()],
                |row| row.get(0),
            )
            .ok();

        if first_response.is_some() {
            conn.execute(
                "UPDATE support_tickets SET updated_at = ?2, last_response_at = ?2 WHERE id = ?1",
                params![ticket_id.clone(), now],
            )
            .map_err(|e| DentalCommandError::DatabaseError(e.to_string()))?;
        } else {
            conn.execute(
                "UPDATE support_tickets SET updated_at = ?2, first_response_at = ?2, last_response_at = ?2 WHERE id = ?1",
                params![ticket_id.clone(), now],
            )
            .map_err(|e| DentalCommandError::DatabaseError(e.to_string()))?;
        }
    } else {
        conn.execute(
            "UPDATE support_tickets SET updated_at = ?2 WHERE id = ?1",
            params![ticket_id.clone(), now],
        )
        .map_err(|e| DentalCommandError::DatabaseError(e.to_string()))?;
    }

    Ok(SupportTicketMessage {
        id,
        ticket_id,
        body,
        author_type: data.author_type,
        author_id: data.author_id,
        author_name: data.author_name,
        created_at: now,
        is_internal: is_internal == 1,
    })
}

#[tauri::command]
pub fn support_ticket_assign(
    state: State<'_, DentalState>,
    ticket_id: String,
    agent_id: Option<String>,
) -> DentalCommandResult<()> {
    let conn = state
        .db
        .get_connection()
        .map_err(|e| DentalCommandError::DatabaseError(e.to_string()))?;

    let (assigned_to, assigned_to_name): (Option<String>, Option<String>) = if let Some(id) = agent_id {
        if id.trim().is_empty() {
            (None, None)
        } else {
            let full_name: String = conn
                .query_row(
                    "SELECT first_name || ' ' || last_name FROM users WHERE id = ?1 AND active = 1",
                    params![id],
                    |row| row.get(0),
                )
                .map_err(|_| DentalCommandError::NotFound("Agente no encontrado".to_string()))?;
            (Some(id), Some(full_name.trim().to_string()))
        }
    } else {
        (None, None)
    };

    conn.execute(
        "UPDATE support_tickets SET assigned_to = ?2, assigned_to_name = ?3, updated_at = ?4 WHERE id = ?1",
        params![ticket_id, assigned_to, assigned_to_name, Utc::now().to_rfc3339()],
    )
    .map_err(|e| DentalCommandError::DatabaseError(e.to_string()))?;

    Ok(())
}

#[tauri::command]
pub fn support_ticket_update_status(
    state: State<'_, DentalState>,
    ticket_id: String,
    next_status: String,
) -> DentalCommandResult<()> {
    ensure_status(&next_status)?;

    let conn = state
        .db
        .get_connection()
        .map_err(|e| DentalCommandError::DatabaseError(e.to_string()))?;

    let current: String = conn
        .query_row(
            "SELECT status FROM support_tickets WHERE id = ?1",
            params![ticket_id],
            |row| row.get(0),
        )
        .map_err(|_| DentalCommandError::NotFound("Ticket no encontrado".to_string()))?;

    if !can_transition_status(&current, &next_status) {
        return Err(DentalCommandError::ValidationError(
            "Transición de estado inválida".to_string(),
        ));
    }

    conn.execute(
        "UPDATE support_tickets SET status = ?2, updated_at = ?3 WHERE id = ?1",
        params![ticket_id, next_status, Utc::now().to_rfc3339()],
    )
    .map_err(|e| DentalCommandError::DatabaseError(e.to_string()))?;

    Ok(())
}

#[tauri::command]
pub fn support_ticket_list_agents(
    state: State<'_, DentalState>,
) -> DentalCommandResult<Vec<SupportTicketAgent>> {
    let conn = state
        .db
        .get_connection()
        .map_err(|e| DentalCommandError::DatabaseError(e.to_string()))?;

    let mut stmt = conn
        .prepare(
            "SELECT id, first_name, last_name, email, active FROM users WHERE active = 1 ORDER BY first_name, last_name",
        )
        .map_err(|e| DentalCommandError::DatabaseError(e.to_string()))?;

    let rows = stmt
        .query_map([], |row| {
            let first: String = row.get(1)?;
            let last: String = row.get(2)?;
            let is_active: i64 = row.get(4)?;
            Ok(SupportTicketAgent {
                id: row.get(0)?,
                full_name: format!("{} {}", first, last).trim().to_string(),
                email: row.get(3)?,
                is_active: is_active == 1,
            })
        })
        .map_err(|e| DentalCommandError::DatabaseError(e.to_string()))?;

    let mut agents = Vec::new();
    for item in rows {
        agents.push(item.map_err(|e| DentalCommandError::DatabaseError(e.to_string()))?);
    }

    Ok(agents)
}

#[tauri::command]
pub fn support_ticket_list_canned_responses(
    state: State<'_, DentalState>,
) -> DentalCommandResult<Vec<CannedResponse>> {
    let conn = state
        .db
        .get_connection()
        .map_err(|e| DentalCommandError::DatabaseError(e.to_string()))?;

    let mut stmt = conn
        .prepare(
            "SELECT id, title, body, is_active, created_at
             FROM support_canned_responses WHERE is_active = 1 ORDER BY title ASC",
        )
        .map_err(|e| DentalCommandError::DatabaseError(e.to_string()))?;

    let rows = stmt
        .query_map([], |row| {
            let is_active: i64 = row.get(3)?;
            Ok(CannedResponse {
                id: row.get(0)?,
                title: row.get(1)?,
                body: row.get(2)?,
                is_active: is_active == 1,
                created_at: row.get(4)?,
            })
        })
        .map_err(|e| DentalCommandError::DatabaseError(e.to_string()))?;

    let mut responses = Vec::new();
    for item in rows {
        responses.push(item.map_err(|e| DentalCommandError::DatabaseError(e.to_string()))?);
    }

    Ok(responses)
}
