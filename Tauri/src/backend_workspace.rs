use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

const MAX_FUNCTIONS_PER_CRATE: usize = 12;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceFunction {
    pub name: String,
    pub signature: String,
    pub file: String,
    pub line: usize,
    pub kind: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceCrateInfo {
    pub name: String,
    pub package_name: String,
    pub description: Option<String>,
    pub relative_path: String,
    pub rust_file_count: usize,
    pub public_function_count: usize,
    pub tauri_command_count: usize,
    pub top_functions: Vec<WorkspaceFunction>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackendWorkspaceCatalog {
    pub workspace_root: String,
    pub crate_count: usize,
    pub rust_file_count: usize,
    pub public_function_count: usize,
    pub tauri_command_count: usize,
    pub route: &'static str,
    pub crates: Vec<WorkspaceCrateInfo>,
}

#[derive(Default)]
struct CrateScanStats {
    rust_file_count: usize,
    public_function_count: usize,
    tauri_command_count: usize,
    functions: Vec<WorkspaceFunction>,
}

pub fn inspect_backend_workspace() -> Result<BackendWorkspaceCatalog, String> {
    let crates_root = resolve_crates_root()?;
    let mut crates = Vec::new();

    let entries = fs::read_dir(&crates_root).map_err(|error| {
        format!(
            "Failed to read crates directory {}: {error}",
            crates_root.display()
        )
    })?;

    for entry in entries {
        let entry = entry.map_err(|error| format!("Failed to read crates entry: {error}"))?;
        let entry_path = entry.path();

        if !entry_path.is_dir() {
            continue;
        }

        if entry.file_name().to_string_lossy().starts_with('.') || entry.file_name() == "target" {
            continue;
        }

        let cargo_toml_path = entry_path.join("Cargo.toml");
        if !cargo_toml_path.exists() {
            continue;
        }

        crates.push(inspect_crate(&crates_root, &entry_path, &cargo_toml_path)?);
    }

    crates.sort_by(|left, right| {
        right
            .public_function_count
            .cmp(&left.public_function_count)
            .then_with(|| left.name.cmp(&right.name))
    });

    let rust_file_count = crates.iter().map(|item| item.rust_file_count).sum();
    let public_function_count = crates.iter().map(|item| item.public_function_count).sum();
    let tauri_command_count = crates.iter().map(|item| item.tauri_command_count).sum();

    Ok(BackendWorkspaceCatalog {
        workspace_root: crates_root.display().to_string(),
        crate_count: crates.len(),
        rust_file_count,
        public_function_count,
        tauri_command_count,
        route: "workspace/backend/workspace-crates",
        crates,
    })
}

fn resolve_crates_root() -> Result<PathBuf, String> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    Ok(manifest_dir.join("crates"))
}

fn inspect_crate(
    crates_root: &Path,
    crate_dir: &Path,
    cargo_toml_path: &Path,
) -> Result<WorkspaceCrateInfo, String> {
    let cargo_toml = fs::read_to_string(cargo_toml_path)
        .map_err(|error| format!("Failed to read {}: {error}", cargo_toml_path.display()))?;

    let package_name = extract_package_field(&cargo_toml, "name").unwrap_or_else(|| {
        crate_dir
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    });
    let description = extract_package_field(&cargo_toml, "description");

    let src_dir = crate_dir.join("src");
    let stats = if src_dir.exists() {
        scan_rust_tree(crate_dir, &src_dir)?
    } else {
        CrateScanStats::default()
    };

    Ok(WorkspaceCrateInfo {
        name: crate_dir
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        package_name,
        description,
        relative_path: crate_dir
            .strip_prefix(crates_root)
            .unwrap_or(crate_dir)
            .display()
            .to_string(),
        rust_file_count: stats.rust_file_count,
        public_function_count: stats.public_function_count,
        tauri_command_count: stats.tauri_command_count,
        top_functions: stats.functions,
    })
}

fn scan_rust_tree(crate_root: &Path, dir: &Path) -> Result<CrateScanStats, String> {
    let mut stats = CrateScanStats::default();

    let entries = fs::read_dir(dir)
        .map_err(|error| format!("Failed to read source directory {}: {error}", dir.display()))?;

    for entry in entries {
        let entry = entry.map_err(|error| format!("Failed to inspect source entry: {error}"))?;
        let path = entry.path();

        if path.is_dir() {
            let nested = scan_rust_tree(crate_root, &path)?;
            stats.rust_file_count += nested.rust_file_count;
            stats.public_function_count += nested.public_function_count;
            stats.tauri_command_count += nested.tauri_command_count;

            for function in nested.functions {
                push_top_function(&mut stats.functions, function);
            }

            continue;
        }

        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }

        stats.rust_file_count += 1;
        let file_stats = inspect_rust_file(crate_root, &path)?;
        stats.public_function_count += file_stats.public_function_count;
        stats.tauri_command_count += file_stats.tauri_command_count;

        for function in file_stats.functions {
            push_top_function(&mut stats.functions, function);
        }
    }

    Ok(stats)
}

