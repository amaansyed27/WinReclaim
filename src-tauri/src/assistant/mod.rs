use crate::domain::{RiskClass, ScanReport, StorageAssistantReport, StorageAssistantStatus};
use anyhow::Result;
use chrono::Utc;
use std::collections::BTreeMap;

const ENGINE_NAME: &str = "WinReclaim deterministic analysis";
const ENGINE_VERSION: &str = "storage-rules-v1";
const MAX_OBSERVATIONS: usize = 6;

#[derive(Debug, Default)]
struct CategoryAccumulator {
    bytes: u64,
    locations: u64,
    actionable_locations: u64,
    safe_now: u64,
    rebuild_or_redownload: u64,
    review_first: u64,
    protected: u64,
}

pub fn status(busy: bool) -> StorageAssistantStatus {
    StorageAssistantStatus {
        available: true,
        busy,
        provider: ENGINE_NAME.to_string(),
        model: ENGINE_VERSION.to_string(),
        privacy_note: "The summary is generated locally from the completed scan. No network request, API key, model download, path upload, or file-content upload is used."
            .to_string(),
    }
}

pub fn analyze(report: &ScanReport) -> Result<StorageAssistantReport> {
    let categories = aggregate_categories(report);
    let actionable_bytes = report
        .findings
        .iter()
        .filter(|finding| finding.action_available)
        .fold(0_u64, |total, finding| {
            total.saturating_add(finding.estimated_bytes)
        });
    let actionable_locations = report
        .findings
        .iter()
        .filter(|finding| finding.action_available)
        .count();
    let drive_count = report.drives.len().max(1);

    let summary = format!(
        "WinReclaim measured {} used and {} free across {} drive{}. The scan reported {} storage location{}, including {} location{} with verified cleanup actions representing up to {} in reported size. Category rows can overlap, so the drive totals remain authoritative.",
        format_bytes(report.disk.used_bytes),
        format_bytes(report.disk.free_bytes),
        drive_count,
        plural(drive_count),
        report.findings.len(),
        plural(report.findings.len()),
        actionable_locations,
        plural(actionable_locations),
        format_bytes(actionable_bytes),
    );

    let mut observations = Vec::new();
    for (category, values) in categories.iter().take(3) {
        observations.push(format!(
            "{} is one of the largest reported categories at {} across {} location{}; {} of those location{} have verified actions.",
            category,
            format_bytes(values.bytes),
            values.locations,
            plural(values.locations as usize),
            values.actionable_locations,
            plural(values.actionable_locations as usize),
        ));
    }

    let risk_counts = report.findings.iter().fold([0_u64; 4], |mut counts, finding| {
        match finding.risk_class {
            RiskClass::SafeNow => counts[0] += 1,
            RiskClass::RebuildOrRedownload => counts[1] += 1,
            RiskClass::ReviewFirst => counts[2] += 1,
            RiskClass::Protected => counts[3] += 1,
        }
        counts
    });
    observations.push(format!(
        "Safety classification: {} low-impact, {} rebuild/redownload, {} review-first, and {} protected location{}.",
        risk_counts[0],
        risk_counts[1],
        risk_counts[2],
        risk_counts[3],
        plural(report.findings.len()),
    ));

    if report.skipped_entries > 0 {
        observations.push(format!(
            "The scanner skipped {} inaccessible, active, protected, or unsupported entr{}; no cleanup authority was inferred from those entries.",
            report.skipped_entries,
            if report.skipped_entries == 1 { "y" } else { "ies" },
        ));
    }

    if !report.errors.is_empty() {
        observations.push(format!(
            "The scan completed with {} reported warning{}; review the scan diagnostics before acting on incomplete areas.",
            report.errors.len(),
            plural(report.errors.len()),
        ));
    }

    observations.truncate(MAX_OBSERVATIONS);

    Ok(StorageAssistantReport {
        scan_id: report.scan_id,
        generated_at: Utc::now(),
        model: ENGINE_VERSION.to_string(),
        summary,
        observations,
        advisory_only: true,
    })
}

fn aggregate_categories(report: &ScanReport) -> Vec<(String, CategoryAccumulator)> {
    let mut categories = BTreeMap::<String, CategoryAccumulator>::new();

    for finding in &report.findings {
        let category = clean_line(&finding.category, 80);
        let entry = categories.entry(category).or_default();
        entry.bytes = entry.bytes.saturating_add(finding.estimated_bytes);
        entry.locations = entry.locations.saturating_add(1);
        if finding.action_available {
            entry.actionable_locations = entry.actionable_locations.saturating_add(1);
        }
        match finding.risk_class {
            RiskClass::SafeNow => entry.safe_now = entry.safe_now.saturating_add(1),
            RiskClass::RebuildOrRedownload => {
                entry.rebuild_or_redownload = entry.rebuild_or_redownload.saturating_add(1)
            }
            RiskClass::ReviewFirst => entry.review_first = entry.review_first.saturating_add(1),
            RiskClass::Protected => entry.protected = entry.protected.saturating_add(1),
        }
    }

    let mut categories = categories.into_iter().collect::<Vec<_>>();
    categories.sort_by(|left, right| right.1.bytes.cmp(&left.1.bytes));
    categories
}

fn clean_line(value: &str, max_chars: usize) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(max_chars)
        .collect()
}

fn format_bytes(bytes: u64) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = KIB * 1024.0;
    const GIB: f64 = MIB * 1024.0;
    const TIB: f64 = GIB * 1024.0;
    let value = bytes as f64;

    if value >= TIB {
        format!("{:.2} TiB", value / TIB)
    } else if value >= GIB {
        format!("{:.2} GiB", value / GIB)
    } else if value >= MIB {
        format!("{:.1} MiB", value / MIB)
    } else if value >= KIB {
        format!("{:.1} KiB", value / KIB)
    } else {
        format!("{bytes} B")
    }
}

fn plural(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_bytes_for_reports() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1024), "1.0 KiB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GiB");
    }

    #[test]
    fn local_status_is_always_available() {
        let status = status(false);
        assert!(status.available);
        assert!(!status.busy);
        assert!(status.privacy_note.contains("No network request"));
    }
}
