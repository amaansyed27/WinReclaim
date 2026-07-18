use crate::domain::DiskSnapshot;
use anyhow::{anyhow, Result};
use std::fs::Metadata;
use std::path::{Path, PathBuf};

#[cfg(windows)]
use std::os::windows::fs::MetadataExt;

const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0400;

pub fn is_reparse_point(metadata: &Metadata) -> bool {
    if metadata.file_type().is_symlink() {
        return true;
    }

    #[cfg(windows)]
    {
        metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
    }

    #[cfg(not(windows))]
    {
        false
    }
}

pub fn user_profile() -> Result<PathBuf> {
    std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(PathBuf::from))
        .ok_or_else(|| anyhow!("Unable to resolve the current user profile"))
}

pub fn local_app_data() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA").map(PathBuf::from)
}

pub fn roaming_app_data() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(PathBuf::from)
}

pub fn drive_root(path: &Path) -> PathBuf {
    #[cfg(windows)]
    {
        use std::path::Component;
        if let Some(Component::Prefix(prefix)) = path.components().next() {
            return PathBuf::from(format!("{}\\", prefix.as_os_str().to_string_lossy()));
        }
    }

    PathBuf::from("/")
}

pub fn disk_snapshot(path: &Path) -> Result<DiskSnapshot> {
    let root = drive_root(path);
    let total = fs2::total_space(&root)?;
    let free = fs2::available_space(&root)?;
    Ok(DiskSnapshot {
        root: root.to_string_lossy().to_string(),
        total_bytes: total,
        free_bytes: free,
        used_bytes: total.saturating_sub(free),
    })
}

pub fn canonical_is_within(path: &Path, allowed_root: &Path) -> Result<bool> {
    let canonical_path = path.canonicalize()?;
    let canonical_root = allowed_root.canonicalize()?;
    Ok(canonical_path.starts_with(canonical_root))
}
