from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def write(path: str, content: str) -> None:
    target = ROOT / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(content, encoding="utf-8", newline="\n")


def replace_once(path: str, old: str, new: str) -> None:
    content = read(path)
    count = content.count(old)
    if count != 1:
        raise RuntimeError(f"Expected one match in {path}, found {count}: {old[:100]!r}")
    write(path, content.replace(old, new, 1))


# Rust dependency for native Recycle Bin inspection and emptying.
replace_once(
    "src-tauri/Cargo.toml",
    'walkdir = "2"\n',
    'walkdir = "2"\nwindows-sys = { version = "0.61", features = ["Win32_Foundation", "Win32_UI_Shell"] }\n',
)

# Platform helpers and native Shell API adapter.
replace_once(
    "src-tauri/src/platform/windows/mod.rs",
    "mod commands;\nmod filesystem;\n\npub use commands::*;\npub use filesystem::*;\n",
    "mod commands;\nmod filesystem;\nmod system_storage;\n\npub use commands::*;\npub use filesystem::*;\npub use system_storage::*;\n",
)

write(
    "src-tauri/src/platform/windows/system_storage.rs",
    r'''use super::{drive_root, local_app_data};
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
        let trimmed = text.trim_end_matches(|character| character == '\\' || character == '/');
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
    path.as_os_str().encode_wide().chain(std::iter::once(0)).collect()
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
''',
)

# Domain additions.
replace_once(
    "src-tauri/src/domain/types.rs",
    "    UserTemp,\n    CrashDumps,\n    HuggingfacePrune,\n",
    "    UserTemp,\n    SystemTemp,\n    Prefetch,\n    RecycleBin,\n    CrashDumps,\n    HuggingfacePrune,\n",
)
replace_once(
    "src-tauri/src/domain/types.rs",
    "    #[serde(default = \"default_true\")]\n    pub include_app_data: bool,\n    #[serde(default = \"default_minimum_finding_bytes\")]\n",
    "    #[serde(default = \"default_true\")]\n    pub include_app_data: bool,\n    #[serde(default = \"default_true\")]\n    pub include_system_drive_caches: bool,\n    #[serde(default = \"default_minimum_finding_bytes\")]\n",
)
replace_once(
    "src-tauri/src/domain/types.rs",
    "            discover_unknown: true,\n            include_app_data: true,\n            minimum_finding_bytes: default_minimum_finding_bytes(),\n",
    "            discover_unknown: true,\n            include_app_data: true,\n            include_system_drive_caches: true,\n            minimum_finding_bytes: default_minimum_finding_bytes(),\n",
)

