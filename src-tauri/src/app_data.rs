use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const DATA_GENERATION: &str = "2026.07.multi-drive.1";
const DATA_GENERATION_FILE: &str = "data-generation";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppDataSummary {
    pub root: String,
    pub snapshot_count: u64,
    pub receipt_count: u64,
    pub vault_entry_count: u64,
    pub vault_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppDataMutation {
    pub removed_entries: u64,
    pub removed_bytes: u64,
    pub included_restore_files: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResetAppRequest {
    #[serde(default)]
    pub include_restore_files: bool,
}

pub fn initialize() -> Result<()> {
    let root = app_root();
    fs::create_dir_all(&root)?;
    let generation_path = root.join(DATA_GENERATION_FILE);
    let current = fs::read_to_string(&generation_path).unwrap_or_default();

    if current.trim() != DATA_GENERATION {
        remove_owned_path(&snapshot_root())?;
        remove_owned_path(&receipt_root())?;
        fs::write(generation_path, DATA_GENERATION)?;
    }

    Ok(())
}

pub fn summary() -> Result<AppDataSummary> {
    let snapshots = direct_json_count(&snapshot_root())?;
    let receipts = direct_json_count(&receipt_root())?;
    let vault = tree_stats(&vault_root())?;
    let vault_entry_count = if vault_root().exists() {
        fs::read_dir(vault_root())?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().join("manifest.json").is_file())
            .count() as u64
    } else {
        0
    };

    Ok(AppDataSummary {
        root: app_root().to_string_lossy().to_string(),
        snapshot_count: snapshots,
        receipt_count: receipts,
        vault_entry_count,
        vault_bytes: vault.bytes,
    })
}

pub fn clear_scan_history() -> Result<AppDataMutation> {
    clear_owned_directory(&snapshot_root(), false)
}

pub fn clear_cleanup_records() -> Result<AppDataMutation> {
    clear_owned_directory(&receipt_root(), false)
}

pub fn reset(request: &ResetAppRequest) -> Result<AppDataMutation> {
    let root = app_root();
    fs::create_dir_all(&root)?;
    let mut mutation = AppDataMutation {
        removed_entries: 0,
        removed_bytes: 0,
        included_restore_files: request.include_restore_files,
    };

    for entry in fs::read_dir(&root)? {
        let entry = entry?;
        let name = entry.file_name();
        if name == DATA_GENERATION_FILE || name == "models" {
            continue;
        }
        if !request.include_restore_files && name == "vault" {
            continue;
        }
        merge_mutation(&mut mutation, remove_owned_path(&entry.path())?);
    }

    fs::write(root.join(DATA_GENERATION_FILE), DATA_GENERATION)?;
    Ok(mutation)
}

pub fn app_root() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("WinReclaim")
}

pub fn snapshot_root() -> PathBuf {
    app_root().join("snapshots")
}

pub fn receipt_root() -> PathBuf {
    app_root().join("receipts")
}

pub fn vault_root() -> PathBuf {
    app_root().join("vault")
}

fn clear_owned_directory(path: &Path, include_restore_files: bool) -> Result<AppDataMutation> {
    let mut mutation = remove_owned_path(path)?;
    mutation.included_restore_files = include_restore_files;
    Ok(mutation)
}

fn remove_owned_path(path: &Path) -> Result<AppDataMutation> {
    if !path.exists() {
        return Ok(AppDataMutation {
            removed_entries: 0,
            removed_bytes: 0,
            included_restore_files: false,
        });
    }

    let stats = tree_stats(path)?;
    let metadata = fs::symlink_metadata(path)?;
    if metadata.is_dir() {
        fs::remove_dir_all(path)?;
    } else {
        fs::remove_file(path)?;
    }

    Ok(AppDataMutation {
        removed_entries: stats.entries,
        removed_bytes: stats.bytes,
        included_restore_files: false,
    })
}

fn direct_json_count(path: &Path) -> Result<u64> {
    if !path.exists() {
        return Ok(0);
    }
    Ok(fs::read_dir(path)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().and_then(|value| value.to_str()) == Some("json"))
        .count() as u64)
}

#[derive(Debug, Default, Clone, Copy)]
struct TreeStats {
    entries: u64,
    bytes: u64,
}

fn tree_stats(root: &Path) -> Result<TreeStats> {
    if !root.exists() {
        return Ok(TreeStats::default());
    }

    let mut stats = TreeStats::default();
    let mut stack = vec![root.to_path_buf()];
    while let Some(path) = stack.pop() {
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(_) => continue,
        };
        stats.entries = stats.entries.saturating_add(1);
        if metadata.is_file() {
            stats.bytes = stats.bytes.saturating_add(metadata.len());
            continue;
        }
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            continue;
        }
        let entries = match fs::read_dir(path) {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            stack.push(entry.path());
        }
    }
    Ok(stats)
}

fn merge_mutation(target: &mut AppDataMutation, next: AppDataMutation) {
    target.removed_entries = target.removed_entries.saturating_add(next.removed_entries);
    target.removed_bytes = target.removed_bytes.saturating_add(next.removed_bytes);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reset_request_keeps_restore_files_by_default() {
        let request: ResetAppRequest = serde_json::from_str("{}").unwrap();
        assert!(!request.include_restore_files);
    }
}
