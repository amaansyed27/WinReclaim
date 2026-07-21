use crate::cloud;
use crate::domain::{AiStatus, Finding, RiskClass};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const ROUTER_NAME: &str = "OpenRouter Free Models Router";
const ROUTER_MODEL: &str = "openrouter/free";
const MAX_CANDIDATES: usize = 200;

#[derive(Debug, Deserialize)]
pub(crate) struct ModelConstraints {
    pub target_reclaim_bytes: Option<u64>,
    pub allowed_risk_classes: Vec<String>,
    pub excluded_candidate_ids: Vec<Uuid>,
    pub summary: String,
}

#[derive(Debug, Serialize)]
struct IntentPayload<'a> {
    prompt: &'a str,
    candidates: Vec<IntentCandidate>,
}

#[derive(Debug, Serialize)]
struct IntentCandidate {
    candidate_id: Uuid,
    category: String,
    size_bytes: u64,
    risk_class: &'static str,
    consequence: String,
}

pub fn ai_status() -> AiStatus {
    AiStatus {
        configured: cloud::assistant_endpoint().starts_with("https://"),
        model: ROUTER_MODEL.to_string(),
        privacy_note: "Requests use the WinReclaim cloud proxy and OpenRouter's free model router. Candidate IDs, category, size, risk and consequence are sent; paths, usernames, folder names, project names and file contents stay local."
            .to_string(),
    }
}

pub(crate) fn request_constraints(
    prompt: &str,
    candidates: &[&Finding],
) -> Result<(String, ModelConstraints)> {
    let payload = IntentPayload {
        prompt,
        candidates: candidates
            .iter()
            .take(MAX_CANDIDATES)
            .map(|finding| IntentCandidate {
                candidate_id: finding.id,
                category: finding.category.clone(),
                size_bytes: finding.estimated_bytes,
                risk_class: risk_name(finding.risk_class),
                consequence: finding.consequence.clone(),
            })
            .collect(),
    };

    let (model, constraints) = cloud::request("intent_constraints", &payload)?;
    Ok((format!("{ROUTER_NAME} · {model}"), constraints))
}

fn risk_name(risk: RiskClass) -> &'static str {
    match risk {
        RiskClass::SafeNow => "safe_now",
        RiskClass::RebuildOrRedownload => "rebuild_or_redownload",
        RiskClass::ReviewFirst => "review_first",
        RiskClass::Protected => "protected",
    }
}