# Rules: classic locations plus verified system-drive cache candidates.
replace_once(
    "src-tauri/src/rules/catalog.rs",
    "use crate::platform::windows::{local_app_data, roaming_app_data, user_profile};\n",
    "use crate::platform::windows::{\n    local_app_data, program_data, roaming_app_data, system_drive_root, user_profile, user_temp,\n    windows_directory,\n};\n",
)
replace_once(
    "src-tauri/src/rules/catalog.rs",
    'pub const RULE_SET_VERSION: &str = "2026.07-alpha.1";\n',
    'pub const RULE_SET_VERSION: &str = "2026.07-alpha.2";\n',
)
replace_once(
    "src-tauri/src/rules/catalog.rs",
    "    let roaming = roaming_app_data().unwrap_or_else(|| user.join(\"AppData\").join(\"Roaming\"));\n\n    Ok(vec![\n",
    "    let roaming = roaming_app_data().unwrap_or_else(|| user.join(\"AppData\").join(\"Roaming\"));\n    let windows = windows_directory()?;\n    let system_drive = system_drive_root()?;\n    let program_data = program_data()?;\n\n    Ok(vec![\n",
)
replace_once(
    "src-tauri/src/rules/catalog.rs",
    '''        target(
            ("windows.user_temp", "User temporary files", "Windows"),
            local.join("Temp"),
            RiskClass::SafeNow,
            "Temporary files created by applications for short-lived work.",
            "Only files older than seven days are eligible. Active and locked files are skipped.",
            Some(ActionKind::UserTemp),
        ),
''',
    '''        target(
            ("windows.user_temp", "User Temp (%TEMP%)", "Classic Windows cleanup"),
            user_temp()?,
            RiskClass::SafeNow,
            "The current user's %TEMP% folder used by applications for short-lived work.",
            "Only files older than seven days are eligible. Active and locked files are skipped and eligible files enter the compressed Undo Vault.",
            Some(ActionKind::UserTemp),
        ),
        target(
            ("system_drive.windows_temp", "Windows Temp", "Classic Windows cleanup"),
            windows.join("Temp"),
            RiskClass::SafeNow,
            "Machine-level temporary files under the active Windows installation.",
            "Only unlocked files older than seven days are removed. This exact-root action may require administrator rights and is not reversible.",
            Some(ActionKind::SystemTemp),
        ),
        target(
            ("system_drive.prefetch", "Windows Prefetch", "Classic Windows cleanup"),
            windows.join("Prefetch"),
            RiskClass::ReviewFirst,
            "Windows launch traces used to optimise application and boot startup.",
            "Only .pf files are removed after explicit review. Windows rebuilds them, and launches may be temporarily slower. Administrator rights may be required.",
            Some(ActionKind::Prefetch),
        ),
        target(
            ("system_drive.recycle_bin", "Recycle Bin", "Classic Windows cleanup"),
            system_drive.join("$Recycle.Bin"),
            RiskClass::ReviewFirst,
            "Deleted items retained by Windows on the drive containing the active Windows installation.",
            "The native Windows Shell API permanently empties this drive's Recycle Bin. This cannot be undone through WinReclaim.",
            Some(ActionKind::RecycleBin),
        ),
        target(
            ("system_drive.windows_update_download", "Windows Update download cache", "System-drive cache"),
            windows.join("SoftwareDistribution").join("Download"),
            RiskClass::ReviewFirst,
            "Downloaded Windows Update packages and staging data.",
            "Inspection only. Servicing state must be managed through Windows Update or supported maintenance tools, not raw deletion.",
            None,
        ),
        target(
            ("system_drive.delivery_optimization", "Delivery Optimization cache", "System-drive cache"),
            program_data
                .join("Microsoft")
                .join("Windows")
                .join("DeliveryOptimization")
                .join("Cache"),
            RiskClass::ReviewFirst,
            "Locally cached Windows delivery content used for update distribution.",
            "Inspection only. WinReclaim does not bypass the Windows Delivery Optimization service.",
            None,
        ),
        target(
            ("system_drive.package_cache", "Installer package cache", "System-drive cache"),
            program_data.join("Package Cache"),
            RiskClass::Protected,
            "Installer payloads that applications may require for repair, update or uninstall operations.",
            "Protected. Raw deletion can break repair, update and uninstall workflows.",
            None,
        ),
        target(
            ("system_drive.wer_archive", "Windows Error Reporting archive", "System-drive cache"),
            program_data
                .join("Microsoft")
                .join("Windows")
                .join("WER")
                .join("ReportArchive"),
            RiskClass::ReviewFirst,
            "Archived system-level Windows Error Reporting data.",
            "Inspection only in this release because reports may still be needed for diagnostics.",
            None,
        ),
        target(
            ("system_drive.vendor_amd", "AMD installer files", "Driver installer cache"),
            system_drive.join("AMD"),
            RiskClass::ReviewFirst,
            "Top-level AMD driver installer extraction data on the Windows drive.",
            "Inspection only. Confirm that no driver installation or rollback depends on these files.",
            None,
        ),
        target(
            ("system_drive.vendor_nvidia", "NVIDIA installer files", "Driver installer cache"),
            system_drive.join("NVIDIA"),
            RiskClass::ReviewFirst,
            "Top-level NVIDIA driver installer extraction data on the Windows drive.",
            "Inspection only. Confirm that no driver installation or rollback depends on these files.",
            None,
        ),
        target(
            ("system_drive.vendor_intel", "Intel installer files", "Driver installer cache"),
            system_drive.join("Intel"),
            RiskClass::ReviewFirst,
            "Top-level Intel driver installer extraction data on the Windows drive.",
            "Inspection only. Confirm that no driver installation or rollback depends on these files.",
            None,
        ),
''',
)

# Sizing helpers.
replace_once(
    "src-tauri/src/scanner/sizing.rs",
    "use crate::platform::windows::is_reparse_point;\n",
    "use crate::platform::windows::{is_reparse_point, recycle_bin_snapshot};\n",
)
replace_once(
    "src-tauri/src/scanner/sizing.rs",
    '''pub fn recognised_dump_size(path: &Path, cancel: &AtomicBool) -> SizeStats {
    walk_size(path, cancel, None, Some(&["dmp", "mdmp", "wer"]))
}

''',
    '''pub fn recognised_dump_size(path: &Path, cancel: &AtomicBool) -> SizeStats {
    walk_size(path, cancel, None, Some(&["dmp", "mdmp", "wer"]))
}

pub fn prefetch_size(path: &Path, cancel: &AtomicBool) -> SizeStats {
    walk_size(path, cancel, None, Some(&["pf"]))
}

pub fn recycle_bin_size() -> SizeStats {
    recycle_bin_snapshot()
        .map(|(bytes, entries)| SizeStats {
            bytes,
            entries,
            skipped: 0,
        })
        .unwrap_or_default()
}

''',
)

