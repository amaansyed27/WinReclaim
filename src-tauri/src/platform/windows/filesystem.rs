use crate::domain::{DiskSnapshot, DriveInfo, DriveKind};
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

#[cfg(windows)]
pub fn list_drives() -> Result<Vec<DriveInfo>> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        GetDriveTypeW, GetLogicalDrives, GetVolumeInformationW, DRIVE_CDROM, DRIVE_FIXED,
        DRIVE_RAMDISK, DRIVE_REMOTE, DRIVE_REMOVABLE,
    };

    let system_root = std::env::var_os("SystemDrive")
        .map(|value| {
            PathBuf::from(format!(
                "{}\\",
                value.to_string_lossy().trim_end_matches(['\\', '/'])
            ))
        })
        .unwrap_or_else(|| drive_root(&user_profile().unwrap_or_else(|_| PathBuf::from("C:\\"))));
    let mask = unsafe { GetLogicalDrives() };
    if mask == 0 {
        return Err(anyhow!("Windows did not return any logical drives"));
    }

    let mut drives = Vec::new();
    for index in 0..26_u32 {
        if mask & (1_u32 << index) == 0 {
            continue;
        }
        let letter = (b'A' + index as u8) as char;
        let root = PathBuf::from(format!("{letter}:\\"));
        let wide = root
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect::<Vec<_>>();
        let drive_type = unsafe { GetDriveTypeW(wide.as_ptr()) };
        let kind = match drive_type {
            DRIVE_FIXED => DriveKind::Fixed,
            DRIVE_REMOVABLE => DriveKind::Removable,
            DRIVE_REMOTE => DriveKind::Network,
            DRIVE_CDROM => DriveKind::Optical,
            DRIVE_RAMDISK => DriveKind::RamDisk,
            _ => DriveKind::Other,
        };
        if matches!(kind, DriveKind::Optical | DriveKind::Other) {
            continue;
        }

        let total = match fs2::total_space(&root) {
            Ok(value) if value > 0 => value,
            _ => continue,
        };
        let free = fs2::available_space(&root).unwrap_or_default();
        let mut label_buffer = [0_u16; 261];
        let mut file_system_buffer = [0_u16; 64];
        let mut serial = 0_u32;
        let mut max_component = 0_u32;
        let mut flags = 0_u32;
        let metadata_ok = unsafe {
            GetVolumeInformationW(
                wide.as_ptr(),
                label_buffer.as_mut_ptr(),
                label_buffer.len() as u32,
                &mut serial,
                &mut max_component,
                &mut flags,
                file_system_buffer.as_mut_ptr(),
                file_system_buffer.len() as u32,
            )
        } != 0;

        let label = if metadata_ok {
            wide_string(&label_buffer)
        } else {
            String::new()
        };
        let file_system = if metadata_ok {
            wide_string(&file_system_buffer)
        } else {
            String::new()
        };
        let volume_id = if metadata_ok {
            format!("{serial:08X}")
        } else {
            root.to_string_lossy().to_ascii_uppercase()
        };

        drives.push(DriveInfo {
            root: root.to_string_lossy().to_string(),
            label,
            file_system,
            volume_id,
            total_bytes: total,
            free_bytes: free,
            used_bytes: total.saturating_sub(free),
            is_system: same_drive(&root, &system_root),
            kind,
        });
    }

    drives.sort_by_key(|drive| (!drive.is_system, drive.root.clone()));
    Ok(drives)
}

#[cfg(windows)]
fn wide_string(buffer: &[u16]) -> String {
    let length = buffer
        .iter()
        .position(|value| *value == 0)
        .unwrap_or(buffer.len());
    String::from_utf16_lossy(&buffer[..length])
}

#[cfg(not(windows))]
pub fn list_drives() -> Result<Vec<DriveInfo>> {
    let snapshot = disk_snapshot(Path::new("/"))?;
    Ok(vec![DriveInfo {
        root: snapshot.root.clone(),
        label: "Root".to_string(),
        file_system: String::new(),
        volume_id: snapshot.root.clone(),
        total_bytes: snapshot.total_bytes,
        free_bytes: snapshot.free_bytes,
        used_bytes: snapshot.used_bytes,
        is_system: true,
        kind: DriveKind::Fixed,
    }])
}

pub fn same_drive(left: &Path, right: &Path) -> bool {
    drive_root(left)
        .to_string_lossy()
        .eq_ignore_ascii_case(&drive_root(right).to_string_lossy())
}

pub fn canonical_is_within(path: &Path, allowed_root: &Path) -> Result<bool> {
    let canonical_path = path.canonicalize()?;
    let canonical_root = allowed_root.canonicalize()?;
    Ok(canonical_path.starts_with(canonical_root))
}
