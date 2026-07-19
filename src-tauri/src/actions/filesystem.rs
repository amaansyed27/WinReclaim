use crate::platform::windows::{canonical_is_within, is_reparse_point, local_app_data};
use crate::vault::quarantine_files;
use anyhow::{anyhow, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use uuid::Uuid;

const TEMP_MINIMUM_AGE: Duration = Duration::from_secs(7 * 24 * 60 * 60);

#[derive(Debug)]
pub struct FilesystemOutcome {
    pub affected_entries: u64,
    pub skipped_entries: u64,
    pub message: String,
    pub vault_entry_ids: Vec<Uuid>,
}

pub fn quarantine_user_temp(
    target: &Path,
    receipt_id: Uuid,
    finding_id: Uuid,
    display_name: &str,
) -> Result<FilesystemOutcome> {
    let allowed = local_app_data()
        .ok_or_else(|| anyhow!("LOCALAPPDATA is unavailable"))?
        .join("Temp");
    validate_exact_target(target, &allowed)?;
    quarantine_tree(
        target,
        Some(TEMP_MINIMUM_AGE),
        None,
        receipt_id,
        finding_id,
        display_name,
        "stale temporary",
    )
}

pub fn quarantine_crash_dumps(
    target: &Path,
    receipt_id: Uuid,
    finding_id: Uuid,
    display_name: &str,
) -> Result<FilesystemOutcome> {
    let allowed = local_app_data()
        .ok_or_else(|| anyhow!("LOCALAPPDATA is unavailable"))?
        .join("CrashDumps");
    validate_exact_target(target, &allowed)?;
    quarantine_tree(
        target,
        None,
        Some(&["dmp", "mdmp", "wer"]),
        receipt_id,
        finding_id,
        display_name,
        "recognised crash dump",
    )
}

fn validate_exact_target(target: &Path, allowed: &Path) -> Result<()> {
    if !target.exists() {
        return Ok(());
    }
    if !allowed.exists() {
        return Err(anyhow!("Allowed cleanup root does not exist"));
    }
    if !canonical_is_within(target, allowed)? || target.canonicalize()? != allowed.canonicalize()? {
        return Err(anyhow!("Cleanup target failed protected-root validation"));
    }
    if is_reparse_point(&fs::symlink_metadata(target)?) {
        return Err(anyhow!("Cleanup targets cannot be reparse points"));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn quarantine_tree(
    root: &Path,
    minimum_age: Option<Duration>,
    allowed_extensions: Option<&[&str]>,
    receipt_id: Uuid,
    finding_id: Uuid,
    display_name: &str,
    noun: &str,
) -> Result<FilesystemOutcome> {
    if !root.exists() {
        return Ok(FilesystemOutcome {
            affected_entries: 0,
            skipped_entries: 0,
            message: "The cleanup location no longer exists".to_string(),
            vault_entry_ids: Vec::new(),
        });
    }

    let (files, mut directories, discovery_skipped) =
        collect_eligible_files(root, minimum_age, allowed_extensions);
    let outcome = quarantine_files(root, &files, receipt_id, finding_id, display_name)?;
    let mut removed_directories = 0_u64;
    directories.sort_by_key(|path| std::cmp::Reverse(path.components().count()));
    for directory in directories {
        if directory != root && fs::remove_dir(&directory).is_ok() {
            removed_directories = removed_directories.saturating_add(1);
        }
    }

    let vault_entry_ids = outcome
        .entry
        .as_ref()
        .map(|entry| vec![entry.id])
        .unwrap_or_default();
    let skipped_entries = discovery_skipped.saturating_add(outcome.skipped_entries);
    let affected_entries = outcome.moved_entries.saturating_add(removed_directories);
    let vault_description = if outcome.compressed {
        "compressed seven-day Undo Vault"
    } else {
        "seven-day Undo Vault"
    };

    Ok(FilesystemOutcome {
        affected_entries,
        skipped_entries,
        message: if outcome.moved_entries > 0 {
            format!(
                "Moved {} {noun} files into the {vault_description}; skipped {skipped_entries}. Actual reclaimed space is measured after compression.",
                outcome.moved_entries
            )
        } else {
            format!("No eligible {noun} files were available; skipped {skipped_entries}")
        },
        vault_entry_ids,
    })
}

fn collect_eligible_files(
    root: &Path,
    minimum_age: Option<Duration>,
    allowed_extensions: Option<&[&str]>,
) -> (Vec<PathBuf>, Vec<PathBuf>, u64) {
    let now = SystemTime::now();
    let mut files = Vec::new();
    let mut directories = Vec::new();
    let mut stack = vec![PathBuf::from(root)];
    let mut skipped = 0_u64;

    while let Some(path) = stack.pop() {
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(_) => {
                skipped = skipped.saturating_add(1);
                continue;
            }
        };
        if is_reparse_point(&metadata) {
            skipped = skipped.saturating_add(1);
            continue;
        }
        if metadata.is_file() {
            if let Some(extensions) = allowed_extensions {
                let extension = path
                    .extension()
                    .and_then(|value| value.to_str())
                    .unwrap_or_default()
                    .to_ascii_lowercase();
                if !extensions.iter().any(|candidate| *candidate == extension) {
                    continue;
                }
            }
            if let Some(age) = minimum_age {
                let modified = match metadata.modified() {
                    Ok(value) => value,
                    Err(_) => {
                        skipped = skipped.saturating_add(1);
                        continue;
                    }
                };
                if now.duration_since(modified).unwrap_or_default() < age {
                    continue;
                }
            }
            files.push(path);
        } else if metadata.is_dir() {
            directories.push(path.clone());
            match fs::read_dir(path) {
                Ok(entries) => {
                    for entry in entries {
                        match entry {
                            Ok(entry) => stack.push(entry.path()),
                            Err(_) => skipped = skipped.saturating_add(1),
                        }
                    }
                }
                Err(_) => skipped = skipped.saturating_add(1),
            }
        }
    }

    (files, directories, skipped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_arbitrary_target() {
        let arbitrary =
            std::env::temp_dir().join(format!("winreclaim-arbitrary-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&arbitrary).unwrap();
        assert!(
            quarantine_user_temp(&arbitrary, Uuid::new_v4(), Uuid::new_v4(), "Fixture").is_err()
        );
        fs::remove_dir_all(arbitrary).unwrap();
    }
}