fn inspect_rust_file(crate_root: &Path, file_path: &Path) -> Result<CrateScanStats, String> {
    let source = fs::read_to_string(file_path)
        .map_err(|error| format!("Failed to read {}: {error}", file_path.display()))?;

    let mut stats = CrateScanStats::default();
    let mut pending_tauri_command = false;

    for (index, raw_line) in source.lines().enumerate() {
        let line_number = index + 1;
        let trimmed = raw_line.trim();

        if trimmed.starts_with("#[tauri::command]") {
            pending_tauri_command = true;
            continue;
        }

        if let Some((name, kind)) = parse_public_function_signature(trimmed) {
            stats.public_function_count += 1;

            let function = WorkspaceFunction {
                name,
                signature: trimmed.to_string(),
                file: file_path
                    .strip_prefix(crate_root)
                    .unwrap_or(file_path)
                    .display()
                    .to_string(),
                line: line_number,
                kind: if pending_tauri_command {
                    stats.tauri_command_count += 1;
                    format!("tauri-{kind}")
                } else {
                    kind.to_string()
                },
            };

            push_top_function(&mut stats.functions, function);
            pending_tauri_command = false;
            continue;
        }

        if !trimmed.is_empty() && !trimmed.starts_with("#") {
            pending_tauri_command = false;
        }
    }

    Ok(stats)
}

fn parse_public_function_signature(line: &str) -> Option<(String, &'static str)> {
    let prefixes = [("pub async fn ", "async-fn"), ("pub fn ", "fn")];

    for (prefix, kind) in prefixes {
        if let Some(rest) = line.strip_prefix(prefix) {
            let name = rest
                .split(|character: char| {
                    character == '(' || character.is_whitespace() || character == '<'
                })
                .next()
                .unwrap_or_default()
                .trim();

            if !name.is_empty() {
                return Some((name.to_string(), kind));
            }
        }
    }

    None
}

fn push_top_function(functions: &mut Vec<WorkspaceFunction>, function: WorkspaceFunction) {
    if functions.len() < MAX_FUNCTIONS_PER_CRATE {
        functions.push(function);
    }
}

fn extract_package_field(cargo_toml: &str, field: &str) -> Option<String> {
    let mut in_package = false;
    let key = format!("{field} = \"");

    for raw_line in cargo_toml.lines() {
        let line = raw_line.trim();

        if line.starts_with('[') {
            in_package = line == "[package]";
            continue;
        }

        if !in_package || !line.starts_with(&key) {
            continue;
        }

        let value = line
            .trim_start_matches(&key)
            .trim_end_matches('"')
            .trim()
            .to_string();

        if !value.is_empty() {
            return Some(value);
        }
    }

    None
}
