use super::sizing::directory_size;
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
