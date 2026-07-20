use crate::domain::{ActionKind, Confidence, Finding, RiskClass};
use crate::platform::windows::is_reparse_point;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use uuid::Uuid;
use walkdir::WalkDir;

#[derive(Debug, Clone, Copy)]
pub struct DiscoveryOptions {
    pub max_depth: usize,
    pub max_entries: u64,
    pub minimum_bytes: u64,
    pub max_findings: usize,
    pub include_app_data: bool,
}

#[derive(Debug, Default)]
pub struct DiscoveryResult {
    pub findings: Vec<Finding>,
    pub scanned_entries: u64,
    pub skipped_entries: u64,
    pub discovered_bytes: u64,
    pub entry_limit_reached: bool,
}

#[derive(Debug, Default, Clone, Copy)]
struct Aggregate {
    bytes: u64,
    files: u64,
}

#[derive(Debug, Clone, Copy)]
struct UnknownClassification {
    category: &'static str,
    confidence: Confidence,
    explanation: &'static str,
    consequence: &'static str,
    risk_class: RiskClass,
    action_kind: Option<ActionKind>,
}

pub fn discover_unknown_directories(
    root: &Path,
    excluded_roots: &[PathBuf],
    cancel: &AtomicBool,
    options: DiscoveryOptions,
) -> DiscoveryResult {
    let mut result = DiscoveryResult::default();
    let mut aggregates: HashMap<PathBuf, Aggregate> = HashMap::new();
    let mut walker = WalkDir::new(root)
        .max_depth(options.max_depth)
        .follow_links(false)
        .into_iter();

    while let Some(entry) = walker.next() {
        if cancel.load(Ordering::Relaxed) {
            break;
        }
        if result.scanned_entries >= options.max_entries {
            result.entry_limit_reached = true;
            break;
        }

        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => {
                result.skipped_entries = result.skipped_entries.saturating_add(1);
                continue;
            }
        };
        result.scanned_entries = result.scanned_entries.saturating_add(1);

        let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
        if entry.file_type().is_dir() && should_skip_directory(root, entry.path(), &name, options) {
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
            if metadata.is_dir() {
                walker.skip_current_dir();
            }
            continue;
        }
        if !metadata.is_file() {
            continue;
        }

        let file_bytes = metadata.len();
        let mut ancestor = entry.path().parent();
        while let Some(directory) = ancestor {
            if directory == root || !directory.starts_with(root) {
                break;
            }
            let aggregate = aggregates.entry(directory.to_path_buf()).or_default();
            aggregate.bytes = aggregate.bytes.saturating_add(file_bytes);
            aggregate.files = aggregate.files.saturating_add(1);
            ancestor = directory.parent();
        }
    }

    let mut candidates = aggregates
        .into_iter()
        .filter(|(path, aggregate)| {
            aggregate.bytes >= options.minimum_bytes
                && aggregate.files > 0
                && candidate_depth(root, path) >= minimum_candidate_depth(root, path)
                && !overlaps_excluded_root(path, excluded_roots)
        })
        .collect::<Vec<_>>();

    candidates.sort_by(|(left_path, left), (right_path, right)| {
        right
            .bytes
            .cmp(&left.bytes)
            .then_with(|| candidate_depth(root, right_path).cmp(&candidate_depth(root, left_path)))
    });

    let mut selected_paths: Vec<PathBuf> = Vec::new();
    for (path, aggregate) in candidates {
        if result.findings.len() >= options.max_findings {
            break;
        }
        if selected_paths
            .iter()
            .any(|selected| path.starts_with(selected) || selected.starts_with(&path))
        {
            continue;
        }

        let display_name = path
            .file_name()
            .and_then(|value| value.to_str())
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("Unclassified directory");
        let classification = classify_unknown_name(display_name);
        let action_available = classification.action_kind.is_some();

        result.discovered_bytes = result.discovered_bytes.saturating_add(aggregate.bytes);
        selected_paths.push(path.clone());
        result.findings.push(Finding {
            id: Uuid::new_v4(),
            rule_id: if action_available {
                "dynamic.portable_cache".to_string()
            } else {
                "dynamic.large_directory".to_string()
            },
            display_name: if action_available {
                format!("{display_name} (detected cache)")
            } else {
                format!("{display_name} (unclassified)")
            },
            category: classification.category.to_string(),
            path: path.to_string_lossy().to_string(),
            estimated_bytes: aggregate.bytes,
            risk_class: classification.risk_class,
            explanation: classification.explanation.to_string(),
            consequence: classification.consequence.to_string(),
            confidence: classification.confidence,
            action_kind: classification.action_kind,
            action_available,
            selected_by_default: false,
        });
    }

    result
}

