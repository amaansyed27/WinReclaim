use super::{drive_root, local_app_data};
use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

pub fn user_temp() -> Result<PathBuf> {
    std::env::var_os("TEMP")
        .map(PathBuf::from)
        .or_else(|| local_app_data().map(|path| path.join("Temp")))
        .ok_or_else(|| anyhow!("Unable to resolve %TEMP%"))
}

pub fn windows_directory() -> Result<PathBuf> {
    std::env::var_os("WINDIR")
        .or_else(|| std::env::var_os("SystemRoot"))
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("Unable to resolve the Windows directory"))
}

pub fn system_drive_root() -> Result<PathBuf> {
    if let Some(value) = std::env::var_os("SystemDrive") {
        let text = value.to_string_lossy();
        let trimmed = text.trim_end_matches(['\\', '/']);
        return Ok(PathBuf::from(format!("{trimmed}\\")));
    }
    Ok(drive_root(&windows_directory()?))
}

pub fn program_data() -> Result<PathBuf> {
    std::env::var_os("ProgramData")
        .map(PathBuf::from)
        .map(Ok)
        .unwrap_or_else(|| Ok(system_drive_root()?.join("ProgramData")))
}

pub fn system_cache_roots() -> Result<Vec<PathBuf>> {
    Ok(vec![windows_directory()?, program_data()?])
}

#[cfg(windows)]
fn wide_null(path: &Path) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    path.as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

#[cfg(windows)]
pub fn recycle_bin_snapshot() -> Result<(u64, u64)> {
    use std::mem::size_of;
    use windows_sys::Win32::UI::Shell::{SHQueryRecycleBinW, SHQUERYRBINFO};

    let root = system_drive_root()?;
    let wide = wide_null(&root);
    let mut info = SHQUERYRBINFO {
        cbSize: size_of::<SHQUERYRBINFO>() as u32,
        i64Size: 0,
        i64NumItems: 0,
    };
    let result = unsafe { SHQueryRecycleBinW(wide.as_ptr(), &mut info) };
    if result < 0 {
        return Err(anyhow!(
            "Windows could not query the Recycle Bin (HRESULT 0x{:08X})",
            result as u32
        ));
    }
    Ok((info.i64Size.max(0) as u64, info.i64NumItems.max(0) as u64))
}

#[cfg(not(windows))]
pub fn recycle_bin_snapshot() -> Result<(u64, u64)> {
    Ok((0, 0))
}

#[cfg(windows)]
pub fn empty_recycle_bin() -> Result<()> {
    use windows_sys::Win32::UI::Shell::{
        SHEmptyRecycleBinW, SHERB_NOCONFIRMATION, SHERB_NOPROGRESSUI, SHERB_NOSOUND,
    };

    let root = system_drive_root()?;
    let wide = wide_null(&root);
    let flags = SHERB_NOCONFIRMATION | SHERB_NOPROGRESSUI | SHERB_NOSOUND;
    let result = unsafe { SHEmptyRecycleBinW(std::ptr::null_mut(), wide.as_ptr(), flags) };
    if result < 0 {
        return Err(anyhow!(
            "Windows could not empty the Recycle Bin (HRESULT 0x{:08X})",
            result as u32
        ));
    }
    Ok(())
}

#[cfg(not(windows))]
pub fn empty_recycle_bin() -> Result<()> {
    Err(anyhow!("Recycle Bin cleanup is available only on Windows"))
}