# Bounded system-drive cache discovery.
replace_once(
    "src-tauri/src/scanner/mod.rs",
    "mod sizing;\n\npub use profile::scan_profile;\npub use sizing::directory_size;\n",
    "mod sizing;\nmod system_cache;\n\npub use profile::scan_profile;\npub use sizing::{directory_size, recycle_bin_size};\n",
)

write(
    "src-tauri/src/scanner/system_cache.rs",
    r'''use super::sizing::directory_size;
use crate::domain::{Confidence, Finding, RiskClass};
use crate::platform::windows::is_reparse_point;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use uuid::Uuid;
use walkdir::WalkDir;

#[derive(Debug, Clone, Copy)]
pub struct SystemCacheOptions {
    pub max_depth: usize,
    pub max_entries: u64,
    pub minimum_bytes: u64,
    pub max_findings: usize,
}

#[derive(Debug, Default)]
pub struct SystemCacheResult {
    pub findings: Vec<Finding>,
    pub scanned_entries: u64,
    pub skipped_entries: u64,
    pub discovered_bytes: u64,
    pub entry_limit_reached: bool,
}

pub fn discover_system_drive_caches(
    roots: &[PathBuf],
    excluded_roots: &[PathBuf],
    cancel: &AtomicBool,
    options: SystemCacheOptions,
) -> SystemCacheResult {
    let mut result = SystemCacheResult::default();

    'roots: for root in roots {
        if !root.exists() || !root.is_dir() {
            continue;
        }
        let mut walker = WalkDir::new(root)
            .max_depth(options.max_depth)
            .follow_links(false)
            .into_iter();

        while let Some(entry) = walker.next() {
            if cancel.load(Ordering::Relaxed) {
                break 'roots;
            }
            if result.scanned_entries >= options.max_entries {
                result.entry_limit_reached = true;
                break 'roots;
            }

            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => {
                    result.skipped_entries = result.skipped_entries.saturating_add(1);
                    continue;
                }
            };
            result.scanned_entries = result.scanned_entries.saturating_add(1);

            if entry.depth() == 0 || !entry.file_type().is_dir() {
                continue;
            }
            if is_excluded(entry.path(), excluded_roots) {
                walker.skip_current_dir();
                continue;
            }

            let metadata = match entry.path().symlink_metadata() {
                Ok(metadata) => metadata,
                Err(_) => {
                    result.skipped_entries = result.skipped_entries.saturating_add(1);
                    continue;
                }
            };
            if is_reparse_point(&metadata) {
                result.skipped_entries = result.skipped_entries.saturating_add(1);
                walker.skip_current_dir();
                continue;
            }

            let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
            if should_block(&name) {
                walker.skip_current_dir();
                continue;
            }
            if !looks_like_cache(&name) {
                continue;
            }

            walker.skip_current_dir();
            let stats = directory_size(entry.path(), cancel);
            result.scanned_entries = result.scanned_entries.saturating_add(stats.entries);
            result.skipped_entries = result.skipped_entries.saturating_add(stats.skipped);
            if stats.bytes < options.minimum_bytes {
                continue;
            }

            let confidence = if is_exact_cache_name(&name) {
                Confidence::Medium
            } else {
                Confidence::Low
            };
            result.findings.push(Finding {
                id: Uuid::new_v4(),
                rule_id: "dynamic.system_drive_cache".to_string(),
                display_name: format!("{} (system-drive cache candidate)", entry.file_name().to_string_lossy()),
                category: "System-drive cache candidate".to_string(),
                path: entry.path().to_string_lossy().to_string(),
                estimated_bytes: stats.bytes,
                risk_class: RiskClass::ReviewFirst,
                explanation: "Discovered by bounded cache-name analysis under the active Windows and ProgramData roots. It does not match a verified cleanup rule.".to_string(),
                consequence: "Inspection only. Confirm the owning Windows component or application before removing anything.".to_string(),
                confidence,
                action_kind: None,
                action_available: false,
                selected_by_default: false,
            });
        }
    }

    result
        .findings
        .sort_by_key(|finding| std::cmp::Reverse(finding.estimated_bytes));
    result.findings.truncate(options.max_findings);
    result.discovered_bytes = result
        .findings
        .iter()
        .map(|finding| finding.estimated_bytes)
        .sum();
    result
}

fn is_excluded(path: &Path, excluded_roots: &[PathBuf]) -> bool {
    excluded_roots
        .iter()
        .any(|excluded| path == excluded || path.starts_with(excluded))
}

fn should_block(name: &str) -> bool {
    matches!(
        name,
        "winsxs"
            | "system32"
            | "syswow64"
            | "installer"
            | "servicing"
            | "assembly"
            | "fonts"
            | "systemapps"
            | "program files"
            | "program files (x86)"
            | "users"
            | "$recycle.bin"
            | "winreclaim"
    )
}

fn is_exact_cache_name(name: &str) -> bool {
    matches!(
        name,
        "cache"
            | "caches"
            | "temp"
            | "tmp"
            | "download"
            | "downloads"
            | "package cache"
            | "shadercache"
            | "dxcache"
            | "glcache"
            | "code cache"
            | "crashdumps"
    )
}

fn looks_like_cache(name: &str) -> bool {
    is_exact_cache_name(name)
        || name.ends_with("-cache")
        || name.ends_with("_cache")
        || name.contains("shader cache")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_cache_names_without_treating_installer_as_cache() {
        assert!(looks_like_cache("dxcache"));
        assert!(looks_like_cache("vendor-cache"));
        assert!(!looks_like_cache("documents"));
        assert!(should_block("installer"));
    }
}
''',
)

