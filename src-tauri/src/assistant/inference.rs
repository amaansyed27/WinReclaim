use super::{BASE_MODEL, MODEL_NAME, QUANTIZATION};
use crate::domain::{AssistantAnnotation, ScanReport, StorageAssistantReport};
use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;
use llama_cpp_2::{send_logs_to_tracing, LogOptions};
use serde::Deserialize;
use std::collections::HashSet;
use std::num::NonZeroU32;
use std::path::Path;
use uuid::Uuid;

const CONTEXT_TOKENS: u32 = 8_192;
const MAX_ANNOTATIONS: usize = 15;
const ALLOWED_GROUPS: &[&str] = &[
    "Windows and system",
    "Browsers and web runtimes",
    "Developer tools and package managers",
    "Android development",
    "Media and recordings",
    "Projects and downloads",
    "Installed applications",
    "User data",
    "Other large locations",
];

#[derive(Debug, Deserialize)]
struct RawReport {
    summary: String,
    #[serde(default)]
    observations: Vec<String>,
    #[serde(default)]
    annotations: Vec<RawAnnotation>,
}

#[derive(Debug, Deserialize)]
struct RawAnnotation {
    finding_id: Uuid,
    suggested_name: String,
    group: String,
    explanation: String,
    confidence: f32,
}

pub fn generate(model_path: &Path, prompt: &str, max_output_tokens: i32) -> Result<String> {
    if !model_path.is_file() {
        return Err(anyhow!(
            "Install the optional Storage Assistant model in Settings before generating a report"
        ));
    }

    send_logs_to_tracing(LogOptions::default().with_logs_enabled(false));
    let backend = LlamaBackend::init().context("Unable to initialize the local model runtime")?;
    let model_params = LlamaModelParams::default();
    let model = LlamaModel::load_from_file(&backend, model_path, &model_params)
        .context("Unable to load the Storage Assistant model")?;

    let threads = std::thread::available_parallelism()
        .map(|count| count.get().min(8) as i32)
        .unwrap_or(4);
    let context_params = LlamaContextParams::default()
        .with_n_ctx(Some(
            NonZeroU32::new(CONTEXT_TOKENS).expect("context is non-zero"),
        ))
        .with_n_threads(threads)
        .with_n_threads_batch(threads);
    let mut context = model
        .new_context(&backend, context_params)
        .context("Unable to allocate local model context")?;

    let prompt_tokens = model
        .str_to_token(prompt, AddBos::Never)
        .context("Unable to tokenize the storage report")?;
    if prompt_tokens.is_empty() {
        return Err(anyhow!("The Storage Assistant prompt was empty"));
    }

    let max_context = i32::try_from(context.n_ctx()).unwrap_or(i32::MAX);
    let requested = i32::try_from(prompt_tokens.len())?.saturating_add(max_output_tokens.max(1));
    if requested > max_context {
        return Err(anyhow!(
            "The scan report is too large for the local assistant context"
        ));
    }

    let mut batch = LlamaBatch::new(prompt_tokens.len().max(512), 1);
    let last_index = i32::try_from(prompt_tokens.len().saturating_sub(1))?;
    for (index, token) in (0_i32..).zip(prompt_tokens.into_iter()) {
        batch.add(token, index, &[0], index == last_index)?;
    }
    context
        .decode(&mut batch)
        .context("The local model could not process the scan report")?;

    let mut sampler = LlamaSampler::greedy();
    let mut decoder = encoding_rs::UTF_8.new_decoder();
    let mut output = String::new();
    let mut current = batch.n_tokens();
    let end = current.saturating_add(max_output_tokens.max(1));

    while current < end {
        let token = sampler.sample(&context, batch.n_tokens() - 1);
        sampler.accept(token);
        if model.is_eog_token(token) {
            break;
        }

        output.push_str(
            &model
                .token_to_piece(token, &mut decoder, true, None)
                .context("Unable to decode local model output")?,
        );
        batch.clear();
        batch.add(token, current, &[0], true)?;
        current = current.saturating_add(1);
        context
            .decode(&mut batch)
            .context("Local model generation failed")?;
    }

    if output.trim().is_empty() {
        return Err(anyhow!("The Storage Assistant returned an empty report"));
    }
    Ok(output)
}

