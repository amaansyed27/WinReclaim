use super::{BASE_MODEL, MODEL_NAME, QUANTIZATION};
use crate::domain::{AssistantAnnotation, ScanReport, StorageAssistantReport};
use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
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

pub fn generate(
    runtime_path: &Path,
    model_path: &Path,
    prompt: &str,
    max_output_tokens: i32,
) -> Result<String> {
    if !runtime_path.is_file() {
        return Err(anyhow!(
            "Install the optional Storage Assistant runtime in Settings before generating a report"
        ));
    }
    if !model_path.is_file() {
        return Err(anyhow!(
            "Install the optional Storage Assistant model in Settings before generating a report"
        ));
    }
    if prompt.trim().is_empty() {
        return Err(anyhow!("The Storage Assistant prompt was empty"));
    }

    let request_root = super::model_root().join("requests");
    fs::create_dir_all(&request_root)?;
    let prompt_path = request_root.join(format!("{}.txt", Uuid::new_v4()));
    fs::write(&prompt_path, prompt).context("Unable to prepare the local assistant request")?;

    let threads = std::thread::available_parallelism()
        .map(|count| count.get().min(8))
        .unwrap_or(4);
    let mut command = Command::new(runtime_path);
    command
        .current_dir(runtime_path.parent().unwrap_or_else(|| Path::new(".")))
        .arg("--model")
        .arg(model_path)
        .arg("--file")
        .arg(&prompt_path)
        .arg("--ctx-size")
        .arg(CONTEXT_TOKENS.to_string())
        .arg("--n-predict")
        .arg(max_output_tokens.max(1).to_string())
        .arg("--threads")
        .arg(threads.to_string())
        .arg("--temp")
        .arg("0")
        .arg("--no-conversation")
        .arg("--single-turn")
        .arg("--no-display-prompt")
        .arg("--no-show-timings")
        .arg("--no-warmup")
        .arg("--log-disable")
        .arg("--color")
        .arg("off")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }

    let result = command.output();
    let _ = fs::remove_file(&prompt_path);
    let output = result.context("Unable to launch the optional llama.cpp runtime")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!(
            "The local Storage Assistant runtime failed: {}",
            compact_process_error(&stderr)
        ));
    }

    let generated = String::from_utf8(output.stdout)
        .context("The local Storage Assistant returned invalid UTF-8 output")?;
    if generated.trim().is_empty() {
        return Err(anyhow!("The Storage Assistant returned an empty report"));
    }
    Ok(generated)
}

fn compact_process_error(stderr: &str) -> String {
    let compact = stderr.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.is_empty() {
        "no diagnostic output was provided".to_string()
    } else {
        compact.chars().take(600).collect()
    }
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
    fn process_errors_are_bounded() {
        let message = compact_process_error(&"failure ".repeat(500));
        assert!(message.len() <= 600);
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
