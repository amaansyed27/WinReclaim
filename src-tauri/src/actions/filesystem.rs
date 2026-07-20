use crate::domain::ActionKind;
use crate::platform::windows::{
    canonical_is_within, is_reparse_point, local_app_data, user_profile, windows_directory,
};
use crate::policy::VAULT_RETENTION_DAYS;
use crate::rules::known_targets;
use crate::vault::quarantine_files;
use anyhow::{anyhow, Result};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug)]
pub struct FilesystemOutcome {
    pub affected_entries: u64,
    pub skipped_entries: u64,
    pub message: String,
    pub vault_entry_ids: Vec<Uuid>,
}

pub fn clean_user_temp(target: &Path) -> Result<FilesystemOutcome> {
    let allowed = local_app_data()
        .ok_or_else(|| anyhow!("LOCALAPPDATA is unavailable"))?
        .join("Temp");
    validate_exact_target(target, &allowed)?;
    delete_tree(
        target,
        None,
        true,
        "temporary",
        "Windows-locked, active or inaccessible entries were skipped",
    )
}

pub fn clean_system_temp(target: &Path) -> Result<FilesystemOutcome> {
    let allowed = windows_directory()?.join("Temp");
    validate_exact_target(target, &allowed)?;
    delete_tree(
        target,
        None,
        true,
        "Windows Temp",
        "Windows-locked, active, inaccessible or administrator-protected entries were skipped",
    )
}

pub fn clean_prefetch(target: &Path) -> Result<FilesystemOutcome> {
    let allowed = windows_directory()?.join("Prefetch");
    validate_exact_target(target, &allowed)?;
    delete_tree(
        target,
        None,
        true,
        "Prefetch",
        "Windows-locked, active, inaccessible or administrator-protected entries were skipped",
    )
}

