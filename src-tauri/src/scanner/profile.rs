use super::discovery::{discover_unknown_directories, DiscoveryOptions};
use super::sizing::{
    directory_size, eligible_temp_size, prefetch_size, recognised_dump_size, recycle_bin_size,
};
use super::system_cache::{discover_system_drive_caches, SystemCacheOptions};
use crate::domain::{
    ActionKind, Finding, RiskClass, ScanMode, ScanProgress, ScanReport, ScanRequest,
};
use crate::platform::windows::{command_succeeds, disk_snapshot, system_cache_roots, user_profile};
use crate::rules::known_targets;
use crate::storage::AppState;
use anyhow::{anyhow, Result};
use chrono::Utc;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

const TEMP_MINIMUM_AGE: Duration = Duration::from_secs(7 * 24 * 60 * 60);
type ProgressCounts = (usize, usize, u64, u64);

pub fn scan_profile(app: &AppHandle, state: &AppState, request: ScanRequest) -> Result<ScanReport> {
    state.cancel_scan.store(false, Ordering::Relaxed);
    let started_at = Utc::now();
    let default_profile_scan = request.root.is_none();
    let root = request
        .root
        .clone()
        .map(PathBuf::from)
        .unwrap_or(user_profile()?);

    if !root.exists() || !root.is_dir() {
        return Err(anyhow!("Scan root is not an accessible directory"));
    }

    let mode = mode_limits(request.mode);
    let targets = known_targets()?
        .into_iter()
        .filter(|target| {
            let profile_target = request.include_known_targets && target.path.starts_with(&root);
            let system_target = default_profile_scan
                && request.include_system_drive_caches
                && target.rule_id.starts_with("system_drive.");
            profile_target || system_target
        })
        .collect::<Vec<_>>();
    let known_paths = targets
        .iter()
        .map(|target| target.path.clone())
        .collect::<Vec<_>>();
    let project_roots = if request.include_project_outputs {
        project_roots(&root)
    } else {
        Vec::new()
    };
    let total_targets = targets.len()
        + project_roots.len()
        + usize::from(request.discover_unknown)
        + usize::from(default_profile_scan && request.include_system_drive_caches);
    let mut findings = Vec::new();
    let mut scanned_entries = 0_u64;
    let mut skipped_entries = 0_u64;
    let mut discovered_bytes = 0_u64;
    let mut completed_targets = 0_usize;
    let mut errors = Vec::new();

    emit_progress(
        app,
        "Inspecting verified storage locations",
        Some(&root),
        (
            completed_targets,
            total_targets,
            discovered_bytes,
            scanned_entries,
        ),
    );

    for target in targets {
        if state.cancel_scan.load(Ordering::Relaxed) {
            return Err(anyhow!("Scan cancelled"));
        }
        emit_progress(
            app,
            "Inspecting verified storage locations",
            Some(&target.path),
            (
                completed_targets,
                total_targets,
                discovered_bytes,
                scanned_entries,
            ),
        );
        let stats = match target.rule_id {
            "windows.user_temp" | "system_drive.windows_temp" => {
                eligible_temp_size(&target.path, &state.cancel_scan, TEMP_MINIMUM_AGE)
            }
            "system_drive.prefetch" => prefetch_size(&target.path, &state.cancel_scan),
            "system_drive.recycle_bin" => recycle_bin_size(),
            "windows.crash_dumps" => recognised_dump_size(&target.path, &state.cancel_scan),
            _ => directory_size(&target.path, &state.cancel_scan),
        };
        scanned_entries = scanned_entries.saturating_add(stats.entries);
        skipped_entries = skipped_entries.saturating_add(stats.skipped);
        discovered_bytes = discovered_bytes.saturating_add(stats.bytes);
        completed_targets += 1;
        if stats.bytes > 0 {
            let mut finding = target.into_finding(stats.bytes);
            if let Some(action) = finding.action_kind {
                finding.action_available = action_is_available(action);
            }
            findings.push(finding);
        }
    }

    let mut seen_project_outputs = HashSet::new();
    for project_root in project_roots {
        if state.cancel_scan.load(Ordering::Relaxed) {
            return Err(anyhow!("Scan cancelled"));
        }
        emit_progress(
            app,
            "Discovering project output",
            Some(&project_root),
            (
                completed_targets,
                total_targets,
                discovered_bytes,
                scanned_entries,
            ),
        );
        match discover_project_outputs(
            &project_root,
            &state.cancel_scan,
            &mut seen_project_outputs,
            mode.project_depth,
            request.minimum_finding_bytes.max(64 * 1024 * 1024),
        ) {
            Ok((mut output_findings, entries, skipped, bytes)) => {
                findings.append(&mut output_findings);
                scanned_entries = scanned_entries.saturating_add(entries);
                skipped_entries = skipped_entries.saturating_add(skipped);
                discovered_bytes = discovered_bytes.saturating_add(bytes);
            }
            Err(error) => errors.push(format!("{}: {error}", project_root.display())),
        }
        completed_targets += 1;
    }

    if request.discover_unknown {
        emit_progress(
            app,
            "Discovering unclassified large directories",
            Some(&root),
            (
                completed_targets,
                total_targets,
                discovered_bytes,
                scanned_entries,
            ),
        );
        let mut excluded = known_paths.clone();
        excluded.extend(seen_project_outputs.iter().cloned());
        let dynamic = discover_unknown_directories(
            &root,
            &excluded,
            &state.cancel_scan,
            DiscoveryOptions {
                max_depth: mode.discovery_depth,
                max_entries: mode.max_entries,
                minimum_bytes: request
                    .minimum_finding_bytes
                    .clamp(32 * 1024 * 1024, 20 * 1024 * 1024 * 1024),
                max_findings: request.max_unknown_findings.clamp(5, 100),
                include_app_data: request.include_app_data,
            },
        );
        if dynamic.entry_limit_reached {
            errors.push(format!(
                "Dynamic discovery reached the {:?} scan entry limit; use Deep mode for broader coverage.",
                request.mode
            ));
        }
        scanned_entries = scanned_entries.saturating_add(dynamic.scanned_entries);
        skipped_entries = skipped_entries.saturating_add(dynamic.skipped_entries);
        discovered_bytes = discovered_bytes.saturating_add(dynamic.discovered_bytes);
        findings.extend(dynamic.findings);
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
    }

    findings.sort_by_key(|finding| std::cmp::Reverse(finding.estimated_bytes));
    let report = ScanReport {
        scan_id: Uuid::new_v4(),
        started_at,
        completed_at: Utc::now(),
        root: root.to_string_lossy().to_string(),
        disk: disk_snapshot(&root)?,
        findings,
        scanned_entries,
        skipped_entries,
        errors,
    };

    emit_progress(
        app,
        "Scan complete",
        None,
        (
            total_targets,
            total_targets,
            discovered_bytes,
            scanned_entries,
        ),
    );
    Ok(report)
}