# Scanner integration.
replace_once(
    "src-tauri/src/scanner/profile.rs",
    "use super::discovery::{discover_unknown_directories, DiscoveryOptions};\nuse super::sizing::{directory_size, eligible_temp_size, recognised_dump_size};\n",
    "use super::discovery::{discover_unknown_directories, DiscoveryOptions};\nuse super::sizing::{\n    directory_size, eligible_temp_size, prefetch_size, recognised_dump_size, recycle_bin_size,\n};\nuse super::system_cache::{\n    discover_system_drive_caches, SystemCacheOptions,\n};\n",
)
replace_once(
    "src-tauri/src/scanner/profile.rs",
    "use crate::platform::windows::{command_succeeds, disk_snapshot, user_profile};\n",
    "use crate::platform::windows::{\n    command_succeeds, disk_snapshot, system_cache_roots, user_profile,\n};\n",
)
replace_once(
    "src-tauri/src/scanner/profile.rs",
    "    let started_at = Utc::now();\n    let root = request\n",
    "    let started_at = Utc::now();\n    let default_profile_scan = request.root.is_none();\n    let root = request\n",
)
replace_once(
    "src-tauri/src/scanner/profile.rs",
    '''    let targets = if request.include_known_targets {
        known_targets()?
            .into_iter()
            .filter(|target| target.path.starts_with(&root))
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
''',
    '''    let targets = known_targets()?
        .into_iter()
        .filter(|target| {
            let profile_target = request.include_known_targets && target.path.starts_with(&root);
            let system_target = default_profile_scan
                && request.include_system_drive_caches
                && target.rule_id.starts_with("system_drive.");
            profile_target || system_target
        })
        .collect::<Vec<_>>();
''',
)
replace_once(
    "src-tauri/src/scanner/profile.rs",
    "    let total_targets = targets.len() + project_roots.len() + usize::from(request.discover_unknown);\n",
    "    let total_targets = targets.len()\n        + project_roots.len()\n        + usize::from(request.discover_unknown)\n        + usize::from(default_profile_scan && request.include_system_drive_caches);\n",
)
replace_once(
    "src-tauri/src/scanner/profile.rs",
    '''        let stats = match target.rule_id {
            "windows.user_temp" => {
                eligible_temp_size(&target.path, &state.cancel_scan, TEMP_MINIMUM_AGE)
            }
            "windows.crash_dumps" => recognised_dump_size(&target.path, &state.cancel_scan),
            _ => directory_size(&target.path, &state.cancel_scan),
        };
''',
    '''        let stats = match target.rule_id {
            "windows.user_temp" | "system_drive.windows_temp" => {
                eligible_temp_size(&target.path, &state.cancel_scan, TEMP_MINIMUM_AGE)
            }
            "system_drive.prefetch" => prefetch_size(&target.path, &state.cancel_scan),
            "system_drive.recycle_bin" => recycle_bin_size(),
            "windows.crash_dumps" => recognised_dump_size(&target.path, &state.cancel_scan),
            _ => directory_size(&target.path, &state.cancel_scan),
        };
''',
)
replace_once(
    "src-tauri/src/scanner/profile.rs",
    "        let mut excluded = known_paths;\n",
    "        let mut excluded = known_paths.clone();\n",
)
replace_once(
    "src-tauri/src/scanner/profile.rs",
    '''        findings.extend(dynamic.findings);
    }

    findings.sort_by_key(|finding| std::cmp::Reverse(finding.estimated_bytes));
''',
    '''        findings.extend(dynamic.findings);
    }

    if default_profile_scan && request.include_system_drive_caches {
        emit_progress(
            app,
            "Checking caches on the Windows drive",
            None,
            (
                completed_targets,
                total_targets,
                discovered_bytes,
                scanned_entries,
            ),
        );
        let roots = system_cache_roots()?;
        let system_caches = discover_system_drive_caches(
            &roots,
            &known_paths,
            &state.cancel_scan,
            SystemCacheOptions {
                max_depth: mode.system_cache_depth,
                max_entries: mode.system_cache_entries,
                minimum_bytes: request
                    .minimum_finding_bytes
                    .clamp(32 * 1024 * 1024, 20 * 1024 * 1024 * 1024),
                max_findings: request.max_unknown_findings.clamp(5, 30),
            },
        );
        if system_caches.entry_limit_reached {
            errors.push(format!(
                "System-drive cache discovery reached the {:?} entry limit; use Deep or Ultra mode for broader coverage.",
                request.mode
            ));
        }
        scanned_entries = scanned_entries.saturating_add(system_caches.scanned_entries);
        skipped_entries = skipped_entries.saturating_add(system_caches.skipped_entries);
        discovered_bytes = discovered_bytes.saturating_add(system_caches.discovered_bytes);
        findings.extend(system_caches.findings);
        completed_targets += 1;
    }

    findings.sort_by_key(|finding| std::cmp::Reverse(finding.estimated_bytes));
''',
)
replace_once(
    "src-tauri/src/scanner/profile.rs",
    '''struct ModeLimits {
    project_depth: usize,
    discovery_depth: usize,
    max_entries: u64,
}
''',
    '''struct ModeLimits {
    project_depth: usize,
    discovery_depth: usize,
    max_entries: u64,
    system_cache_depth: usize,
    system_cache_entries: u64,
}
''',
)
replace_once(
    "src-tauri/src/scanner/profile.rs",
    '''        ScanMode::Quick => ModeLimits {
            project_depth: 4,
            discovery_depth: 5,
            max_entries: 150_000,
        },
        ScanMode::Balanced => ModeLimits {
            project_depth: 7,
            discovery_depth: 8,
            max_entries: 600_000,
        },
        ScanMode::Deep => ModeLimits {
            project_depth: 10,
            discovery_depth: 14,
            max_entries: 2_000_000,
        },
''',
    '''        ScanMode::Quick => ModeLimits {
            project_depth: 4,
            discovery_depth: 5,
            max_entries: 150_000,
            system_cache_depth: 3,
            system_cache_entries: 100_000,
        },
        ScanMode::Balanced => ModeLimits {
            project_depth: 7,
            discovery_depth: 8,
            max_entries: 600_000,
            system_cache_depth: 5,
            system_cache_entries: 250_000,
        },
        ScanMode::Deep => ModeLimits {
            project_depth: 10,
            discovery_depth: 14,
            max_entries: 2_000_000,
            system_cache_depth: 7,
            system_cache_entries: 750_000,
        },
''',
)
replace_once(
    "src-tauri/src/scanner/profile.rs",
    '''    match action {
        ActionKind::UserTemp | ActionKind::CrashDumps => true,
        ActionKind::HuggingfacePrune => command_succeeds("hf", &["--version"]),
''',
    '''    match action {
        ActionKind::UserTemp
        | ActionKind::SystemTemp
        | ActionKind::Prefetch
        | ActionKind::RecycleBin
        | ActionKind::CrashDumps => true,
        ActionKind::HuggingfacePrune => command_succeeds("hf", &["--version"]),
''',
)