pub fn parse_report(report: &ScanReport, output: &str) -> Result<StorageAssistantReport> {
    let json = extract_json(output)?;
    let raw: RawReport = serde_json::from_str(json)
        .context("The Storage Assistant returned malformed structured output")?;
    let finding_ids = report
        .findings
        .iter()
        .map(|finding| finding.id)
        .collect::<HashSet<_>>();

    let annotations = raw
        .annotations
        .into_iter()
        .filter(|annotation| finding_ids.contains(&annotation.finding_id))
        .filter(|annotation| ALLOWED_GROUPS.contains(&annotation.group.trim()))
        .filter(|annotation| !contains_cleanup_claim(&annotation.explanation))
        .filter_map(|annotation| {
            let finding = report
                .findings
                .iter()
                .find(|finding| finding.id == annotation.finding_id)?;
            if !is_unclear_finding(finding.display_name.as_str(), finding.category.as_str()) {
                return None;
            }
            let suggested_name = clean_line(&annotation.suggested_name, 80);
            let explanation = clean_line(&annotation.explanation, 240);
            if suggested_name.len() < 3
                || explanation.len() < 8
                || suggested_name.eq_ignore_ascii_case(&finding.display_name)
            {
                return None;
            }
            Some(AssistantAnnotation {
                finding_id: annotation.finding_id,
                suggested_name,
                group: annotation.group.trim().to_string(),
                explanation,
                confidence: annotation.confidence.clamp(0.0, 1.0),
            })
        })
        .take(MAX_ANNOTATIONS)
        .collect::<Vec<_>>();

    let summary = clean_line(&raw.summary, 700);
    if summary.len() < 20 {
        return Err(anyhow!(
            "The Storage Assistant summary was too short to be useful"
        ));
    }
    let observations = raw
        .observations
        .into_iter()
        .map(|observation| clean_line(&observation, 260))
        .filter(|observation| observation.len() >= 8)
        .filter(|observation| !contains_cleanup_claim(observation))
        .take(6)
        .collect::<Vec<_>>();

    Ok(StorageAssistantReport {
        scan_id: report.scan_id,
        generated_at: Utc::now(),
        model: format!("{MODEL_NAME} ({BASE_MODEL} {QUANTIZATION})"),
        summary,
        observations,
        annotations,
        advisory_only: true,
    })
}

fn extract_json(output: &str) -> Result<&str> {
    let start = output
        .find('{')
        .ok_or_else(|| anyhow!("The Storage Assistant output did not contain JSON"))?;
    let end = output
        .rfind('}')
        .ok_or_else(|| anyhow!("The Storage Assistant output did not contain complete JSON"))?;
    if end < start {
        return Err(anyhow!("The Storage Assistant JSON was incomplete"));
    }
    Ok(&output[start..=end])
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

fn is_unclear_finding(name: &str, category: &str) -> bool {
    let normalized = name.trim().to_ascii_lowercase();
    category.to_ascii_lowercase().contains("unclassified")
        || normalized
            .chars()
            .all(|character| character.is_ascii_digit())
        || normalized.len() >= 24
            && normalized
                .chars()
                .all(|character| character.is_ascii_hexdigit())
        || matches!(
            normalized.as_str(),
            "bin"
                | "cache"
                | "caches"
                | "data"
                | "downloads"
                | "index"
                | "packages"
                | "plugins"
                | "programs"
                | "saved"
                | "share"
                | "tmp"
                | "user data"
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Confidence, Finding, RiskClass};

    #[test]
    fn cleanup_claims_are_rejected() {
        assert!(contains_cleanup_claim("This is safe to delete."));
        assert!(!contains_cleanup_claim(
            "This appears to be application state."
        ));
    }

    #[test]
    fn parser_cannot_create_actions() {
        let finding_id = Uuid::new_v4();
        let report = ScanReport {
            scan_id: Uuid::new_v4(),
            started_at: Utc::now(),
            completed_at: Utc::now(),
            root: "C:\\".to_string(),
            drives: vec![],
            scope_fingerprint: "test".to_string(),
            disk: crate::domain::DiskSnapshot {
                root: "C:\\".to_string(),
                total_bytes: 100,
                free_bytes: 50,
                used_bytes: 50,
            },
            findings: vec![Finding {
                id: finding_id,
                rule_id: "dynamic.unknown".to_string(),
                display_name: "2026".to_string(),
                category: "Unclassified".to_string(),
                path: "C:\\Sessions\\2026".to_string(),
                estimated_bytes: 10,
                risk_class: RiskClass::ReviewFirst,
                explanation: "Unknown".to_string(),
                consequence: "Not cleaned".to_string(),
                confidence: Confidence::Low,
                action_kind: None,
                action_available: false,
                selected_by_default: false,
            }],
            scanned_entries: 1,
            skipped_entries: 0,
            errors: vec![],
        };
        let output = format!(
            "{{\"summary\":\"The scan contains one unclear storage location requiring manual interpretation.\",\"observations\":[],\"annotations\":[{{\"finding_id\":\"{finding_id}\",\"suggested_name\":\"Session archive — 2026\",\"group\":\"Developer tools and package managers\",\"explanation\":\"This likely contains session records based on its parent path.\",\"confidence\":0.8}}]}}"
        );
        let parsed = parse_report(&report, &output).unwrap();
        assert_eq!(parsed.annotations.len(), 1);
        assert!(!report.findings[0].action_available);
        assert!(report.findings[0].action_kind.is_none());
    }
}
