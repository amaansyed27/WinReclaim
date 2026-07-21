use super::discovery::{discover_unknown_directories, DiscoveryOptions};
use super::sizing::{directory_size, recognised_dump_size, recycle_bin_size};
use super::system_cache::{discover_system_drive_caches, SystemCacheOptions};
use crate::domain::{
    ActionKind, DiskSnapshot, DriveInfo, DriveKind, Finding, RiskClass, ScanMode, ScanProgress,
    ScanReport, ScanRequest,
};
use crate::platform::windows::{
    command_succeeds, disk_snapshot, drive_root, list_drives, same_drive, system_cache_roots,
    system_drive_root, user_profile,
};
use crate::policy::scan_scope_fingerprint;
use crate::rules::known_targets;
use crate::storage::AppState;
use anyhow::{anyhow, Result};
use chrono::Utc;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

type ProgressCounts = (usize, usize, u64, u64);

pub fn scan_profile(app: &AppHandle, state: &AppState, request: ScanRequest) -> Result<ScanReport> {
    state.cancel_scan.store(false, Ordering::Relaxed);
    let started_at = Utc::now();
    let available_drives = list_drives()?;
    let roots = resolve_roots(&request, &available_drives)?;
    if roots.is_empty() {
        return Err(anyhow!("Select at least one accessible drive"));
    }

    let selected_drives = selected_drive_info(&roots, &available_drives)?;
    let mut findings_by_key = HashMap::<String, Finding>::new();
    let mut scanned_entries = 0_u64;
    let mut skipped_entries = 0_u64;
    let mut errors = Vec::new();

    for (index, root) in roots.iter().enumerate() {
        if state.cancel_scan.load(Ordering::Relaxed) {
            return Err(anyhow!("Scan cancelled"));
        }
        let drive = selected_drives
            .iter()
            .find(|drive| same_drive(Path::new(&drive.root), root));
        let mut partial = scan_single_root(
            app,
            state,
            &request,
            root,
            index + 1,
            roots.len(),
            drive.map(|value| value.kind),
        )?;
        scanned_entries = scanned_entries.saturating_add(partial.scanned_entries);
        skipped_entries = skipped_entries.saturating_add(partial.skipped_entries);
        errors.append(&mut partial.errors);
        for finding in partial.findings {
            let key = format!("{}|{}", finding.rule_id, finding.path.to_ascii_lowercase());
            findings_by_key
                .entry(key)
                .and_modify(|existing| {
                    if finding.estimated_bytes > existing.estimated_bytes {
                        *existing = finding.clone();
                    }
                })
                .or_insert(finding);
        }
    }

    let mut findings = findings_by_key.into_values().collect::<Vec<_>>();
    findings.sort_by_key(|finding| std::cmp::Reverse(finding.estimated_bytes));
    let disk = aggregate_disk(&selected_drives);
    let scope_fingerprint = scan_scope_fingerprint(&roots, &selected_drives, &request);
    let root_label = roots
        .iter()
        .map(|root| root.to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join(";");

    let report = ScanReport {
        scan_id: Uuid::new_v4(),
        started_at,
        completed_at: Utc::now(),
        root: root_label,
        drives: selected_drives,
        scope_fingerprint,
        disk,
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
            roots.len(),
            roots.len(),
            report.disk.used_bytes,
            scanned_entries,
        ),
    );
    Ok(report)
}