# Exact-root filesystem adapters for Windows Temp and Prefetch.
replace_once(
    "src-tauri/src/actions/filesystem.rs",
    "use crate::platform::windows::{canonical_is_within, is_reparse_point, local_app_data};\n",
    "use crate::platform::windows::{\n    canonical_is_within, is_reparse_point, local_app_data, windows_directory,\n};\n",
)
replace_once(
    "src-tauri/src/actions/filesystem.rs",
    '''pub fn quarantine_crash_dumps(
    target: &Path,
    receipt_id: Uuid,
    finding_id: Uuid,
    display_name: &str,
) -> Result<FilesystemOutcome> {
''',
    '''pub fn clean_system_temp(target: &Path) -> Result<FilesystemOutcome> {
    let allowed = windows_directory()?.join("Temp");
    validate_exact_target(target, &allowed)?;
    delete_tree(
        target,
        Some(TEMP_MINIMUM_AGE),
        None,
        true,
        "stale Windows Temp",
    )
}

pub fn clean_prefetch(target: &Path) -> Result<FilesystemOutcome> {
    let allowed = windows_directory()?.join("Prefetch");
    validate_exact_target(target, &allowed)?;
    delete_tree(target, None, Some(&["pf"]), false, "Windows Prefetch .pf")
}

pub fn quarantine_crash_dumps(
    target: &Path,
    receipt_id: Uuid,
    finding_id: Uuid,
    display_name: &str,
) -> Result<FilesystemOutcome> {
''',
)
replace_once(
    "src-tauri/src/actions/filesystem.rs",
    '''fn collect_eligible_files(
    root: &Path,
''',
    '''fn delete_tree(
    root: &Path,
    minimum_age: Option<Duration>,
    allowed_extensions: Option<&[&str]>,
    remove_empty_directories: bool,
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
            if directory != root && fs::remove_dir(&directory).is_ok() {
                removed_directories = removed_directories.saturating_add(1);
            }
        }
    }
    let affected_entries = removed_files.saturating_add(removed_directories);
    let message = if removed_files > 0 {
        format!(
            "Removed {removed_files} {noun} files; skipped {skipped_entries}. This exact-root action is not stored in the Undo Vault."
        )
    } else if skipped_entries > 0 {
        format!(
            "No eligible {noun} files were removed; skipped {skipped_entries}. Administrator rights or unlocked files may be required."
        )
    } else {
        format!("No eligible {noun} files were available")
    };

    Ok(FilesystemOutcome {
        affected_entries,
        skipped_entries,
        message,
        vault_entry_ids: Vec::new(),
    })
}

fn collect_eligible_files(
    root: &Path,
''',
)