pub fn clean_generic_directory(target: &Path) -> Result<FilesystemOutcome> {
    validate_generic_target(target)?;
    delete_tree(
        target,
        None,
        true,
        "rebuildable cache or generated",
        "Locked, active or inaccessible entries were skipped",
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

fn validate_generic_target(target: &Path) -> Result<()> {
    if !target.exists() {
        return Ok(());
    }
    let metadata = fs::symlink_metadata(target)?;
    if !metadata.is_dir() || is_reparse_point(&metadata) {
        return Err(anyhow!(
            "Generic cleanup targets must be normal directories"
        ));
    }

    if is_known_generic_target(target)?
        || is_verified_project_output(target)?
        || is_safe_dynamic_cache(target)?
    {
        return Ok(());
    }

    Err(anyhow!(
        "The selected directory no longer matches a verified rebuildable cache or generated-output fingerprint"
    ))
}

fn is_known_generic_target(target: &Path) -> Result<bool> {
    Ok(known_targets()?.into_iter().any(|candidate| {
        candidate.action_kind == Some(ActionKind::GenericDirectory)
            && paths_equal(target, &candidate.path)
    }))
}

fn is_verified_project_output(target: &Path) -> Result<bool> {
    let user = user_profile()?;
    if !canonical_is_within(target, &user)? || paths_equal(target, &user) {
        return Ok(false);
    }
    let Some(name) = target
        .file_name()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
    else {
        return Ok(false);
    };
    let parent = target.parent().unwrap_or(target);
    let valid = match name.as_str() {
        "node_modules" => has_any_marker(
            parent,
            &[
                "package.json",
                "package-lock.json",
                "pnpm-lock.yaml",
                "yarn.lock",
                "bun.lock",
                "bun.lockb",
            ],
        ),
        "target" => parent.join("Cargo.toml").is_file(),
        ".venv" | "venv" => target.join("pyvenv.cfg").is_file(),
        "dist" | "build" | ".next" | ".nuxt" | "out" | "coverage" => has_project_marker(parent),
        _ => false,
    };
    Ok(valid)
}

fn is_safe_dynamic_cache(target: &Path) -> Result<bool> {
    let user = user_profile()?;
    if !canonical_is_within(target, &user)? || paths_equal(target, &user) {
        return Ok(false);
    }
    let depth = target
        .strip_prefix(&user)
        .map(|relative| relative.components().count())
        .unwrap_or_default();
    if depth < 2 {
        return Ok(false);
    }

    let Some(name) = target.file_name().and_then(|value| value.to_str()) else {
        return Ok(false);
    };
    if !is_cache_like_name(name) || has_protected_component(target) {
        return Ok(false);
    }

    for candidate in known_targets()? {
        if candidate.action_kind != Some(ActionKind::GenericDirectory)
            && paths_overlap(target, &candidate.path)
        {
            return Ok(false);
        }
    }
    Ok(true)
}

fn is_cache_like_name(name: &str) -> bool {
    let lower = name.trim().to_ascii_lowercase();
    matches!(
        lower.as_str(),
        "cache"
            | "caches"
            | "cache2"
            | "tmp"
            | "temp"
            | "computecache"
            | "gpucache"
            | "shadercache"
            | "code cache"
            | "media cache"
            | "preview cache"
            | "thumbnail cache"
            | "node-compile-cache"
    ) || lower.ends_with("-cache")
        || lower.ends_with("_cache")
}

fn has_protected_component(path: &Path) -> bool {
    path.components().any(|component| {
        matches!(
            component
                .as_os_str()
                .to_string_lossy()
                .to_ascii_lowercase()
                .as_str(),
            ".git"
                | ".hg"
                | ".svn"
                | ".ollama"
                | "models"
                | "model"
                | "checkpoints"
                | "weights"
                | "user data"
                | "profiles"
                | "extensions"
                | "vault"
                | "snapshots"
        )
    })
}

fn has_project_marker(directory: &Path) -> bool {
    has_any_marker(
        directory,
        &[
            "package.json",
            "Cargo.toml",
            "pyproject.toml",
            "requirements.txt",
            "CMakeLists.txt",
            "pubspec.yaml",
            "build.gradle",
            "build.gradle.kts",
            "settings.gradle",
            "settings.gradle.kts",
        ],
    )
}

fn has_any_marker(directory: &Path, names: &[&str]) -> bool {
    names.iter().any(|name| directory.join(name).is_file())
}

fn paths_equal(left: &Path, right: &Path) -> bool {
    match (left.canonicalize(), right.canonicalize()) {
        (Ok(left), Ok(right)) => left == right,
        _ => left
            .to_string_lossy()
            .trim_end_matches(['\\', '/'])
            .eq_ignore_ascii_case(right.to_string_lossy().trim_end_matches(['\\', '/'])),
    }
}

fn paths_overlap(left: &Path, right: &Path) -> bool {
    let left = left.canonicalize().unwrap_or_else(|_| left.to_path_buf());
    let right = right.canonicalize().unwrap_or_else(|_| right.to_path_buf());
    left.starts_with(&right) || right.starts_with(&left)
}

#[allow(clippy::too_many_arguments)]
fn quarantine_tree(
    root: &Path,
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

    let (files, mut directories, discovery_skipped) = collect_files(root, allowed_extensions);
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
        format!("compressed {VAULT_RETENTION_DAYS}-day Undo Vault")
    } else {
        format!("{VAULT_RETENTION_DAYS}-day Undo Vault")
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

fn delete_tree(
    root: &Path,
    allowed_extensions: Option<&[&str]>,
    remove_empty_directories: bool,
    noun: &str,
    skip_reason: &str,
) -> Result<FilesystemOutcome> {
    if !root.exists() {
        return Ok(FilesystemOutcome {
            affected_entries: 0,
            skipped_entries: 0,
            message: "The cleanup location no longer exists".to_string(),
            vault_entry_ids: Vec::new(),
        });
    }

    let (files, mut directories, discovery_skipped) = collect_files(root, allowed_extensions);
    let mut removed_files = 0_u64;
    let mut skipped_entries = discovery_skipped;
    for file in files {
        match fs::remove_file(file) {
            Ok(()) => removed_files = removed_files.saturating_add(1),
            Err(_) => skipped_entries = skipped_entries.saturating_add(1),
        }
    }

    let mut removed_directories = 0_u64;
    if remove_empty_directories {
        directories.sort_by_key(|path| std::cmp::Reverse(path.components().count()));
        for directory in directories {
            if directory != root {
                match fs::remove_dir(&directory) {
                    Ok(()) => removed_directories = removed_directories.saturating_add(1),
                    Err(error) if error.kind() == std::io::ErrorKind::DirectoryNotEmpty => {}
                    Err(_) => skipped_entries = skipped_entries.saturating_add(1),
                }
            }
        }
    }
    let affected_entries = removed_files.saturating_add(removed_directories);
    let message = if affected_entries > 0 {
        format!(
            "Removed {removed_files} {noun} files and {removed_directories} empty folders; skipped {skipped_entries}. {skip_reason}. This action is not stored in Restore files."
        )
    } else if skipped_entries > 0 {
        format!("No {noun} entries were removed; skipped {skipped_entries}. {skip_reason}.")
    } else {
        format!("No {noun} entries were available")
    };

    Ok(FilesystemOutcome {
        affected_entries,
        skipped_entries,
        message,
        vault_entry_ids: Vec::new(),
    })
}

fn collect_files(
    root: &Path,
    allowed_extensions: Option<&[&str]>,
) -> (Vec<PathBuf>, Vec<PathBuf>, u64) {
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
    fn refuses_arbitrary_temp_target() {
        let arbitrary =
            std::env::temp_dir().join(format!("winreclaim-arbitrary-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&arbitrary).unwrap();
        assert!(clean_user_temp(&arbitrary).is_err());
        fs::remove_dir_all(arbitrary).unwrap();
    }

    #[test]
    fn recognises_portable_cache_names() {
        for name in [
            "Cache",
            "ComputeCache",
            "shader-cache",
            "node-compile-cache",
            "tmp",
        ] {
            assert!(is_cache_like_name(name));
        }
        for name in ["models", "User Data", "Documents", "plugins"] {
            assert!(!is_cache_like_name(name));
        }
    }
}