fn scan_single_root(
    app: &AppHandle,
    state: &AppState,
    request: &ScanRequest,
    root: &Path,
    drive_index: usize,
    drive_count: usize,
    drive_kind: Option<DriveKind>,
) -> Result<PartialScan> {
    if !root.exists() || !root.is_dir() {
        return Err(anyhow!(
            "Scan root {} is not an accessible directory",
            root.display()
        ));
    }

    let system_selected = same_drive(root, &system_drive_root()?);
    let profile_scan = request.roots.is_empty() && request.root.is_none();
    let mode = mode_limits(request.mode);
    let targets = known_targets()?
        .into_iter()
        .filter(|target| {
            let selected_target = request.include_known_targets && target.path.starts_with(root);
            let system_target = system_selected
                && request.include_system_drive_caches
                && target.rule_id.starts_with("system_drive.");
            let profile_target = profile_scan
                && request.include_known_targets
                && target.path.starts_with(user_profile().unwrap_or_default());
            selected_target || system_target || profile_target
        })
        .collect::<Vec<_>>();
    let known_paths = targets
        .iter()
        .map(|target| target.path.clone())
        .collect::<Vec<_>>();
    let project_roots = if request.include_project_outputs {
        project_roots(root)
    } else {
        Vec::new()
    };
    let total_targets = targets.len()
        + project_roots.len()
        + usize::from(request.discover_unknown)
        + usize::from(system_selected && request.include_system_drive_caches);
    let mut findings = Vec::new();
    let mut scanned_entries = 0_u64;
    let mut skipped_entries = 0_u64;
    let mut discovered_bytes = 0_u64;
    let mut completed_targets = 0_usize;
    let mut errors = Vec::new();
    let prefix = format!("Drive {drive_index} of {drive_count}");

    emit_progress(
        app,
        &format!("{prefix}: inspecting verified storage locations"),
        Some(root),
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
            &format!("{prefix}: inspecting verified storage locations"),
            Some(&target.path),
            (
                completed_targets,
                total_targets,
                discovered_bytes,
                scanned_entries,
            ),
        );
        let stats = match target.rule_id {
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
            apply_drive_safety(&mut finding, drive_kind);
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
            &format!("{prefix}: discovering project output"),
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
                for finding in &mut output_findings {
                    apply_drive_safety(finding, drive_kind);
                }
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
            &format!("{prefix}: discovering large directories"),
            Some(root),
            (
                completed_targets,
                total_targets,
                discovered_bytes,
                scanned_entries,
            ),
        );
        let mut excluded = known_paths.clone();
        excluded.extend(seen_project_outputs.iter().cloned());
        let mut dynamic = discover_unknown_directories(
            root,
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
                "{} discovery reached the {:?} scan entry limit; use Deep mode for broader coverage.",
                root.display(),
                request.mode
            ));
        }
        for finding in &mut dynamic.findings {
            apply_drive_safety(finding, drive_kind);
        }
        scanned_entries = scanned_entries.saturating_add(dynamic.scanned_entries);
        skipped_entries = skipped_entries.saturating_add(dynamic.skipped_entries);
        discovered_bytes = discovered_bytes.saturating_add(dynamic.discovered_bytes);
        findings.extend(dynamic.findings);
    }

    if system_selected && request.include_system_drive_caches {
        emit_progress(
            app,
            &format!("{prefix}: checking caches on the Windows drive"),
            None,
            (
                completed_targets,
                total_targets,
                discovered_bytes,
                scanned_entries,
            ),
        );
        let roots = system_cache_roots()?;
        let mut system_caches = discover_system_drive_caches(
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
        for finding in &mut system_caches.findings {
            apply_drive_safety(finding, drive_kind);
        }
        scanned_entries = scanned_entries.saturating_add(system_caches.scanned_entries);
        skipped_entries = skipped_entries.saturating_add(system_caches.skipped_entries);
        findings.extend(system_caches.findings);
    }

    Ok(PartialScan {
        findings,
        scanned_entries,
        skipped_entries,
        errors,
    })
}

#[derive(Debug)]
struct PartialScan {
    findings: Vec<Finding>,
    scanned_entries: u64,
    skipped_entries: u64,
    errors: Vec<String>,
}

fn resolve_roots(request: &ScanRequest, drives: &[DriveInfo]) -> Result<Vec<PathBuf>> {
    let requested = if !request.roots.is_empty() {
        request.roots.iter().map(PathBuf::from).collect::<Vec<_>>()
    } else if let Some(root) = &request.root {
        vec![PathBuf::from(root)]
    } else {
        vec![user_profile()?]
    };

    let mut roots = Vec::new();
    let mut seen = HashSet::new();
    for requested_root in requested {
        if !requested_root.exists() || !requested_root.is_dir() {
            return Err(anyhow!(
                "{} is not an accessible directory",
                requested_root.display()
            ));
        }
        let key = requested_root.to_string_lossy().to_ascii_lowercase();
        if seen.insert(key) {
            roots.push(requested_root);
        }
    }

    if !request.roots.is_empty() {
        for root in &roots {
            if !drives
                .iter()
                .any(|drive| same_drive(Path::new(&drive.root), root))
            {
                return Err(anyhow!(
                    "{} is not one of the currently mounted drives",
                    root.display()
                ));
            }
        }
    }
    roots.sort_by_key(|root| root.to_string_lossy().to_ascii_lowercase());
    Ok(roots)
}

fn selected_drive_info(roots: &[PathBuf], drives: &[DriveInfo]) -> Result<Vec<DriveInfo>> {
    let mut selected = Vec::new();
    let mut seen = HashSet::new();
    for root in roots {
        let drive = drives
            .iter()
            .find(|drive| same_drive(Path::new(&drive.root), root))
            .cloned()
            .or_else(|| {
                disk_snapshot(root).ok().map(|snapshot| DriveInfo {
                    root: snapshot.root.clone(),
                    label: String::new(),
                    file_system: String::new(),
                    volume_id: snapshot.root.clone(),
                    total_bytes: snapshot.total_bytes,
                    free_bytes: snapshot.free_bytes,
                    used_bytes: snapshot.used_bytes,
                    is_system: same_drive(root, &system_drive_root().unwrap_or_default()),
                    kind: DriveKind::Fixed,
                })
            })
            .ok_or_else(|| anyhow!("Unable to read drive information for {}", root.display()))?;
        if seen.insert(drive.volume_id.clone()) {
            selected.push(drive);
        }
    }
    selected.sort_by_key(|drive| (!drive.is_system, drive.root.clone()));
    Ok(selected)
}

