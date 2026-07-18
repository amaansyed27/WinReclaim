use crate::platform::windows::{canonical_is_within, is_reparse_point, local_app_data};
use anyhow::{anyhow, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

const TEMP_MINIMUM_AGE: Duration = Duration::from_secs(7 * 24 * 60 * 60);

pub fn clean_user_temp(target: &Path) -> Result<(u64, u64, String)> {
    let allowed = local_app_data()
        .ok_or_else(|| anyhow!("LOCALAPPDATA is unavailable"))?
        .join("Temp");
    validate_exact_target(target, &allowed)?;
    let (deleted, skipped) = clean_tree(target, Some(TEMP_MINIMUM_AGE), None)?;
    Ok((
        deleted,
        skipped,
        format!("Removed {deleted} stale temporary entries; skipped {skipped}"),
    ))
}

pub fn clean_crash_dumps(target: &Path) -> Result<(u64, u64, String)> {
    let allowed = local_app_data()
        .ok_or_else(|| anyhow!("LOCALAPPDATA is unavailable"))?
        .join("CrashDumps");
    validate_exact_target(target, &allowed)?;
    let (deleted, skipped) = clean_tree(target, None, Some(&["dmp", "mdmp", "wer"]))?;
    Ok((
        deleted,
        skipped,
        format!("Removed {deleted} recognised crash dumps; skipped {skipped}"),
    ))
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

fn clean_tree(
    root: &Path,
    minimum_age: Option<Duration>,
    allowed_extensions: Option<&[&str]>,
) -> Result<(u64, u64)> {
    if !root.exists() {
        return Ok((0, 0));
    }
    let now = SystemTime::now();
    let mut files = Vec::new();
    let mut directories = Vec::new();
    let mut stack = vec![PathBuf::from(root)];
    let mut skipped = 0_u64;
    while let Some(path) = stack.pop() {
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(_) => {
                skipped += 1;
                continue;
            }
        };
        if is_reparse_point(&metadata) {
            skipped += 1;
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
                        skipped += 1;
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
                            Err(_) => skipped += 1,
                        }
                    }
                }
                Err(_) => skipped += 1,
            }
        }
    }
    let mut deleted = 0_u64;
    for file in files {
        match fs::remove_file(file) {
            Ok(()) => deleted += 1,
            Err(_) => skipped += 1,
        }
    }
    directories.sort_by_key(|path| std::cmp::Reverse(path.components().count()));
    for directory in directories {
        if directory != root && fs::remove_dir(&directory).is_ok() {
            deleted += 1;
        }
    }
    Ok((deleted, skipped))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_arbitrary_target() {
        let arbitrary =
            std::env::temp_dir().join(format!("winreclaim-arbitrary-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&arbitrary).unwrap();
        assert!(clean_user_temp(&arbitrary).is_err());
        fs::remove_dir_all(arbitrary).unwrap();
    }
}
