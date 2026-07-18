use crate::domain::{
    AiStatus, Finding, IntentRequest, IntentSuggestion, RiskClass, ScanReport,
};
use anyhow::{anyhow, Context, Result};
use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashSet;
use std::time::Duration;
use uuid::Uuid;

const OPENAI_RESPONSES_URL: &str = "https://api.openai.com/v1/responses";
const DEFAULT_MODEL: &str = "gpt-5.6";
const MAX_PROMPT_CHARS: usize = 1_000;

#[derive(Debug, Deserialize)]
struct ModelConstraints {
    target_reclaim_bytes: Option<u64>,
    allowed_risk_classes: Vec<String>,
    excluded_candidate_ids: Vec<Uuid>,
    summary: String,
}

pub fn ai_status() -> AiStatus {
    AiStatus {
        configured: openai_api_key().is_some(),
        model: configured_model(),
        privacy_note: "Only anonymized category, size, risk and consequence metadata is sent. Paths, usernames and project names stay local."
            .to_string(),
    }
}

pub fn interpret_intent(report: &ScanReport, request: IntentRequest) -> Result<IntentSuggestion> {
    if report.scan_id != request.scan_id {
        return Err(anyhow!("The intent request does not match the current scan"));
    }

    let prompt = request.prompt.trim();
    if prompt.is_empty() {
        return Err(anyhow!("Describe what you want to reclaim or protect"));
    }
    if prompt.chars().count() > MAX_PROMPT_CHARS {
        return Err(anyhow!("Intent requests are limited to {MAX_PROMPT_CHARS} characters"));
    }

    let candidates = report
        .findings
        .iter()
        .filter(|finding| {
            finding.action_available
                && finding.action_kind.is_some()
                && finding.risk_class != RiskClass::Protected
        })
        .collect::<Vec<_>>();

    if candidates.is_empty() {
        return Err(anyhow!("No executable cleanup candidates are available in this scan"));
    }

    let api_key = openai_api_key().ok_or_else(|| {
        anyhow!("Set OPENAI_API_KEY before launching WinReclaim to use reclaim-by-intent")
    })?;
    let model = configured_model();
    let constraints = request_constraints(&api_key, &model, prompt, &candidates)?;
    build_suggestion(&model, &candidates, constraints)
}