fn aggregate_disk(drives: &[DriveInfo]) -> DiskSnapshot {
    DiskSnapshot {
        root: drives
            .iter()
            .map(|drive| drive.root.clone())
            .collect::<Vec<_>>()
            .join(";"),
        total_bytes: drives.iter().map(|drive| drive.total_bytes).sum(),
        free_bytes: drives.iter().map(|drive| drive.free_bytes).sum(),
        used_bytes: drives.iter().map(|drive| drive.used_bytes).sum(),
    }
}

fn apply_drive_safety(finding: &mut Finding, drive_kind: Option<DriveKind>) {
    if matches!(
        drive_kind,
        Some(DriveKind::Network | DriveKind::Removable | DriveKind::Optical | DriveKind::Other)
    ) {
        finding.action_available = false;
        if finding.risk_class != RiskClass::Protected {
            finding.risk_class = RiskClass::ReviewFirst;
        }
        finding.consequence =
            "This volume is inspection-only. WinReclaim does not clean removable or network storage."
                .to_string();
    }
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

fn project_roots(root: &Path) -> Vec<PathBuf> {
    if root == drive_root(root) {
        return vec![root.to_path_buf()];
    }

    [
        "Desktop",
        "Documents",
        "Downloads",
        "Projects",
        "Source",
        "source",
        "dev",
        "develop",
        "repos",
        "workspace",
    ]
    .into_iter()
    .map(|name| root.join(name))
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

        let Some((rule_id, display_name, category, consequence, risk_class)) =
            project_output_descriptor(entry.path(), &name)
        else {
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
            risk_class,
            explanation: "Rebuildable or reinstallable data verified by project files next to this directory.".to_string(),
            consequence: consequence.to_string(),
            confidence: crate::domain::Confidence::High,
            action_kind: Some(ActionKind::GenericDirectory),
            action_available: true,
            selected_by_default: false,
        });
    }
    Ok((findings, entries, skipped, bytes))
}

fn project_output_descriptor(
    path: &Path,
    name: &str,
) -> Option<(
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    RiskClass,
)> {
    let parent = path.parent().unwrap_or(path);
    match name {
        "node_modules"
            if has_any_marker(
                parent,
                &[
                    "package.json",
                    "package-lock.json",
                    "pnpm-lock.yaml",
                    "yarn.lock",
                    "bun.lock",
                    "bun.lockb",
                ],
            ) =>
        {
            Some((
                "project.node_modules",
                "Project node_modules",
                "JavaScript project dependencies",
                "Dependencies must be installed again from the project manifest or lockfile.",
                RiskClass::RebuildOrRedownload,
            ))
        }
        "target" if parent.join("Cargo.toml").is_file() => Some((
            "project.rust_target",
            "Rust target directory",
            "Rust build output",
            "Cargo will rebuild the removed binaries and intermediate artifacts.",
            RiskClass::RebuildOrRedownload,
        )),
        ".venv" | "venv" if path.join("pyvenv.cfg").is_file() => Some((
            "project.python_venv",
            "Python virtual environment",
            "Python project environment",
            "The environment must be recreated and dependencies reinstalled.",
            RiskClass::RebuildOrRedownload,
        )),
        "dist" | "build" | ".next" | ".nuxt" | "out" | "coverage" if has_project_marker(parent) => {
            Some((
                "project.build_output",
                "Project build output",
                "Generated project output",
                "The next project build should recreate this directory.",
                RiskClass::ReviewFirst,
            ))
        }
        _ => None,
    }
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

fn action_is_available(action: ActionKind) -> bool {
    match action {
        ActionKind::UserTemp
        | ActionKind::SystemTemp
        | ActionKind::Prefetch
        | ActionKind::GenericDirectory
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn project_outputs_require_nearby_project_evidence() {
        let root = std::env::temp_dir().join(format!("winreclaim-project-{}", Uuid::new_v4()));
        let project = root.join("app");
        let modules = project.join("node_modules");
        fs::create_dir_all(&modules).unwrap();
        assert!(project_output_descriptor(&modules, "node_modules").is_none());
        fs::write(project.join("package.json"), "{}").unwrap();
        assert!(project_output_descriptor(&modules, "node_modules").is_some());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn aggregate_disk_sums_selected_volumes() {
        let drives = vec![
            DriveInfo {
                root: "C:\\".into(),
                label: String::new(),
                file_system: "NTFS".into(),
                volume_id: "1".into(),
                total_bytes: 100,
                free_bytes: 40,
                used_bytes: 60,
                is_system: true,
                kind: DriveKind::Fixed,
            },
            DriveInfo {
                root: "D:\\".into(),
                label: String::new(),
                file_system: "NTFS".into(),
                volume_id: "2".into(),
                total_bytes: 200,
                free_bytes: 50,
                used_bytes: 150,
                is_system: false,
                kind: DriveKind::Fixed,
            },
        ];
        let disk = aggregate_disk(&drives);
        assert_eq!(disk.total_bytes, 300);
        assert_eq!(disk.free_bytes, 90);
        assert_eq!(disk.used_bytes, 210);
    }
}