fn should_skip_directory(root: &Path, path: &Path, name: &str, options: DiscoveryOptions) -> bool {
    if matches!(
        name,
        ".git"
            | ".hg"
            | ".svn"
            | "$recycle.bin"
            | "system volume information"
            | "windows"
            | "program files"
            | "program files (x86)"
    ) {
        return true;
    }

    let winreclaim_data = root.join("AppData").join("Local").join("WinReclaim");
    if path == winreclaim_data {
        return true;
    }

    !options.include_app_data && path == root.join("AppData")
}

fn overlaps_excluded_root(candidate: &Path, excluded_roots: &[PathBuf]) -> bool {
    excluded_roots
        .iter()
        .any(|excluded| candidate.starts_with(excluded) || excluded.starts_with(candidate))
}

fn candidate_depth(root: &Path, path: &Path) -> usize {
    path.strip_prefix(root)
        .map(|relative| relative.components().count())
        .unwrap_or_default()
}

fn minimum_candidate_depth(root: &Path, path: &Path) -> usize {
    let relative = path.strip_prefix(root).ok();
    if relative
        .and_then(|value| value.components().next())
        .map(|component| {
            component
                .as_os_str()
                .to_string_lossy()
                .eq_ignore_ascii_case("AppData")
        })
        .unwrap_or(false)
    {
        3
    } else {
        2
    }
}

fn classify_unknown_name(name: &str) -> UnknownClassification {
    let lower = name.trim().to_ascii_lowercase();
    if is_portable_cache_name(&lower) {
        UnknownClassification {
            category: "Detected cache",
            confidence: Confidence::Medium,
            explanation: "Detected from a portable cache-directory fingerprint rather than a machine-specific application rule.",
            consequence: "The owning application may run slower once while it rebuilds or downloads this cache again. Locked or active entries are skipped.",
            risk_class: RiskClass::RebuildOrRedownload,
            action_kind: Some(ActionKind::GenericDirectory),
        }
    } else if lower.contains("model") || lower.contains("checkpoint") || lower.contains("weights") {
        UnknownClassification {
            category: "Potential model data",
            confidence: Confidence::Medium,
            explanation: "Discovered dynamically because this large directory appears related to local model data but has no verified cleanup adapter.",
            consequence: "Inspection only. Confirm the owning application and contents before removing anything.",
            risk_class: RiskClass::ReviewFirst,
            action_kind: None,
        }
    } else {
        UnknownClassification {
            category: "Unclassified storage",
            confidence: Confidence::Low,
            explanation: "Discovered by bounded size analysis because this directory is large and does not match a known WinReclaim rule.",
            consequence: "Inspection only. Confirm the owning application and contents before removing anything.",
            risk_class: RiskClass::ReviewFirst,
            action_kind: None,
        }
    }
}

fn is_portable_cache_name(name: &str) -> bool {
    matches!(
        name,
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
    ) || name.ends_with("-cache")
        || name.ends_with("_cache")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    fn discovers_portable_cache_with_optional_action() {
        let root = std::env::temp_dir().join(format!("winreclaim-discovery-{}", Uuid::new_v4()));
        let candidate = root.join("Vendor").join("shader-cache");
        fs::create_dir_all(&candidate).unwrap();
        let mut file = fs::File::create(candidate.join("blob.bin")).unwrap();
        file.write_all(&[1_u8; 128]).unwrap();

        let cancel = AtomicBool::new(false);
        let result = discover_unknown_directories(
            &root,
            &[],
            &cancel,
            DiscoveryOptions {
                max_depth: 6,
                max_entries: 1_000,
                minimum_bytes: 64,
                max_findings: 10,
                include_app_data: true,
            },
        );

        assert_eq!(result.findings.len(), 1);
        assert_eq!(
            result.findings[0].risk_class,
            RiskClass::RebuildOrRedownload
        );
        assert_eq!(
            result.findings[0].action_kind,
            Some(ActionKind::GenericDirectory)
        );
        assert!(result.findings[0].action_available);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn keeps_unclassified_directories_inspection_only() {
        let classification = classify_unknown_name("plugins");
        assert_eq!(classification.risk_class, RiskClass::ReviewFirst);
        assert!(classification.action_kind.is_none());
    }

    #[test]
    fn skips_winreclaim_internal_app_data() {
        let root = PathBuf::from("C:\\Users\\Example");
        let internal = root.join("AppData").join("Local").join("WinReclaim");
        let options = DiscoveryOptions {
            max_depth: 6,
            max_entries: 1_000,
            minimum_bytes: 64,
            max_findings: 10,
            include_app_data: true,
        };
        assert!(should_skip_directory(
            &root,
            &internal,
            "winreclaim",
            options
        ));
    }
}
