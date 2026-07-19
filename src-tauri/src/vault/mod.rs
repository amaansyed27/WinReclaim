use crate::domain::{RestoreResult, VaultEntry, VaultStatus};
use anyhow::{anyhow, Result};
use chrono::{Duration, Utc};
use std::fs;
use std::path::{Component, Path, PathBuf};
use uuid::Uuid;

const RETENTION_DAYS: i64 = 7;

#[derive(Debug)]
pub struct QuarantineOutcome {
    pub moved_entries: u64,
    pub skipped_entries: u64,
    pub entry: Option<VaultEntry>,
}

pub fn quarantine_files(
    original_root: &Path,
    files: &[PathBuf],
    receipt_id: Uuid,
    finding_id: Uuid,
    display_name: &str,
) -> Result<QuarantineOutcome> {
    if files.is_empty() {
        return Ok(QuarantineOutcome {
            moved_entries: 0,
            skipped_entries: 0,
            entry: None,
        });
    }

    let id = Uuid::new_v4();
    let entry_root = vault_root().join(id.to_string());
    let payload_root = entry_root.join("payload");
    fs::create_dir_all(&payload_root)?;

    let mut moved_entries = 0_u64;
    let mut skipped_entries = 0_u64;
    let mut stored_bytes = 0_u64;
    let mut relative_paths = Vec::new();

    for source in files {
        let relative = match source.strip_prefix(original_root) {
            Ok(relative) if is_safe_relative(relative) => relative,
            _ => {
                skipped_entries = skipped_entries.saturating_add(1);
                continue;
            }
        };
        let bytes = match fs::metadata(source) {
            Ok(metadata) if metadata.is_file() => metadata.len(),
            _ => {
                skipped_entries = skipped_entries.saturating_add(1);
                continue;
            }
        };
        let destination = payload_root.join(relative);
        if let Some(parent) = destination.parent() {
            if fs::create_dir_all(parent).is_err() {
                skipped_entries = skipped_entries.saturating_add(1);
                continue;
            }
        }
        if move_file(source, &destination).is_err() {
            skipped_entries = skipped_entries.saturating_add(1);
            continue;
        }
        moved_entries = moved_entries.saturating_add(1);
        stored_bytes = stored_bytes.saturating_add(bytes);
        relative_paths.push(relative.to_string_lossy().to_string());
    }

    if moved_entries == 0 {
        let _ = fs::remove_dir_all(entry_root);
        return Ok(QuarantineOutcome {
            moved_entries,
            skipped_entries,
            entry: None,
        });
    }

    let created_at = Utc::now();
    let entry = VaultEntry {
        id,
        receipt_id,
        finding_id,
        display_name: display_name.to_string(),
        original_root: original_root.to_string_lossy().to_string(),
        payload_root: payload_root.to_string_lossy().to_string(),
        relative_paths,
        stored_bytes,
        created_at,
        expires_at: created_at + Duration::days(RETENTION_DAYS),
        restored_at: None,
        status: VaultStatus::Active,
    };
    persist_entry(&entry)?;

    Ok(QuarantineOutcome {
        moved_entries,
        skipped_entries,
        entry: Some(entry),
    })
}

pub fn list_entries() -> Result<Vec<VaultEntry>> {
    purge_expired_entries()?;
    let root = vault_root();
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut entries = Vec::new();
    for directory in fs::read_dir(root)? {
        let directory = match directory {
            Ok(directory) => directory,
            Err(_) => continue,
        };
        let manifest = directory.path().join("manifest.json");
        let bytes = match fs::read(manifest) {
            Ok(bytes) => bytes,
            Err(_) => continue,
        };
        if let Ok(entry) = serde_json::from_slice::<VaultEntry>(&bytes) {
            entries.push(entry);
        }
    }
    entries.sort_by_key(|entry| std::cmp::Reverse(entry.created_at));
    Ok(entries)
}