#[derive(Debug, Clone, Copy)]
struct ModeLimits {
    project_depth: usize,
    discovery_depth: usize,
    max_entries: u64,
    system_cache_depth: usize,
    system_cache_entries: u64,
}

fn mode_limits(mode: ScanMode) -> ModeLimits {
    match mode {
        ScanMode::Quick => ModeLimits {
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
    }
}

fn project_roots(user: &Path) -> Vec<PathBuf> {
    [
        "Desktop",
        "Documents",
        "Downloads",
        "Projects",
        "Source",
        "source",
        "dev",
        "repos",
        "workspace",
    ]
    .into_iter()
    .map(|name| user.join(name))
    .filter(|path| path.exists() && path.is_dir())
    .collect()
}

fn discover_project_outputs(
    root: &Path,
    cancel: &std::sync::atomic::AtomicBool,
    seen: &mut HashSet<PathBuf>,
    max_depth: usize,
    minimum_bytes: u64,
) -> Result<(Vec<Finding>, u64, u64, u64)> {
    let mut findings = Vec::new();
    let mut entries = 0_u64;
    let mut skipped = 0_u64;
    let mut bytes = 0_u64;
    let mut walker = walkdir::WalkDir::new(root)
        .max_depth(max_depth)
        .follow_links(false)
        .into_iter();

    while let Some(entry) = walker.next() {
        if cancel.load(Ordering::Relaxed) {
            break;
        }
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => {
                skipped += 1;
                continue;
            }
        };
        entries += 1;
        let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
        if matches!(name.as_str(), ".git" | ".hg" | ".svn" | "$recycle.bin") {
            if entry.file_type().is_dir() {
                walker.skip_current_dir();
            }
            continue;
        }
        if !entry.file_type().is_dir() {
            continue;
        }

        let descriptor = match name.as_str() {
            "node_modules" => Some((
                "project.node_modules",
                "Project node_modules",
                "JavaScript project dependencies",
                "Dependencies can be restored from the project lockfile, but the project will not run until packages are installed again.",
            )),
            "target" => Some((
                "project.rust_target",
                "Rust target directory",
                "Rust build output",
                "Cargo will rebuild the removed binaries and intermediate artifacts.",
            )),
            ".venv" | "venv" => Some((
                "project.python_venv",
                "Python virtual environment",
                "Python project environment",
                "The environment must be recreated and dependencies reinstalled.",
            )),
            "dist" | "build" | ".next" | ".nuxt" | "out" | "coverage" => Some((
                "project.build_output",
                "Project build output",
                "Generated project output",
                "Confirm the directory is generated; the next project build should recreate it.",
            )),
            _ => None,
        };
        let Some((rule_id, display_name, category, consequence)) = descriptor else {
            continue;
        };
        walker.skip_current_dir();
        let canonical = entry
            .path()
            .canonicalize()
            .unwrap_or_else(|_| entry.path().to_path_buf());
        if !seen.insert(canonical) {
            continue;
        }
        let stats = directory_size(entry.path(), cancel);
        if stats.bytes < minimum_bytes {
            continue;
        }
        bytes = bytes.saturating_add(stats.bytes);
        entries = entries.saturating_add(stats.entries);
        skipped = skipped.saturating_add(stats.skipped);
        findings.push(Finding {
            id: Uuid::new_v4(),
            rule_id: rule_id.to_string(),
            display_name: display_name.to_string(),
            category: category.to_string(),
            path: entry.path().to_string_lossy().to_string(),
            estimated_bytes: stats.bytes,
            risk_class: RiskClass::ReviewFirst,
            explanation: "Generated or installed data located inside a project tree.".to_string(),
            consequence: consequence.to_string(),
            confidence: crate::domain::Confidence::Medium,
            action_kind: None,
            action_available: false,
            selected_by_default: false,
        });
    }
    Ok((findings, entries, skipped, bytes))
}

fn action_is_available(action: ActionKind) -> bool {
    match action {
        ActionKind::UserTemp
        | ActionKind::SystemTemp
        | ActionKind::Prefetch
        | ActionKind::RecycleBin
        | ActionKind::CrashDumps => true,
        ActionKind::HuggingfacePrune => command_succeeds("hf", &["--version"]),
        ActionKind::NpmCache => command_succeeds("npm", &["--version"]),
        ActionKind::DockerPrune => {
            command_succeeds("docker", &["info", "--format", "{{.ServerVersion}}"])
        }
    }
}

fn emit_progress(
    app: &AppHandle,
    phase: &str,
    current_path: Option<&Path>,
    counts: ProgressCounts,
) {
    let (completed_targets, total_targets, discovered_bytes, scanned_entries) = counts;
    let _ = app.emit(
        "scan-progress",
        ScanProgress {
            phase: phase.to_string(),
            current_path: current_path.map(|path| path.to_string_lossy().to_string()),
            completed_targets,
            total_targets,
            discovered_bytes,
            scanned_entries,
        },
    );
}