# Action execution and recovery classification.
replace_once(
    "src-tauri/src/actions/mod.rs",
    "use crate::scanner::directory_size;\n",
    "use crate::platform::windows::empty_recycle_bin;\nuse crate::scanner::{directory_size, recycle_bin_size};\n",
)
replace_once(
    "src-tauri/src/actions/mod.rs",
    "    let measured_before = directory_size(target, &cancel).bytes;\n",
    "    let measured_before = measure_target(item.action_kind, target, &cancel);\n",
)
replace_once(
    "src-tauri/src/actions/mod.rs",
    '''        ActionKind::CrashDumps => filesystem::quarantine_crash_dumps(
''',
    '''        ActionKind::SystemTemp => filesystem::clean_system_temp(target).map(|outcome| {
            (
                outcome.affected_entries,
                outcome.skipped_entries,
                outcome.message,
                outcome.vault_entry_ids,
            )
        }),
        ActionKind::Prefetch => filesystem::clean_prefetch(target).map(|outcome| {
            (
                outcome.affected_entries,
                outcome.skipped_entries,
                outcome.message,
                outcome.vault_entry_ids,
            )
        }),
        ActionKind::RecycleBin => empty_recycle_bin().map(|()| {
            (
                0,
                0,
                "Emptied the Recycle Bin on the Windows installation drive through the native Windows Shell API".to_string(),
                Vec::new(),
            )
        }),
        ActionKind::CrashDumps => filesystem::quarantine_crash_dumps(
''',
)
replace_once(
    "src-tauri/src/actions/mod.rs",
    "    let measured_after = directory_size(target, &cancel).bytes;\n",
    "    let measured_after = measure_target(item.action_kind, target, &cancel);\n",
)
replace_once(
    "src-tauri/src/actions/mod.rs",
    '''fn recovery_class(action: ActionKind) -> RecoveryClass {
    match action {
        ActionKind::UserTemp | ActionKind::CrashDumps => RecoveryClass::Reversible,
        ActionKind::HuggingfacePrune | ActionKind::NpmCache => RecoveryClass::Redownloadable,
        ActionKind::DockerPrune => RecoveryClass::Irreversible,
    }
}
''',
    '''fn measure_target(
    action: ActionKind,
    target: &Path,
    cancel: &AtomicBool,
) -> u64 {
    match action {
        ActionKind::RecycleBin => recycle_bin_size().bytes,
        _ => directory_size(target, cancel).bytes,
    }
}

fn recovery_class(action: ActionKind) -> RecoveryClass {
    match action {
        ActionKind::UserTemp | ActionKind::CrashDumps => RecoveryClass::Reversible,
        ActionKind::Prefetch => RecoveryClass::Rebuildable,
        ActionKind::HuggingfacePrune | ActionKind::NpmCache => RecoveryClass::Redownloadable,
        ActionKind::SystemTemp | ActionKind::RecycleBin | ActionKind::DockerPrune => {
            RecoveryClass::Irreversible
        }
    }
}
''',
)

