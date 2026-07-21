use crate::cloud;
use crate::domain::{RiskClass, ScanReport, StorageAssistantReport, StorageAssistantStatus};
use anyhow::{anyhow, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const ROUTER_NAME: &str = "OpenRouter Free Models Router";
const ROUTER_MODEL: &str = "openrouter/free";
const MAX_CATEGORIES: usize = 40;
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

#[derive(Debug, Serialize)]
struct StorageSummaryPayload {
    used_bytes: u64,
    free_bytes: u64,
    total_bytes: u64,
    drive_count: usize,
    scanned_entries: u64,
    skipped_entries: u64,
    rows_may_overlap: bool,
    categories: Vec<CategorySummary>,
}

#[derive(Debug, Serialize)]
struct CategorySummary {
    category: String,
    bytes: u64,
    locations: u64,
    actionable_locations: u64,
    risk_counts: RiskCounts,
}

#[derive(Debug, Serialize)]
struct RiskCounts {
    safe_now: u64,
    rebuild_or_redownload: u64,
    review_first: u64,
    protected: u64,
}

#[derive(Debug, Deserialize)]
struct CloudStorageSummary {
    summary: String,
    #[serde(default)]
    observations: Vec<String>,
}

pub fn status(busy: bool) -> StorageAssistantStatus {
    StorageAssistantStatus {
        available: cloud::assistant_endpoint().starts_with("https://"),
        busy,
        provider: ROUTER_NAME.to_string(),
        model: ROUTER_MODEL.to_string(),
        privacy_note: "Only aggregate drive totals and category, size, risk and action-count metadata are sent. Paths, usernames, folder names, project names and file contents stay on this PC."
            .to_string(),
    }
}

pub fn analyze(report: &ScanReport) -> Result<StorageAssistantReport> {
    let payload = build_payload(report);
    let (model, response): (String, CloudStorageSummary) =
        cloud::request("storage_summary", &payload)?;

    let summary = clean_line(&response.summary, 700);
    if summary.len() < 20 || contains_cleanup_claim(&summary) {
        return Err(anyhow!(
            "The cloud assistant returned a summary that did not pass WinReclaim safety validation"
        ));
    }

    let observations = response
        .observations
        .into_iter()
        .map(|value| clean_line(&value, 260))
        .filter(|value| value.len() >= 8)
        .filter(|value| !contains_cleanup_claim(value))
        .take(MAX_OBSERVATIONS)
        .collect::<Vec<_>>();

    Ok(StorageAssistantReport {
        scan_id: report.scan_id,
        generated_at: Utc::now(),
        model,
        summary,
        observations,
        advisory_only: true,
    })
}

fn build_payload(report: &ScanReport) -> StorageSummaryPayload {
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
            RiskClass::ReviewFirst => {
                entry.review_first = entry.review_first.saturating_add(1)
            }
            RiskClass::Protected => entry.protected = entry.protected.saturating_add(1),
        }
    }

    let mut categories = categories
        .into_iter()
        .map(|(category, value)| CategorySummary {
            category,
            bytes: value.bytes,
            locations: value.locations,
            actionable_locations: value.actionable_locations,
            risk_counts: RiskCounts {
                safe_now: value.safe_now,
                rebuild_or_redownload: value.rebuild_or_redownload,
                review_first: value.review_first,
                protected: value.protected,
            },
        })
        .collect::<Vec<_>>();
    categories.sort_by(|left, right| right.bytes.cmp(&left.bytes));
    categories.truncate(MAX_CATEGORIES);

    StorageSummaryPayload {
        used_bytes: report.disk.used_bytes,
        free_bytes: report.disk.free_bytes,
        total_bytes: report.disk.total_bytes,
        drive_count: report.drives.len().max(1),
        scanned_entries: report.scanned_entries,
        skipped_entries: report.skipped_entries,
        rows_may_overlap: true,
        categories,
    }
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

fn contains_cleanup_claim(value: &str) -> bool {
    let value = value.to_ascii_lowercase();
    [
        "safe to delete",
        "safe to remove",
        "delete this",
        "remove this",
        "should delete",
        "should remove",
        "clean this",
        "run the command",
    ]
    .iter()
    .any(|claim| value.contains(claim))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleanup_claims_are_rejected() {
        assert!(contains_cleanup_claim("This is safe to delete."));
        assert!(!contains_cleanup_claim(
            "This category contains mostly rebuildable application data."
        ));
    }
}
