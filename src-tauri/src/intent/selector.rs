use super::openrouter::ModelConstraints;
use crate::domain::{Finding, IntentSuggestion, RiskClass};
use anyhow::{anyhow, Result};
use std::collections::HashSet;
use uuid::Uuid;

const MAX_SUMMARY_CHARS: usize = 280;

pub(crate) fn build_suggestion(
    model: &str,
    candidates: &[&Finding],
    constraints: ModelConstraints,
) -> Result<IntentSuggestion> {
    let allowed_risk_classes = validate_risk_classes(&constraints.allowed_risk_classes)?;
    let allowed = allowed_risk_classes.iter().copied().collect::<HashSet<_>>();
    let candidate_ids = candidates
        .iter()
        .map(|finding| finding.id)
        .collect::<HashSet<_>>();
    let excluded = validate_exclusions(&constraints.excluded_candidate_ids, &candidate_ids)?;
    let summary = validate_summary(&constraints.summary)?;

    let mut eligible = candidates
        .iter()
        .copied()
        .filter(|finding| allowed.contains(&finding.risk_class))
        .filter(|finding| !excluded.contains(&finding.id))
        .collect::<Vec<_>>();

    eligible.sort_by(|left, right| {
        risk_rank(left.risk_class)
            .cmp(&risk_rank(right.risk_class))
            .then_with(|| right.estimated_bytes.cmp(&left.estimated_bytes))
    });

    let mut selected_finding_ids = Vec::new();
    let mut estimated_reclaim_bytes = 0_u64;
    for finding in eligible {
        if constraints
            .target_reclaim_bytes
            .is_some_and(|target| estimated_reclaim_bytes >= target)
        {
            break;
        }

        selected_finding_ids.push(finding.id);
        estimated_reclaim_bytes = estimated_reclaim_bytes.saturating_add(finding.estimated_bytes);
    }

    let excluded_labels = candidates
        .iter()
        .filter(|finding| excluded.contains(&finding.id))
        .map(|finding| finding.display_name.clone())
        .collect::<Vec<_>>();

    let summary = match constraints.target_reclaim_bytes {
        Some(target) if estimated_reclaim_bytes < target => format!(
            "{summary} The available approved actions total {estimated_reclaim_bytes} bytes, below the requested {target} bytes."
        ),
        _ => summary,
    };

    Ok(IntentSuggestion {
        selected_finding_ids,
        target_reclaim_bytes: constraints.target_reclaim_bytes,
        estimated_reclaim_bytes,
        allowed_risk_classes,
        excluded_labels,
        summary,
        model: model.to_string(),
        remote_used: true,
    })
}

fn validate_risk_classes(values: &[String]) -> Result<Vec<RiskClass>> {
    let mut seen = HashSet::new();
    let mut parsed = Vec::new();

    for value in values {
        let risk = parse_risk(value)?;
        if seen.insert(risk) {
            parsed.push(risk);
        }
    }

    if parsed.is_empty() {
        return Err(anyhow!(
            "The routed model returned no allowed cleanup risk classes"
        ));
    }

    Ok(parsed)
}

fn validate_exclusions(values: &[Uuid], candidate_ids: &HashSet<Uuid>) -> Result<HashSet<Uuid>> {
    let mut excluded = HashSet::new();
    for value in values {
        if !candidate_ids.contains(value) {
            return Err(anyhow!(
                "The routed model referenced an unknown cleanup candidate"
            ));
        }
        excluded.insert(*value);
    }
    Ok(excluded)
}

fn validate_summary(value: &str) -> Result<String> {
    let summary = value.trim();
    if summary.is_empty() {
        return Err(anyhow!("The routed model returned an empty explanation"));
    }
    if summary.chars().count() > MAX_SUMMARY_CHARS {
        return Err(anyhow!(
            "The routed model returned an unexpectedly long explanation"
        ));
    }
    Ok(summary.to_string())
}

fn parse_risk(value: &str) -> Result<RiskClass> {
    match value {
        "safe_now" => Ok(RiskClass::SafeNow),
        "rebuild_or_redownload" => Ok(RiskClass::RebuildOrRedownload),
        "review_first" => Ok(RiskClass::ReviewFirst),
        _ => Err(anyhow!(
            "The routed model returned an unsupported risk class"
        )),
    }
}

fn risk_rank(risk: RiskClass) -> u8 {
    match risk {
        RiskClass::SafeNow => 0,
        RiskClass::RebuildOrRedownload => 1,
        RiskClass::ReviewFirst => 2,
        RiskClass::Protected => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{ActionKind, Confidence};

    fn finding(name: &str, bytes: u64, risk: RiskClass) -> Finding {
        Finding {
            id: Uuid::new_v4(),
            rule_id: format!("test.{name}"),
            display_name: name.to_string(),
            category: "Test".to_string(),
            path: String::new(),
            estimated_bytes: bytes,
            risk_class: risk,
            explanation: "Fixture".to_string(),
            consequence: "Fixture consequence".to_string(),
            confidence: Confidence::High,
            action_kind: Some(ActionKind::UserTemp),
            action_available: true,
            selected_by_default: false,
        }
    }

    #[test]
    fn selector_respects_risk_and_target() {
        let safe = finding("safe", 40, RiskClass::SafeNow);
        let rebuild = finding("rebuild", 80, RiskClass::RebuildOrRedownload);
        let review = finding("review", 100, RiskClass::ReviewFirst);
        let candidates = vec![&safe, &rebuild, &review];

        let suggestion = build_suggestion(
            "test-model",
            &candidates,
            ModelConstraints {
                target_reclaim_bytes: Some(100),
                allowed_risk_classes: vec![
                    "safe_now".to_string(),
                    "rebuild_or_redownload".to_string(),
                ],
                excluded_candidate_ids: vec![],
                summary: "Fixture".to_string(),
            },
        )
        .unwrap();

        assert_eq!(suggestion.selected_finding_ids, vec![safe.id, rebuild.id]);
        assert_eq!(suggestion.estimated_reclaim_bytes, 120);
        assert!(!suggestion.selected_finding_ids.contains(&review.id));
    }

    #[test]
    fn selector_rejects_unknown_candidate_ids() {
        let safe = finding("safe", 40, RiskClass::SafeNow);
        let candidates = vec![&safe];
        let result = build_suggestion(
            "test-model",
            &candidates,
            ModelConstraints {
                target_reclaim_bytes: None,
                allowed_risk_classes: vec!["safe_now".to_string()],
                excluded_candidate_ids: vec![Uuid::new_v4()],
                summary: "Fixture".to_string(),
            },
        );

        assert!(result.is_err());
    }

    #[test]
    fn selector_rejects_empty_risk_set() {
        let safe = finding("safe", 40, RiskClass::SafeNow);
        let candidates = vec![&safe];
        let result = build_suggestion(
            "test-model",
            &candidates,
            ModelConstraints {
                target_reclaim_bytes: None,
                allowed_risk_classes: vec![],
                excluded_candidate_ids: vec![],
                summary: "Fixture".to_string(),
            },
        );

        assert!(result.is_err());
    }
}
