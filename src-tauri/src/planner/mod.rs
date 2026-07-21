use crate::domain::{
    CleanupPlan, CleanupPlanItem, CreatePlanRequest, PlanSimulation, RecoveryClass, ScanReport,
};
use crate::policy::recovery_class_for_action;
use crate::rules::RULE_SET_VERSION;
use anyhow::{anyhow, Result};
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use uuid::Uuid;

pub fn build_plan(report: &ScanReport, request: &CreatePlanRequest) -> Result<CleanupPlan> {
    if report.scan_id != request.scan_id {
        return Err(anyhow!("The selected scan is no longer current"));
    }
    let selected: HashSet<Uuid> = request.finding_ids.iter().copied().collect();
    if selected.is_empty() {
        return Err(anyhow!("Select at least one cleanup action"));
    }
    let mut items = Vec::new();
    for finding in &report.findings {
        if !selected.contains(&finding.id) {
            continue;
        }
        let action_kind = finding.action_kind.ok_or_else(|| {
            anyhow!(
                "{} does not have an executable cleanup adapter",
                finding.display_name
            )
        })?;
        if !finding.action_available
            || matches!(finding.risk_class, crate::domain::RiskClass::Protected)
        {
            return Err(anyhow!(
                "{} is protected or review-only",
                finding.display_name
            ));
        }
        items.push(CleanupPlanItem {
            finding_id: finding.id,
            display_name: finding.display_name.clone(),
            category: finding.category.clone(),
            path: finding.path.clone(),
            estimated_bytes: finding.estimated_bytes,
            risk_class: finding.risk_class,
            consequence: finding.consequence.clone(),
            action_kind,
        });
    }
    if items.len() != selected.len() {
        return Err(anyhow!(
            "One or more selected findings were not present in the current scan"
        ));
    }

    let estimated_reclaim_bytes = items.iter().map(|item| item.estimated_bytes).sum();
    let simulation = build_simulation(report, &items, estimated_reclaim_bytes);
    let mut plan = CleanupPlan {
        id: Uuid::new_v4(),
        scan_id: report.scan_id,
        created_at: Utc::now(),
        estimated_reclaim_bytes,
        items,
        simulation,
        rule_set_version: RULE_SET_VERSION.to_string(),
        plan_hash: String::new(),
    };
    plan.plan_hash = hash_plan(&plan)?;
    Ok(plan)
}

pub fn verify_plan_hash(plan: &CleanupPlan) -> Result<bool> {
    Ok(hash_plan(plan)? == plan.plan_hash)
}

fn build_simulation(
    report: &ScanReport,
    items: &[CleanupPlanItem],
    estimated_reclaim_bytes: u64,
) -> PlanSimulation {
    let mut simulation = PlanSimulation {
        current_free_bytes: report.disk.free_bytes,
        projected_free_bytes: report
            .disk
            .free_bytes
            .saturating_add(estimated_reclaim_bytes),
        estimated_reclaim_bytes,
        affected_items: items.len(),
        ..PlanSimulation::default()
    };

    for item in items {
        match recovery_class_for_action(item.action_kind) {
            RecoveryClass::Reversible => {
                simulation.reversible_bytes = simulation
                    .reversible_bytes
                    .saturating_add(item.estimated_bytes);
            }
            RecoveryClass::Redownloadable => {
                simulation.redownloadable_bytes = simulation
                    .redownloadable_bytes
                    .saturating_add(item.estimated_bytes);
            }
            RecoveryClass::Rebuildable => {
                simulation.rebuildable_bytes = simulation
                    .rebuildable_bytes
                    .saturating_add(item.estimated_bytes);
            }
            RecoveryClass::Irreversible | RecoveryClass::Protected => {
                simulation.irreversible_bytes = simulation
                    .irreversible_bytes
                    .saturating_add(item.estimated_bytes);
            }
        }
    }

    simulation
}

fn hash_plan(plan: &CleanupPlan) -> Result<String> {
    let mut hashable = plan.clone();
    hashable.plan_hash.clear();
    Ok(hex::encode(Sha256::digest(serde_json::to_vec(&hashable)?)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{ActionKind, Confidence, DiskSnapshot, Finding, RiskClass};

    #[test]
    fn plan_hash_detects_mutation() {
        let finding = Finding {
            id: Uuid::new_v4(),
            rule_id: "test.temp".into(),
            display_name: "Temp".into(),
            category: "Test".into(),
            path: std::env::temp_dir().to_string_lossy().to_string(),
            estimated_bytes: 100,
            risk_class: RiskClass::SafeNow,
            explanation: "Fixture".into(),
            consequence: "None".into(),
            confidence: Confidence::High,
            action_kind: Some(ActionKind::UserTemp),
            action_available: true,
            selected_by_default: false,
        };
        let report = ScanReport {
            scan_id: Uuid::new_v4(),
            started_at: Utc::now(),
            completed_at: Utc::now(),
            root: std::env::temp_dir().to_string_lossy().to_string(),
            drives: vec![],
            scope_fingerprint: "test-scope".into(),
            disk: DiskSnapshot {
                root: "/".into(),
                total_bytes: 1000,
                free_bytes: 500,
                used_bytes: 500,
            },
            findings: vec![finding.clone()],
            scanned_entries: 1,
            skipped_entries: 0,
            errors: vec![],
        };
        let request = CreatePlanRequest {
            scan_id: report.scan_id,
            finding_ids: vec![finding.id],
        };
        let mut plan = build_plan(&report, &request).unwrap();
        assert!(verify_plan_hash(&plan).unwrap());
        assert_eq!(plan.simulation.irreversible_bytes, 100);
        plan.estimated_reclaim_bytes += 1;
        assert!(!verify_plan_hash(&plan).unwrap());
    }
}