pub fn restore_entry(id: Uuid) -> Result<RestoreResult> {
    let mut entry = load_entry(id)?;
    if entry.status != VaultStatus::Active {
        return Err(anyhow!("This vault entry is no longer active"));
    }
    if Utc::now() > entry.expires_at {
        expire_entry(&mut entry)?;
        return Err(anyhow!("The seven-day undo window has expired"));
    }

    let original_root = PathBuf::from(&entry.original_root);
    let payload_root = PathBuf::from(&entry.payload_root);
    let mut restored_entries = 0_u64;
    let mut skipped_entries = 0_u64;
    let mut restored_bytes = 0_u64;

    for relative_text in &entry.relative_paths {
        let relative = PathBuf::from(relative_text);
        if !is_safe_relative(&relative) {
            skipped_entries = skipped_entries.saturating_add(1);
            continue;
        }
        let source = payload_root.join(&relative);
        if !source.exists() {
            continue;
        }
        let destination = original_root.join(&relative);
        if destination.exists() {
            skipped_entries = skipped_entries.saturating_add(1);
            continue;
        }
        if let Some(parent) = destination.parent() {
            if fs::create_dir_all(parent).is_err() {
                skipped_entries = skipped_entries.saturating_add(1);
                continue;
            }
        }
        let bytes = fs::metadata(&source).map(|metadata| metadata.len()).unwrap_or_default();
        if move_file(&source, &destination).is_err() {
            skipped_entries = skipped_entries.saturating_add(1);
            continue;
        }
        restored_entries = restored_entries.saturating_add(1);
        restored_bytes = restored_bytes.saturating_add(bytes);
    }

    let remaining = entry
        .relative_paths
        .iter()
        .filter(|relative| PathBuf::from(&entry.payload_root).join(relative).exists())
        .count();
    entry.restored_at = Some(Utc::now());
    entry.status = if remaining == 0 {
        VaultStatus::Restored
    } else {
        VaultStatus::PartiallyRestored
    };
    persist_entry(&entry)?;
    if remaining == 0 {
        let _ = fs::remove_dir_all(PathBuf::from(&entry.payload_root));
    }

    Ok(RestoreResult {
        vault_entry_id: entry.id,
        restored_entries,
        skipped_entries,
        restored_bytes,
        status: entry.status,
        message: if remaining == 0 {
            format!("Restored {restored_entries} entries from the Undo Vault")
        } else {
            format!(
                "Restored {restored_entries} entries; {remaining} remain because their original paths are occupied"
            )
        },
    })
}

fn purge_expired_entries() -> Result<()> {
    let root = vault_root();
    if !root.exists() {
        return Ok(());
    }
    for directory in fs::read_dir(root)? {
        let directory = match directory {
            Ok(directory) => directory,
            Err(_) => continue,
        };
        let manifest = directory.path().join("manifest.json");
        let bytes = match fs::read(&manifest) {
            Ok(bytes) => bytes,
            Err(_) => continue,
        };
        let mut entry = match serde_json::from_slice::<VaultEntry>(&bytes) {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        if entry.status == VaultStatus::Active && Utc::now() > entry.expires_at {
            expire_entry(&mut entry)?;
        }
    }
    Ok(())
}

fn expire_entry(entry: &mut VaultEntry) -> Result<()> {
    let _ = fs::remove_dir_all(PathBuf::from(&entry.payload_root));
    entry.status = VaultStatus::Expired;
    persist_entry(entry)
}

fn load_entry(id: Uuid) -> Result<VaultEntry> {
    let bytes = fs::read(entry_root(id).join("manifest.json"))?;
    Ok(serde_json::from_slice(&bytes)?)
}

fn persist_entry(entry: &VaultEntry) -> Result<()> {
    let root = entry_root(entry.id);
    fs::create_dir_all(&root)?;
    fs::write(root.join("manifest.json"), serde_json::to_vec_pretty(entry)?)?;
    Ok(())
}

fn move_file(source: &Path, destination: &Path) -> Result<()> {
    match fs::rename(source, destination) {
        Ok(()) => Ok(()),
        Err(_) => {
            fs::copy(source, destination)?;
            fs::remove_file(source)?;
            Ok(())
        }
    }
}

fn is_safe_relative(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && path.components().all(|component| {
            !matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
}

fn entry_root(id: Uuid) -> PathBuf {
    vault_root().join(id.to_string())
}

fn vault_root() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("WinReclaim")
        .join("vault")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_parent_relative_paths() {
        assert!(!is_safe_relative(Path::new("..\\secret")));
        assert!(is_safe_relative(Path::new("folder\\file.bin")));
    }
}
