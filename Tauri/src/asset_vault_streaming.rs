use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetVaultSourceMetadataDto {
    pub source_path: String,
    pub source_name: String,
    pub is_directory: bool,
    pub file_count: u64,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetVaultChunkManifestDto {
    pub index: u64,
    pub offset_bytes: u64,
    pub size_bytes: u64,
    pub checksum_sha256: String,
    pub storage_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetVaultManifestDto {
    pub contract_version: String,
    pub case_id: String,
    pub asset_id: String,
    pub kind: String,
    pub format: String,
    pub storage_path: String,
    pub checksum_sha256: String,
    pub total_bytes: u64,
    pub chunk_size_bytes: u64,
    pub chunks: Vec<AssetVaultChunkManifestDto>,
    pub source: AssetVaultSourceMetadataDto,
    pub caller_metadata: serde_json::Value,
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn safe_segment(value: &str, fallback: &str) -> String {
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

pub fn safe_filename(value: &str, fallback: &str) -> String {
    Path::new(value)
        .file_name()
        .and_then(|name| name.to_str())
        .map(|value| safe_segment(value, fallback))
        .unwrap_or_else(|| fallback.to_string())
}

pub fn collect_source_files(source_path: &Path) -> Result<Vec<PathBuf>, String> {
    if source_path.is_file() {
        return Ok(vec![source_path.to_path_buf()]);
    }

    if !source_path.is_dir() {
        return Err(format!(
            "asset vault import expects a readable file or directory path, got {}",
            source_path.display()
        ));
    }

    let mut pending = vec![source_path.to_path_buf()];
    let mut files = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(&directory).map_err(|error| {
            format!(
                "could not read source directory {}: {error}",
                directory.display()
            )
        })? {
            let entry =
                entry.map_err(|error| format!("could not read source directory entry: {error}"))?;
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if path.is_file() {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

pub fn source_file_bytes_with_limit(files: &[PathBuf], max_bytes: u64) -> Result<u64, String> {
    files.iter().try_fold(0_u64, |acc, path| {
        let metadata = fs::metadata(path)
            .map_err(|error| format!("could not inspect source file {}: {error}", path.display()))?;

        if !metadata.is_file() {
            return Err(format!(
                "asset vault source entry is not a regular file: {}",
                path.display()
            ));
        }

        let next = acc.checked_add(metadata.len()).ok_or_else(|| {
            format!(
                "asset vault source size overflow while inspecting {}",
                path.display()
            )
        })?;

        if next > max_bytes {
            return Err(format!(
                "asset vault source is too large: {next} bytes exceeds the {max_bytes} byte guardrail"
            ));
        }

        Ok(next)
    })
}

pub fn source_metadata(
    source_path: &Path,
    files: &[PathBuf],
    total_bytes: u64,
) -> AssetVaultSourceMetadataDto {
    let source_name = source_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("asset.bin")
        .to_string();

    AssetVaultSourceMetadataDto {
        source_path: source_path.to_string_lossy().to_string(),
        source_name,
        is_directory: source_path.is_dir(),
        file_count: files.len() as u64,
        total_bytes,
    }
}

pub fn write_manifest_file(path: &Path, manifest: &AssetVaultManifestDto) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "could not create manifest directory {}: {error}",
                parent.display()
            )
        })?;
    }

    let file = File::create(path).map_err(|error| {
        format!(
            "could not create asset vault manifest {}: {error}",
            path.display()
        )
    })?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, manifest)
        .map_err(|error| format!("could not serialize asset vault manifest: {error}"))?;
    writer.flush().map_err(|error| {
        format!(
            "could not flush asset vault manifest {}: {error}",
            path.display()
        )
    })
}

pub fn checksum_file_streaming(path: &Path, chunk_size: usize) -> Result<(String, u64), String> {
    let file = File::open(path).map_err(|error| {
        format!(
            "could not open asset for checksum {}: {error}",
            path.display()
        )
    })?;
    let mut reader = BufReader::with_capacity(chunk_size, file);
    let mut buffer = vec![0_u8; chunk_size];
    let mut hasher = Sha256::new();
    let mut total = 0_u64;

    loop {
        let bytes_read = reader
            .read(&mut buffer)
            .map_err(|error| format!("could not read asset checksum chunk: {error}"))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
        total += bytes_read as u64;
    }

    Ok((
        hasher
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
        total,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_filename_strips_path_segments() {
        assert_eq!(
            safe_filename("/tmp/prep scan.STL", "asset.bin"),
            "prepscan.STL"
        );
        assert_eq!(safe_filename("", "asset.bin"), "asset.bin");
    }

    #[test]
    fn sha256_hex_is_stable() {
        assert_eq!(
            sha256_hex(b"tlanticad"),
            "60448e3f97037ec5c23915c525b490ae14acf0fe4ec50b6bd4c950bfbfa58644"
        );
    }
}
