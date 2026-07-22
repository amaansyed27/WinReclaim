mod rules;
mod selector;

use crate::domain::{Finding, IntentRequest, IntentSuggestion, RiskClass, ScanReport};
use anyhow::{anyhow, Result};

pub use rules::status as ai_status;

const MAX_PROMPT_CHARS: usize = 1_000;

pub fn interpret_intent(report: &ScanReport, request: IntentRequest) -> Result<IntentSuggestion> {
    if report.scan_id != request.scan_id {
        return Err(anyhow!(
            "The intent request does not match the current scan"
        ));
    }

    let prompt = request.prompt.trim();
    if prompt.is_empty() {
        return Err(anyhow!("Describe what you want to reclaim or protect"));
    }
    if prompt.chars().count() > MAX_PROMPT_CHARS {
        return Err(anyhow!(
            "Intent requests are limited to {MAX_PROMPT_CHARS} characters"
        ));
    }

    let candidates = report
        .findings
        .iter()
        .filter(is_executable_candidate)
        .collect::<Vec<_>>();

    if candidates.is_empty() {
        return Err(anyhow!(
            "No executable cleanup candidates are available in this scan"
        ));
    }

    let (engine, constraints) = rules::build_constraints(prompt, &candidates)?;
    selector::build_suggestion(&engine, &candidates, constraints)
}

fn is_executable_candidate(finding: &&Finding) -> bool {
    finding.action_available
        && finding.action_kind.is_some()
        && finding.risk_class != RiskClass::Protected
}
