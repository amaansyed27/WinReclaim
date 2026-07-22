use crate::domain::{AiStatus, Finding, RiskClass};
use anyhow::Result;
use uuid::Uuid;

const ENGINE_NAME: &str = "WinReclaim intent rules v1";

#[derive(Debug)]
pub(crate) struct ModelConstraints {
    pub target_reclaim_bytes: Option<u64>,
    pub allowed_risk_classes: Vec<String>,
    pub excluded_candidate_ids: Vec<Uuid>,
    pub summary: String,
}

pub fn ai_status() -> AiStatus {
    AiStatus {
        configured: true,
        model: ENGINE_NAME.to_string(),
        privacy_note: "Reclaim-by-intent is interpreted locally with deterministic rules. No prompt, path, filename, project name, scan metadata, or API key leaves this PC."
            .to_string(),
    }
}

pub(crate) fn request_constraints(
    prompt: &str,
    candidates: &[&Finding],
) -> Result<(String, ModelConstraints)> {
    let normalized = normalize(prompt);
    let target_reclaim_bytes = parse_target_bytes(&normalized);

    let mut allowed_risk_classes = vec!["safe_now".to_string()];
    if accepts_rebuildable(&normalized) {
        allowed_risk_classes.push("rebuild_or_redownload".to_string());
    }
    if explicitly_accepts_review_first(&normalized) {
        allowed_risk_classes.push("review_first".to_string());
    }

    let excluded_candidate_ids = candidates
        .iter()
        .filter(|finding| category_is_excluded(&normalized, &finding.category))
        .map(|finding| finding.id)
        .collect::<Vec<_>>();

    let risk_description = match allowed_risk_classes.len() {
        1 => "low-impact actions only",
        2 => "low-impact and rebuildable/redownloadable actions",
        _ => "low-impact, rebuildable/redownloadable, and explicitly requested review-first actions",
    };
    let target_description = target_reclaim_bytes
        .map(|bytes| format!(" toward a target of {}", format_bytes(bytes)))
        .unwrap_or_default();
    let exclusion_description = if excluded_candidate_ids.is_empty() {
        String::new()
    } else {
        format!(
            ", while excluding {} matching candidate{}",
            excluded_candidate_ids.len(),
            plural(excluded_candidate_ids.len())
        )
    };

    let summary = format!(
        "Local deterministic intent rules selected {risk_description}{target_description}{exclusion_description}. Review the editable suggestion before creating a cleanup plan."
    );

    Ok((
        ENGINE_NAME.to_string(),
        ModelConstraints {
            target_reclaim_bytes,
            allowed_risk_classes,
            excluded_candidate_ids,
            summary,
        },
    ))
}

fn accepts_rebuildable(prompt: &str) -> bool {
    contains_any(
        prompt,
        &[
            "rebuild",
            "redownload",
            "re-download",
            "cache",
            "caches",
            "dependency",
            "dependencies",
            "developer files",
            "build output",
            "build outputs",
            "npm",
            "cargo",
            "gradle",
            "pip",
            "docker",
        ],
    ) && !contains_any(
        prompt,
        &[
            "only safest",
            "safe only",
            "low impact only",
            "do not redownload",
            "don't redownload",
            "no rebuild",
        ],
    )
}

fn explicitly_accepts_review_first(prompt: &str) -> bool {
    contains_any(
        prompt,
        &[
            "include review first",
            "include review-first",
            "allow review first",
            "allow review-first",
            "show review first",
            "show review-first",
        ],
    )
}

fn category_is_excluded(prompt: &str, category: &str) -> bool {
    let category = normalize(category);
    if category.is_empty() {
        return false;
    }

    let phrases = [
        "do not touch",
        "don't touch",
        "dont touch",
        "exclude",
        "keep",
        "protect",
        "avoid",
        "leave",
        "skip",
    ];

    phrases
        .iter()
        .any(|phrase| prompt.contains(&format!("{phrase} {category}")))
        || category
            .split_whitespace()
            .filter(|token| token.len() >= 4)
            .any(|token| {
                phrases
                    .iter()
                    .any(|phrase| prompt.contains(&format!("{phrase} {token}")))
            })
}

fn parse_target_bytes(prompt: &str) -> Option<u64> {
    let tokens = tokenize(prompt);
    for pair in tokens.windows(2) {
        let Ok(value) = pair[0].parse::<f64>() else {
            continue;
        };
        if !value.is_finite() || value <= 0.0 {
            continue;
        }

        let multiplier = match pair[1].as_str() {
            "kb" | "kib" => 1024_f64,
            "mb" | "mib" => 1024_f64.powi(2),
            "gb" | "gib" => 1024_f64.powi(3),
            "tb" | "tib" => 1024_f64.powi(4),
            _ => continue,
        };
        let bytes = value * multiplier;
        if bytes <= u64::MAX as f64 {
            return Some(bytes.round() as u64);
        }
    }
    None
}

fn tokenize(value: &str) -> Vec<String> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Kind {
        Number,
        Letter,
    }

    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut current_kind: Option<Kind> = None;

    for character in value.chars() {
        let next_kind = if character.is_ascii_digit() || character == '.' {
            Some(Kind::Number)
        } else if character.is_ascii_alphabetic() {
            Some(Kind::Letter)
        } else {
            None
        };

        match next_kind {
            Some(kind) if current_kind == Some(kind) => current.push(character),
            Some(kind) => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
                current.push(character);
                current_kind = Some(kind);
            }
            None => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
                current_kind = None;
            }
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

fn contains_any(value: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| value.contains(needle))
}

fn normalize(value: &str) -> String {
    value
        .to_ascii_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_bytes(bytes: u64) -> String {
    const GIB: f64 = 1024.0 * 1024.0 * 1024.0;
    const TIB: f64 = GIB * 1024.0;
    let value = bytes as f64;
    if value >= TIB {
        format!("{:.2} TiB", value / TIB)
    } else {
        format!("{:.2} GiB", value / GIB)
    }
}

fn plural(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_compact_and_spaced_targets() {
        assert_eq!(parse_target_bytes("free 20 gb"), Some(20 * 1024_u64.pow(3)));
        assert_eq!(parse_target_bytes("free 1.5TB"), Some((1.5 * 1024_f64.powi(4)) as u64));
    }

    #[test]
    fn defaults_to_low_impact_only() {
        assert!(!accepts_rebuildable("free some space safely"));
        assert!(accepts_rebuildable("include rebuildable caches"));
    }

    #[test]
    fn review_first_requires_explicit_phrase() {
        assert!(!explicitly_accepts_review_first("review the results"));
        assert!(explicitly_accepts_review_first("include review-first actions"));
    }

    #[test]
    fn exclusions_match_category_words() {
        assert!(category_is_excluded(
            "free 10 gb but do not touch browser caches",
            "Browser caches"
        ));
        assert!(!category_is_excluded("include browser caches", "Browser caches"));
    }

    #[test]
    fn status_is_local_and_configured() {
        let status = ai_status();
        assert!(status.configured);
        assert!(status.privacy_note.contains("locally"));
    }
}
