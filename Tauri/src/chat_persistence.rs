use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatSessionMeta {
    pub id: String,
    pub title: String,
    pub preview: String,
    pub created_at: f64,
    pub updated_at: f64,
    pub message_count: usize,
    pub model_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredChatMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    pub created_at: String,
    pub model_id: Option<String>,
    pub provider: Option<String>,
    #[serde(default)]
    pub tool_calls: Vec<serde_json::Value>,
    #[serde(default)]
    pub images: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredChatSession {
    pub id: String,
    pub title: String,
    pub messages: Vec<StoredChatMessage>,
    pub created_at: f64,
    pub updated_at: f64,
    pub model_id: String,
}

fn safe_segment(value: &str, fallback: &str) -> String {
    let segment: String = value
        .chars()
        .filter(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
        })
        .collect();

    if segment.is_empty() {
        fallback.to_string()
    } else {
        segment
    }
}

fn project_hash(project_path: &str) -> String {
    let mut hasher = DefaultHasher::new();
    project_path.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn chat_root(app: &AppHandle, project_path: &str) -> Result<PathBuf, String> {
    let root = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve app data dir: {error}"))?;
    Ok(root
        .join("chat")
        .join(project_hash(project_path))
        .join("sessions"))
}

fn session_path(app: &AppHandle, project_path: &str, session_id: &str) -> Result<PathBuf, String> {
    Ok(chat_root(app, project_path)?.join(format!("{}.json", safe_segment(session_id, "session"))))
}

#[tauri::command]
pub fn chat_init(app: AppHandle, project_path: String) -> Result<(), String> {
    fs::create_dir_all(chat_root(&app, &project_path)?)
        .map_err(|error| format!("Could not create chat session directory: {error}"))
}

#[tauri::command]
pub fn chat_save_session(
    app: AppHandle,
    project_path: String,
    session: StoredChatSession,
) -> Result<(), String> {
    chat_init(app.clone(), project_path.clone())?;
    let path = session_path(&app, &project_path, &session.id)?;
    let payload = serde_json::to_string_pretty(&session)
        .map_err(|error| format!("Could not serialize chat session: {error}"))?;
    fs::write(&path, payload)
        .map_err(|error| format!("Could not write chat session {}: {error}", path.display()))
}

#[tauri::command]
pub fn chat_load_session(
    app: AppHandle,
    project_path: String,
    session_id: String,
) -> Result<StoredChatSession, String> {
    let path = session_path(&app, &project_path, &session_id)?;
    let payload = fs::read_to_string(&path)
        .map_err(|error| format!("Could not read chat session {}: {error}", path.display()))?;
    serde_json::from_str(&payload)
        .map_err(|error| format!("Could not parse chat session {}: {error}", path.display()))
}

#[tauri::command]
pub fn chat_list_sessions(
    app: AppHandle,
    project_path: String,
) -> Result<Vec<ChatSessionMeta>, String> {
    let root = chat_root(&app, &project_path)?;
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut sessions = Vec::new();
    for entry in fs::read_dir(&root)
        .map_err(|error| format!("Could not list chat sessions {}: {error}", root.display()))?
    {
        let path = entry
            .map_err(|error| format!("Could not read chat session entry: {error}"))?
            .path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let payload = fs::read_to_string(&path)
            .map_err(|error| format!("Could not read chat session {}: {error}", path.display()))?;
        let session: StoredChatSession = serde_json::from_str(&payload)
            .map_err(|error| format!("Could not parse chat session {}: {error}", path.display()))?;
        let preview = session
            .messages
            .iter()
            .find(|message| message.role == "user")
            .map(|message| message.content.chars().take(90).collect())
            .unwrap_or_default();
        sessions.push(ChatSessionMeta {
            id: session.id,
            title: session.title,
            preview,
            created_at: session.created_at,
            updated_at: session.updated_at,
            message_count: session.messages.len(),
            model_id: session.model_id,
        });
    }

    sessions.sort_by(|a, b| b.updated_at.total_cmp(&a.updated_at));
    Ok(sessions)
}

#[tauri::command]
pub fn chat_delete_session(
    app: AppHandle,
    project_path: String,
    session_id: String,
) -> Result<(), String> {
    let path = session_path(&app, &project_path, &session_id)?;
    match fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!(
            "Could not delete chat session {}: {error}",
            path.display()
        )),
    }
}
