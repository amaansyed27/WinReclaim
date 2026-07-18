use super::sizing::{directory_size, eligible_temp_size, recognised_dump_size};
use crate::domain::{ActionKind, Finding, RiskClass, ScanProgress, ScanReport, ScanRequest};
use crate::platform::windows::{command_succeeds, disk_snapshot, user_profile};
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
const PROJECT_SCAN_DEPTH: usize = 7;

pub fn scan_profile(
    app: &AppHandle,
    state: &AppState,
    request: ScanRequest,
) -> Result<ScanReport> {
    state.cancel_scan.store(false, Ordering::Relaxed);
    let started_at = Utc::now();
    let root = request.root.map(PathBuf::from).unwrap_or(user_profile()?);

    if !root.exists() || !root.is_dir() {
        return Err(anyhow!("Scan root is not an accessible directory"));
    }

    let targets = known_targets()?;
    let project_roots = project_roots(&root);
    let total_targets = targets.len() + project_roots.len();
    let mut findings = Vec::new();
    let mut scanned_entries = 0_u64;
    let mut skipped_entries = 0_u64;
    let mut discovered_bytes = 0_u64;
    let mut completed_targets = 0_usize;
    let mut errors = Vec::new();

    emit_progress(
        app,
        "Inspecting known developer and AI storage",
        Some(&root),
        completed_targets,
        total_targets,
        discovered_bytes,
        scanned_entries,
    );

    for target in targets {
        if state.cancel_scan.load(Ordering::Relaxed) {
            return Err(anyhow!("Scan cancelled"));
        }

        emit_progress(
            app,
            "Inspecting known developer and AI storage",
            Some(&target.path),
            completed_targets,
            total_targets,
            discovered_bytes,
            scanned_entries,
        );

        let stats = match target.rule_id {
            "windows.user_temp" => {
                eligible_temp_size(&target.path, &state.cancel_scan, TEMP_MINIMUM_AGE)
            }
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
            "Looking for rebuildable project output",
            Some(&project_root),
            completed_targets,
            total_targets,
            discovered_bytes,
            scanned_entries,
        );

        match discover_project_outputs(
            &project_root,
            &state.cancel_scan,
            &mut seen_project_outputs,
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

    findings.sort_by(|left, right| right.estimated_bytes.cmp(&left.estimated_bytes));
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
        total_targets,
        total_targets,
        discovered_bytes,
        scanned_entries,
    );

    Ok(report)
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
) -> Result<(Vec<Finding>, u64, u64, u64)> {
    let mut findings = Vec::new();
    let mut entries = 0_u64;
    let mut skipped = 0_u64;
    let mut bytes = 0_u64;
    let mut walker = walkdir::WalkDir::new(root)
        .max_depth(PROJECT_SCAN_DEPTH)
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
            "dist" | "build" => Some((
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
        if stats.bytes < 64 * 1024 * 1024 {
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
            explanation: "Generated or installed data located inside a project tree."
                .to_string(),
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
        ActionKind::UserTemp | ActionKind::CrashDumps => true,
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
    completed_targets: usize,
    total_targets: usize,
    discovered_bytes: u64,
    scanned_entries: u64,
) {
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
