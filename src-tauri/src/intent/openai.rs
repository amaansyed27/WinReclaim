use crate::domain::{AiStatus, Finding, RiskClass};
use anyhow::{anyhow, Context, Result};
use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::{json, Value};
use std::time::Duration;
use uuid::Uuid;

const OPENAI_RESPONSES_URL: &str = "https://api.openai.com/v1/responses";
const DEFAULT_MODEL: &str = "gpt-5.6";

#[derive(Debug, Deserialize)]
pub(crate) struct ModelConstraints {
    pub target_reclaim_bytes: Option<u64>,
    pub allowed_risk_classes: Vec<String>,
    pub excluded_candidate_ids: Vec<Uuid>,
    pub summary: String,
}

pub fn ai_status() -> AiStatus {
    AiStatus {
        configured: openai_api_key().is_some(),
        model: configured_model(),
        privacy_note: "Only anonymized category, size, risk and consequence metadata is sent. Paths, usernames and project names stay local."
            .to_string(),
    }
}

pub(crate) fn request_constraints(
    prompt: &str,
    candidates: &[&Finding],
) -> Result<(String, ModelConstraints)> {
    let api_key = openai_api_key().ok_or_else(|| {
        anyhow!("Set OPENAI_API_KEY before launching WinReclaim to use reclaim-by-intent")
    })?;
    let model = configured_model();
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
                                { "type": "integer" },
                                { "type": "null" }
                            ]
                        },
                        "allowed_risk_classes": {
                            "type": "array",
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
                            "items": {
                                "type": "string",
                                "enum": candidate_ids
                            }
                        },
                        "summary": { "type": "string" }
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

    let response = Client::builder()
        .timeout(Duration::from_secs(45))
        .build()?
        .post(OPENAI_RESPONSES_URL)
        .bearer_auth(api_key)
        .header("X-Client-Request-Id", Uuid::new_v4().to_string())
        .json(&body)
        .send()
        .context("OpenAI intent request failed")?;

    let status = response.status();
    let response_body = response
        .text()
        .context("Unable to read the OpenAI response")?;
    if !status.is_success() {
        let detail = response_body.chars().take(500).collect::<String>();
        return Err(anyhow!("OpenAI returned {status}: {detail}"));
    }

    let response_json: Value = serde_json::from_str(&response_body)
        .context("OpenAI returned an invalid JSON response envelope")?;
    if response_json.get("status").and_then(Value::as_str) != Some("completed") {
        return Err(anyhow!("OpenAI did not complete the intent request"));
    }

    let output = extract_output_text(&response_json)
        .ok_or_else(|| anyhow!("OpenAI response did not contain structured output text"))?;
    let constraints =
        serde_json::from_str(output).context("Unable to parse OpenAI intent constraints")?;

    Ok((model, constraints))
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

fn risk_name(risk: RiskClass) -> &'static str {
    match risk {
        RiskClass::SafeNow => "safe_now",
        RiskClass::RebuildOrRedownload => "rebuild_or_redownload",
        RiskClass::ReviewFirst => "review_first",
        RiskClass::Protected => "protected",
    }
}