fn request_constraints(
    api_key: &str,
    model: &str,
    prompt: &str,
    candidates: &[&Finding],
) -> Result<ModelConstraints> {
    let candidate_ids = candidates
        .iter()
        .map(|finding| Value::String(finding.id.to_string()))
        .collect::<Vec<_>>();

    let anonymized_candidates = candidates
        .iter()
        .map(|finding| {
            json!({
                "candidate_id": finding.id,
                "label": finding.display_name,
                "category": finding.category,
                "size_bytes": finding.estimated_bytes,
                "risk_class": risk_name(finding.risk_class),
                "consequence": finding.consequence,
            })
        })
        .collect::<Vec<_>>();

    let body = json!({
        "model": model,
        "store": false,
        "reasoning": { "effort": "low" },
        "instructions": "Interpret a Windows storage reclaim request into conservative constraints. Return constraints only. Never invent paths or commands. Never permit protected data. Default to safe_now when risk tolerance is ambiguous. Permit rebuild_or_redownload only when the user accepts later downloads or rebuilds. Permit review_first only when the user explicitly requests a named review category. Exclusions must be conservative.",
        "input": format!(
            "User request:\n{}\n\nAvailable anonymized cleanup candidates:\n{}",
            prompt,
            serde_json::to_string_pretty(&anonymized_candidates)?
        ),
        "text": {
            "verbosity": "low",
            "format": {
                "type": "json_schema",
                "name": "winreclaim_intent_constraints",
                "description": "Conservative constraints for deterministic local cleanup planning.",
                "strict": true,
                "schema": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "target_reclaim_bytes": {
                            "anyOf": [
                                { "type": "integer", "minimum": 0 },
                                { "type": "null" }
                            ]
                        },
                        "allowed_risk_classes": {
                            "type": "array",
                            "minItems": 1,
                            "uniqueItems": true,
                            "items": {
                                "type": "string",
                                "enum": [
                                    "safe_now",
                                    "rebuild_or_redownload",
                                    "review_first"
                                ]
                            }
                        },
                        "excluded_candidate_ids": {
                            "type": "array",
                            "uniqueItems": true,
                            "items": {
                                "type": "string",
                                "enum": candidate_ids
                            }
                        },
                        "summary": {
                            "type": "string",
                            "minLength": 1,
                            "maxLength": 280
                        }
                    },
                    "required": [
                        "target_reclaim_bytes",
                        "allowed_risk_classes",
                        "excluded_candidate_ids",
                        "summary"
                    ]
                }
            }
        }
    });

    let client_request_id = Uuid::new_v4().to_string();
    let response = Client::builder()
        .timeout(Duration::from_secs(45))
        .build()?
        .post(OPENAI_RESPONSES_URL)
        .bearer_auth(api_key)
        .header("X-Client-Request-Id", client_request_id)
        .json(&body)
        .send()
        .context("OpenAI intent request failed")?;

    let status = response.status();
    let response_body = response.text().context("Unable to read the OpenAI response")?;
    if !status.is_success() {
        let detail = response_body.chars().take(500).collect::<String>();
        return Err(anyhow!("OpenAI returned {status}: {detail}"));
    }

    let response_json: Value = serde_json::from_str(&response_body)
        .context("OpenAI returned an invalid JSON response envelope")?;
    let output = extract_output_text(&response_json)
        .ok_or_else(|| anyhow!("OpenAI response did not contain structured output text"))?;

    serde_json::from_str(output).context("Unable to parse OpenAI intent constraints")
}

fn build_suggestion(
    model: &str,
    candidates: &[&Finding],
    constraints: ModelConstraints,
) -> Result<IntentSuggestion> {
    let allowed_risk_classes = constraints
        .allowed_risk_classes
        .iter()
        .map(|value| parse_risk(value))
        .collect::<Result<Vec<_>>>()?;
    let allowed = allowed_risk_classes.iter().copied().collect::<HashSet<_>>();
    let excluded = constraints
        .excluded_candidate_ids
        .iter()
        .copied()
        .collect::<HashSet<_>>();

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
            "{} The available approved actions reach {} bytes, below the requested {} bytes.",
            constraints.summary, estimated_reclaim_bytes, target
        ),
        _ => constraints.summary,
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

fn extract_output_text(response: &Value) -> Option<&str> {
    response
        .get("output")?
        .as_array()?
        .iter()
        .filter_map(|item| item.get("content")?.as_array())
        .flatten()
        .find(|content| content.get("type").and_then(Value::as_str) == Some("output_text"))?
        .get("text")?
        .as_str()
}

fn openai_api_key() -> Option<String> {
    std::env::var("OPENAI_API_KEY")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn configured_model() -> String {
    std::env::var("OPENAI_MODEL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| DEFAULT_MODEL.to_string())
}

fn parse_risk(value: &str) -> Result<RiskClass> {
    match value {
        "safe_now" => Ok(RiskClass::SafeNow),
        "rebuild_or_redownload" => Ok(RiskClass::RebuildOrRedownload),
        "review_first" => Ok(RiskClass::ReviewFirst),
        _ => Err(anyhow!("OpenAI returned an unsupported risk class")),
    }
}

fn risk_name(risk: RiskClass) -> &'static str {
    match risk {
        RiskClass::SafeNow => "safe_now",
        RiskClass::RebuildOrRedownload => "rebuild_or_redownload",
        RiskClass::ReviewFirst => "review_first",
        RiskClass::Protected => "protected",
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
    fn deterministic_selector_respects_risk_and_target() {
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
}