# Passports and simulation recovery classes.
replace_once(
    "src-tauri/src/insights/mod.rs",
    '''    match finding.action_kind {
        Some(ActionKind::UserTemp | ActionKind::CrashDumps) => RecoveryClass::Reversible,
        Some(ActionKind::HuggingfacePrune | ActionKind::NpmCache) => RecoveryClass::Redownloadable,
        Some(ActionKind::DockerPrune) => RecoveryClass::Irreversible,
''',
    '''    match finding.action_kind {
        Some(ActionKind::UserTemp | ActionKind::CrashDumps) => RecoveryClass::Reversible,
        Some(ActionKind::Prefetch) => RecoveryClass::Rebuildable,
        Some(ActionKind::HuggingfacePrune | ActionKind::NpmCache) => RecoveryClass::Redownloadable,
        Some(ActionKind::SystemTemp | ActionKind::RecycleBin | ActionKind::DockerPrune) => {
            RecoveryClass::Irreversible
        }
''',
)
replace_once(
    "src-tauri/src/insights/mod.rs",
    '''    } else if rule.starts_with("windows.") {
        "Windows / installed applications".to_string()
    } else if rule.starts_with("dynamic.") {
''',
    '''    } else if rule.starts_with("system_drive.") {
        "Windows system drive".to_string()
    } else if rule.starts_with("windows.") {
        "Windows / installed applications".to_string()
    } else if rule.starts_with("dynamic.") {
''',
)
replace_once(
    "src-tauri/src/insights/mod.rs",
    '''fn recovery_method_for_rebuild(finding: &Finding) -> String {
    if finding.rule_id == "project.node_modules" {
''',
    '''fn recovery_method_for_rebuild(finding: &Finding) -> String {
    if finding.rule_id == "system_drive.prefetch" {
        "Windows rebuilds Prefetch traces as applications and the system launch again".to_string()
    } else if finding.rule_id == "project.node_modules" {
''',
)

# Receipt wording.
replace_once(
    "src-tauri/src/commands/mod.rs",
    '''            protected_summary: vec![
                "Prefetch".into(),
                "Browser profiles".into(),
''',
    '''            protected_summary: vec![
                "Browser profiles".into(),
''',
)
replace_once(
    "src-tauri/src/commands/mod.rs",
    '                "Windows directories".into(),\n',
    '                "Unverified Windows directories".into(),\n',
)

# Frontend types and scan controls.
replace_once(
    "src/types.ts",
    '''export type ActionKind =
  | "user_temp"
  | "crash_dumps"
''',
    '''export type ActionKind =
  | "user_temp"
  | "system_temp"
  | "prefetch"
  | "recycle_bin"
  | "crash_dumps"
''',
)
replace_once(
    "src/types.ts",
    "  includeAppData: boolean;\n  minimumFindingBytes: number;\n",
    "  includeAppData: boolean;\n  includeSystemDriveCaches: boolean;\n  minimumFindingBytes: number;\n",
)
replace_once(
    "src/features/scan/ScanView.tsx",
    '''  | "discoverUnknown"
  | "includeAppData";
''',
    '''  | "discoverUnknown"
  | "includeAppData"
  | "includeSystemDriveCaches";
''',
)
replace_once(
    "src/features/scan/ScanView.tsx",
    '  ultra: "Runs every scan source using Deep traversal, AppData discovery and maximum result coverage."\n',
    '  ultra: "Runs every scan source using Deep traversal, AppData and Windows-drive cache discovery, and maximum result coverage."\n',
)
replace_once(
    "src/features/scan/ScanView.tsx",
    "  includeAppData: true,\n  minimumFindingBytes: 256 * 1024 * 1024,\n",
    "  includeAppData: true,\n  includeSystemDriveCaches: true,\n  minimumFindingBytes: 256 * 1024 * 1024,\n",
)
replace_once(
    "src/features/scan/ScanView.tsx",
    "  includeAppData: true,\n  minimumFindingBytes: 64 * 1024 * 1024,\n",
    "  includeAppData: true,\n  includeSystemDriveCaches: true,\n  minimumFindingBytes: 64 * 1024 * 1024,\n",
)
replace_once(
    "src/features/scan/ScanView.tsx",
    '''              <Toggle
                label="Include AppData discovery"
                checked={options.includeAppData}
                disabled={scanning || !options.discoverUnknown || ultraLocked}
                onChange={(value) => setFlag("includeAppData", value)}
              />
''',
    '''              <Toggle
                label="Include AppData discovery"
                checked={options.includeAppData}
                disabled={scanning || !options.discoverUnknown || ultraLocked}
                onChange={(value) => setFlag("includeAppData", value)}
              />
              <Toggle
                label="Check Windows-drive caches"
                checked={options.includeSystemDriveCaches}
                disabled={scanning || ultraLocked}
                onChange={(value) => setFlag("includeSystemDriveCaches", value)}
              />
''',
)
replace_once(
    "src/features/scan/ScanView.tsx",
    "                <span>Dynamic findings never receive cleanup actions automatically.</span>\n",
    "                <span>Unknown system-drive caches stay inspection-only; only exact verified roots receive actions.</span>\n",
)

