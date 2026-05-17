use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use tauri::{AppHandle, Manager};

static MANIFEST_CACHE: OnceLock<Mutex<HashMap<String, Vec<PublicAssetManifestItemDto>>>> =
    OnceLock::new();

const PUBLIC_ASSET_ROOTS: &[&str] = &[
    "Bitmaps",
    "Graphics",
    "icons",
    "images",
    "LayoutPhantoms",
    "PredefinedElements",
    "library",
];

const XML_SAMPLE_BYTES: usize = 16 * 1024;
const STL_ASCII_SAMPLE_BYTES: usize = 512 * 1024;
const PRESET_SAMPLE_BYTES: usize = 160;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicAssetManifestQuery {
    pub root: String,
    pub subpath: Option<String>,
    pub force_refresh: Option<bool>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicAssetManifestItemDto {
    pub id: String,
    pub root: String,
    pub name: String,
    pub extension: String,
    pub kind: String,
    pub relative_path: String,
    pub root_relative_path: String,
    pub absolute_path: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicAssetManifestDto {
    pub root: String,
    pub subpath: Option<String>,
    pub total_count: usize,
    pub items: Vec<PublicAssetManifestItemDto>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicAssetRequest {
    pub root: String,
    pub relative_path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicAssetInsightDetailDto {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicAssetInsightDto {
    pub kind: String,
    pub title: String,
    pub details: Vec<PublicAssetInsightDetailDto>,
    pub sample: Option<String>,
}

fn manifest_cache() -> &'static Mutex<HashMap<String, Vec<PublicAssetManifestItemDto>>> {
    MANIFEST_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn validate_root(root: &str) -> Result<String, String> {
    if PUBLIC_ASSET_ROOTS
        .iter()
        .any(|candidate| *candidate == root)
    {
        Ok(root.to_string())
    } else {
        Err(format!("Unsupported public asset root: {root}"))
    }
}

fn normalize_relative_path(value: &str) -> Result<PathBuf, String> {
    let mut normalized = PathBuf::new();

    for component in Path::new(value).components() {
        match component {
            Component::CurDir => {}
            Component::Normal(segment) => normalized.push(segment),
            Component::Prefix(_) | Component::RootDir | Component::ParentDir => {
                return Err(format!(
                    "Invalid relative path segment in public asset path: {value}"
                ));
            }
        }
    }

    Ok(normalized)
}

fn repository_public_root(root: &str) -> Option<PathBuf> {
    let desktop_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent()?;
    let apps_root = desktop_root.parent()?;

    if root == "icons" {
        return Some(apps_root.join("packages").join("icons"));
    }

    Some(apps_root.join("packages").join("icons").join(root))
}

fn resolve_public_root(app: &AppHandle, root: &str) -> Result<PathBuf, String> {
    let root = validate_root(root)?;
    let mut candidates = Vec::new();

    if let Ok(resource_dir) = app.path().resource_dir() {
        candidates.push(resource_dir.join(&root));
        if root != "icons" {
            candidates.push(resource_dir.join("icons").join(&root));
        }
    }

    if let Some(repo_root) = repository_public_root(&root) {
        candidates.push(repo_root);
    }

    if let Some(desktop_root) = Path::new(env!("CARGO_MANIFEST_DIR")).parent() {
        if root == "icons" {
            candidates.push(desktop_root.join("packages").join("icons"));
        } else {
            candidates.push(desktop_root.join("packages").join("icons").join(&root));
        }
    }

    for candidate in candidates {
        if candidate.is_dir() {
            return Ok(candidate);
        }
    }

    Err(format!(
        "Could not resolve bundled public asset root: {root}"
    ))
}

fn resolve_public_asset_path(
    app: &AppHandle,
    root: &str,
    relative_path: &str,
) -> Result<PathBuf, String> {
    let root_path = resolve_public_root(app, root)?;
    let normalized_relative = normalize_relative_path(relative_path)?;
    if normalized_relative.as_os_str().is_empty() {
        return Err("Public asset relativePath cannot be empty".to_string());
    }

    let candidate = root_path.join(&normalized_relative);
    if !candidate.exists() {
        return Err(format!(
            "Public asset does not exist in root {root}: {}",
            normalized_relative.display()
        ));
    }

    let canonical_root = fs::canonicalize(&root_path).map_err(|error| {
        format!(
            "Could not canonicalize public asset root {}: {error}",
            root_path.display()
        )
    })?;
    let canonical_candidate = fs::canonicalize(&candidate).map_err(|error| {
        format!(
            "Could not canonicalize public asset path {}: {error}",
            candidate.display()
        )
    })?;

    if !canonical_candidate.starts_with(&canonical_root) {
        return Err("Public asset path escaped its declared root".to_string());
    }

    Ok(canonical_candidate)
}

fn get_extension(name: &str) -> String {
    Path::new(name)
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_lowercase()
}

fn infer_kind(extension: &str) -> &'static str {
    match extension {
        "png" | "jpg" | "jpeg" | "webp" | "avif" => "image",
        "svg" => "vector",
        "mp3" | "wav" => "audio",
        "stl" | "obj" | "ply" | "glb" | "gltf" | "off" => "model",
        "xml" | "json" => "xml",
        "sdfa" | "drmsdfa" | "eoff" | "barprofile" | "partinfo" | "metadata" | "matrix4"
        | "dentalproject" => "preset",
        "rendereffect" => "effect",
        "dll" | "db" => "binary",
        _ => "document",
    }
}

fn cache_key(root: &str, subpath: Option<&Path>) -> String {
    let subpath_key = subpath
        .map(|path| path.to_string_lossy().replace('\\', "/"))
        .unwrap_or_default();
    format!("{root}::{subpath_key}")
}

fn collect_manifest_items(
    scan_base: &Path,
    root: &str,
    limit: Option<usize>,
) -> Result<Vec<PublicAssetManifestItemDto>, String> {
    let mut stack = vec![scan_base.to_path_buf()];
    let mut items = Vec::new();
    let limit = limit.unwrap_or(usize::MAX);

    while let Some(current_dir) = stack.pop() {
        let entries = fs::read_dir(&current_dir).map_err(|error| {
            format!(
                "Could not read public asset directory {}: {error}",
                current_dir.display()
            )
        })?;

        for entry in entries {
            let entry =
                entry.map_err(|error| format!("Could not inspect public asset entry: {error}"))?;
            let path = entry.path();
            let file_name = entry.file_name().to_string_lossy().to_string();

            if matches!(file_name.as_str(), ".DS_Store" | "Thumbs.db") {
                continue;
            }

            let file_type = entry
                .file_type()
                .map_err(|error| format!("Could not inspect public asset file type: {error}"))?;

            if file_type.is_dir() {
                stack.push(path);
                continue;
            }

            if !file_type.is_file() {
                continue;
            }

            let relative = path
                .strip_prefix(scan_base)
                .map_err(|error| format!("Could not strip public asset prefix: {error}"))?;
            let relative_path = relative.to_string_lossy().replace('\\', "/");
            let metadata = entry.metadata().map_err(|error| {
                format!(
                    "Could not read public asset metadata {}: {error}",
                    path.display()
                )
            })?;
            let extension = get_extension(&file_name);

            items.push(PublicAssetManifestItemDto {
                id: format!("{root}/{relative_path}"),
                root: root.to_string(),
                name: file_name,
                extension: extension.clone(),
                kind: infer_kind(&extension).to_string(),
                relative_path: format!("{root}/{relative_path}"),
                root_relative_path: relative_path,
                absolute_path: path.to_string_lossy().to_string(),
                bytes: metadata.len(),
            });

            if items.len() >= limit {
                items.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
                return Ok(items);
            }
        }
    }

    items.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(items)
}

fn read_prefix_bytes(path: &Path, limit: usize) -> Result<Vec<u8>, String> {
    let mut file = File::open(path)
        .map_err(|error| format!("Could not open public asset {}: {error}", path.display()))?;
    let mut buffer = vec![0u8; limit];
    let bytes_read = file
        .read(&mut buffer)
        .map_err(|error| format!("Could not read public asset {}: {error}", path.display()))?;
    buffer.truncate(bytes_read);
    Ok(buffer)
}

fn read_prefix_text(path: &Path, limit: usize) -> Result<String, String> {
    Ok(String::from_utf8_lossy(&read_prefix_bytes(path, limit)?).to_string())
}

fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

fn bytes_to_hex(bytes: &[u8], limit: usize) -> String {
    bytes
        .iter()
        .take(limit)
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn extract_xml_root_tag(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] != b'<' {
            index += 1;
            continue;
        }

        if input[index..].starts_with("<?") {
            if let Some(end) = input[index..].find("?>") {
                index += end + 2;
                continue;
            }
            break;
        }

        if input[index..].starts_with("<!--") {
            if let Some(end) = input[index..].find("-->") {
                index += end + 3;
                continue;
            }
            break;
        }

        if input[index..].starts_with("<!") || input[index..].starts_with("</") {
            index += 2;
            continue;
        }

        let start = index + 1;
        let mut end = start;
        while end < bytes.len() {
            let current = bytes[end];
            if current.is_ascii_whitespace() || current == b'>' || current == b'/' {
                break;
            }
            end += 1;
        }

        if end > start {
            return input[start..end].to_string();
        }

        index += 1;
    }

    "unknown".to_string()
}

fn load_textual_insight(
    path: &Path,
    extension: &str,
    size: u64,
) -> Result<PublicAssetInsightDto, String> {
    let raw_text = read_prefix_text(path, XML_SAMPLE_BYTES)?;
    let title = if extension == "json" {
        "JSON document".to_string()
    } else {
        extract_xml_root_tag(&raw_text)
    };

    Ok(PublicAssetInsightDto {
        kind: "xml".to_string(),
        title: title.clone(),
        details: vec![
            PublicAssetInsightDetailDto {
                label: if extension == "json" {
                    "Document".to_string()
                } else {
                    "Root tag".to_string()
                },
                value: title,
            },
            PublicAssetInsightDetailDto {
                label: "Size".to_string(),
                value: format_bytes(size),
            },
        ],
        sample: Some(raw_text.lines().take(8).collect::<Vec<_>>().join("\n")),
    })
}

fn load_stl_insight(path: &Path, size: u64) -> Result<PublicAssetInsightDto, String> {
    let header = read_prefix_bytes(path, 84)?;
    let ascii_header = String::from_utf8_lossy(&header[..header.len().min(80)]).to_string();
    let is_ascii = ascii_header.trim_start().starts_with("solid");

    let triangle_count = if is_ascii {
        read_prefix_text(path, STL_ASCII_SAMPLE_BYTES)?
            .matches("facet normal")
            .count()
            .to_string()
    } else if header.len() >= 84 {
        let raw = u32::from_le_bytes([header[80], header[81], header[82], header[83]]);
        raw.to_string()
    } else {
        "Unknown".to_string()
    };

    Ok(PublicAssetInsightDto {
        kind: "stl".to_string(),
        title: if is_ascii {
            "ASCII STL mesh".to_string()
        } else {
            "Binary STL mesh".to_string()
        },
        details: vec![
            PublicAssetInsightDetailDto {
                label: "Format".to_string(),
                value: if is_ascii {
                    "ASCII STL".to_string()
                } else {
                    "Binary STL".to_string()
                },
            },
            PublicAssetInsightDetailDto {
                label: "Triangles".to_string(),
                value: triangle_count,
            },
            PublicAssetInsightDetailDto {
                label: "Size".to_string(),
                value: format_bytes(size),
            },
        ],
        sample: Some(ascii_header.trim().chars().take(160).collect()),
    })
}

fn load_preset_insight(
    path: &Path,
    extension: &str,
    size: u64,
) -> Result<PublicAssetInsightDto, String> {
    let bytes = read_prefix_bytes(path, PRESET_SAMPLE_BYTES)?;
    Ok(PublicAssetInsightDto {
        kind: "preset".to_string(),
        title: format!("{} asset", extension.to_uppercase()),
        details: vec![
            PublicAssetInsightDetailDto {
                label: "Extension".to_string(),
                value: extension.to_string(),
            },
            PublicAssetInsightDetailDto {
                label: "Signature".to_string(),
                value: bytes_to_hex(&bytes, 16),
            },
            PublicAssetInsightDetailDto {
                label: "Size".to_string(),
                value: format_bytes(size),
            },
        ],
        sample: Some(
            String::from_utf8_lossy(&bytes)
                .replace('\0', "·")
                .trim()
                .to_string(),
        ),
    })
}

fn load_generic_insight(kind: &str, extension: &str, size: u64) -> PublicAssetInsightDto {
    PublicAssetInsightDto {
        kind: "generic".to_string(),
        title: format!("{kind} asset"),
        details: vec![
            PublicAssetInsightDetailDto {
                label: "Kind".to_string(),
                value: kind.to_string(),
            },
            PublicAssetInsightDetailDto {
                label: "Extension".to_string(),
                value: if extension.is_empty() {
                    "unknown".to_string()
                } else {
                    extension.to_string()
                },
            },
            PublicAssetInsightDetailDto {
                label: "Size".to_string(),
                value: format_bytes(size),
            },
        ],
        sample: None,
    }
}

#[tauri::command]
pub fn get_public_asset_manifest(
    app: AppHandle,
    query: PublicAssetManifestQuery,
) -> Result<PublicAssetManifestDto, String> {
    let root = validate_root(&query.root)?;
    let normalized_subpath = query
        .subpath
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(normalize_relative_path)
        .transpose()?;
    let cache_key = cache_key(&root, normalized_subpath.as_deref());
    let force_refresh = query.force_refresh.unwrap_or(false);

    if force_refresh {
        manifest_cache()
            .lock()
            .map_err(|_| "Could not lock public asset manifest cache".to_string())?
            .remove(&cache_key);
    }

    let cached_items = {
        let cache = manifest_cache()
            .lock()
            .map_err(|_| "Could not lock public asset manifest cache".to_string())?;
        cache.get(&cache_key).cloned()
    };

    let items = if let Some(items) = cached_items {
        items
    } else {
        let root_dir = resolve_public_root(&app, &root)?;
        let scan_dir = if let Some(subpath) = &normalized_subpath {
            let candidate = root_dir.join(subpath);
            if !candidate.exists() {
                Vec::new()
            } else if candidate.is_file() {
                let parent = candidate.parent().unwrap_or(&root_dir);
                collect_manifest_items(parent, &root, None)?
                    .into_iter()
                    .filter(|item| item.absolute_path == candidate.to_string_lossy())
                    .collect::<Vec<_>>()
            } else {
                collect_manifest_items(&candidate, &root, None)?
            }
        } else {
            collect_manifest_items(&root_dir, &root, None)?
        };

        manifest_cache()
            .lock()
            .map_err(|_| "Could not lock public asset manifest cache".to_string())?
            .insert(cache_key.clone(), scan_dir.clone());

        scan_dir
    };

    let limited_items = query
        .limit
        .map(|limit| items.iter().take(limit).cloned().collect::<Vec<_>>())
        .unwrap_or_else(|| items.clone());

    Ok(PublicAssetManifestDto {
        root,
        subpath: normalized_subpath.map(|path| path.to_string_lossy().replace('\\', "/")),
        total_count: items.len(),
        items: limited_items,
    })
}

#[tauri::command]
pub fn inspect_public_asset(
    app: AppHandle,
    request: PublicAssetRequest,
) -> Result<PublicAssetInsightDto, String> {
    let path = resolve_public_asset_path(&app, &request.root, &request.relative_path)?;
    let metadata = fs::metadata(&path)
        .map_err(|error| format!("Could not stat public asset {}: {error}", path.display()))?;
    let size = metadata.len();
    let extension = get_extension(
        path.file_name()
            .and_then(|file_name| file_name.to_str())
            .unwrap_or_default(),
    );
    let kind = infer_kind(&extension);

    match extension.as_str() {
        "xml" | "json" => load_textual_insight(&path, &extension, size),
        "stl" => load_stl_insight(&path, size),
        "sdfa" | "drmsdfa" | "eoff" | "barprofile" | "partinfo" | "metadata" | "matrix4"
        | "dentalproject" | "rendereffect" => load_preset_insight(&path, &extension, size),
        _ => Ok(load_generic_insight(kind, &extension, size)),
    }
}