# Documentation.
replace_once(
    "README.md",
    "- Dynamically discovered large directories that do not match the built-in catalogue\n",
    "- Dynamically discovered large directories that do not match the built-in catalogue\n- Classic Windows cleanup targets: `%TEMP%`, Windows Temp, Prefetch `.pf` files and the Recycle Bin\n- Verified and dynamically discovered cache candidates on the drive containing Windows\n",
)
replace_once(
    "README.md",
    "Scan profiles include Quick, Balanced, Deep and Ultra. Ultra enables every source, AppData discovery, the 64 MB minimum threshold and up to 100 dynamic findings while retaining protected-root and reparse-point exclusions.\n",
    "Scan profiles include Quick, Balanced, Deep and Ultra. A separate Windows-drive cache toggle inspects verified servicing, delivery, diagnostics and installer-cache locations plus bounded unknown cache-named directories. Ultra enables every source, AppData and Windows-drive cache discovery, the 64 MB minimum threshold and maximum result coverage while retaining protected-root and reparse-point exclusions.\n",
)
replace_once(
    "README.md",
    "- Recognised user-level crash dumps → compressed Undo Vault\n",
    "- Recognised user-level crash dumps → compressed Undo Vault\n- Windows Temp files older than seven days → exact-root irreversible cleanup\n- Windows Prefetch `.pf` files → manual review-first rebuildable cleanup\n- Windows-drive Recycle Bin → native Windows Shell API, irreversible\n",
)
replace_once(
    "README.md",
    "- clean Prefetch\n",
    "",
)
replace_once(
    "docs/safety.md",
    "WinReclaim never automatically removes Prefetch, registry data, browser profiles, Ollama models, Docker volumes, Android virtual devices, Android SDK packages, Windows directories, Program Files or project source.\n",
    "WinReclaim never automatically removes registry data, browser profiles, Ollama models, Docker volumes, Android virtual devices, Android SDK packages, unverified Windows directories, Program Files or project source. Prefetch is exposed only as a manually selected Review first action, restricted to `.pf` files under the exact active Windows Prefetch root.\n",
)
replace_once(
    "docs/safety.md",
    "Filesystem cleanup validates canonical target paths against compiled allowed roots. Reparse points and links are rejected. Locked files are skipped instead of forcefully removed.\n",
    "Filesystem cleanup validates canonical target paths against compiled allowed roots. Reparse points and links are rejected. Locked files are skipped instead of forcefully removed. `%TEMP%` remains reversible through the compressed vault; Windows Temp and Prefetch are explicit exact-root actions and are labelled with their recovery consequences.\n",
)
replace_once(
    "docs/safety.md",
    "External adapters use `std::process::Command` with explicit argument arrays. No command is passed through a shell. Windows command shims such as `npm.cmd` are resolved by compiled platform code, not by invoking a shell.\n",
    "External adapters use `std::process::Command` with explicit argument arrays. No user-controlled command is passed through a shell. Windows command shims such as `npm.cmd` are resolved by compiled platform code. Recycle Bin inspection and emptying use the native Windows Shell API rather than a script.\n",
)

print("Classic cleanup and system-drive cache migration applied")
