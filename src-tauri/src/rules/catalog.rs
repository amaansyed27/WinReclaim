use crate::domain::{ActionKind, Confidence, Finding, RiskClass};
use crate::platform::windows::{local_app_data, roaming_app_data, user_profile};
use anyhow::Result;
use std::path::PathBuf;
use uuid::Uuid;

pub const RULE_SET_VERSION: &str = "2026.07-alpha.1";

type RuleIdentity = (&'static str, &'static str, &'static str);

#[derive(Debug, Clone)]
pub struct RuleTarget {
    pub rule_id: &'static str,
    pub display_name: &'static str,
    pub category: &'static str,
    pub path: PathBuf,
    pub risk_class: RiskClass,
    pub explanation: &'static str,
    pub consequence: &'static str,
    pub confidence: Confidence,
    pub action_kind: Option<ActionKind>,
}

impl RuleTarget {
    pub fn into_finding(self, estimated_bytes: u64) -> Finding {
        let action_available =
            self.action_kind.is_some() && self.risk_class != RiskClass::Protected;
        Finding {
            id: Uuid::new_v4(),
            rule_id: self.rule_id.to_string(),
            display_name: self.display_name.to_string(),
            category: self.category.to_string(),
            path: self.path.to_string_lossy().to_string(),
            estimated_bytes,
            risk_class: self.risk_class,
            explanation: self.explanation.to_string(),
            consequence: self.consequence.to_string(),
            confidence: self.confidence,
            action_kind: self.action_kind,
            action_available,
            selected_by_default: false,
        }
    }
}

pub fn known_targets() -> Result<Vec<RuleTarget>> {
    let user = user_profile()?;
    let local = local_app_data().unwrap_or_else(|| user.join("AppData").join("Local"));
    let roaming = roaming_app_data().unwrap_or_else(|| user.join("AppData").join("Roaming"));

    Ok(vec![
        target(
            ("windows.user_temp", "User temporary files", "Windows"),
            local.join("Temp"),
            RiskClass::SafeNow,
            "Temporary files created by applications for short-lived work.",
            "Only files older than seven days are eligible. Active and locked files are skipped.",
            Some(ActionKind::UserTemp),
        ),
        target(
            (
                "windows.crash_dumps",
                "Application crash dumps",
                "Windows diagnostics",
            ),
            local.join("CrashDumps"),
            RiskClass::SafeNow,
            "User-level diagnostic dumps left by crashed applications.",
            "Removing recognised dump files prevents later debugging of those crashes.",
            Some(ActionKind::CrashDumps),
        ),
        target(
            (
                "huggingface.cache",
                "Hugging Face cache",
                "AI and machine learning",
            ),
            user.join(".cache").join("huggingface"),
            RiskClass::RebuildOrRedownload,
            "Downloaded model, dataset and revision data managed by Hugging Face.",
            "WinReclaim only prunes detached revisions and incomplete downloads through the hf CLI; referenced models remain.",
            Some(ActionKind::HuggingfacePrune),
        ),
        target(
            ("npm.cache", "npm cache", "JavaScript tooling"),
            local.join("npm-cache"),
            RiskClass::RebuildOrRedownload,
            "Package tarballs and metadata retained by npm to speed future installs.",
            "npm will download packages again. This is for reclaiming space, not fixing npm.",
            Some(ActionKind::NpmCache),
        ),
        target(
            ("gradle.cache", "Gradle cache", "Android and JVM tooling"),
            user.join(".gradle").join("caches"),
            RiskClass::RebuildOrRedownload,
            "Dependency artifacts and build metadata used by Gradle projects.",
            "Future Android or JVM builds may download dependencies and rebuild metadata. Detection only in this alpha.",
            None,
        ),
        target(
            (
                "gradle.distributions",
                "Gradle wrapper distributions",
                "Android and JVM tooling",
            ),
            user.join(".gradle").join("wrapper").join("dists"),
            RiskClass::RebuildOrRedownload,
            "Downloaded Gradle versions used by project wrappers.",
            "Projects using removed versions will download them again. Detection only in this alpha.",
            None,
        ),
        target(
            ("cargo.registry", "Cargo package cache", "Rust tooling"),
            user.join(".cargo").join("registry"),
            RiskClass::RebuildOrRedownload,
            "Rust crate indexes and downloaded package sources.",
            "Cargo may fetch crates again. Detection only in this alpha.",
            None,
        ),
        target(
            ("pip.cache", "pip cache", "Python tooling"),
            local.join("pip").join("Cache"),
            RiskClass::RebuildOrRedownload,
            "Downloaded Python wheels and source packages.",
            "pip will download missing packages again. Detection only in this alpha.",
            None,
        ),
        target(
            ("uv.cache", "uv cache", "Python tooling"),
            local.join("uv").join("cache"),
            RiskClass::RebuildOrRedownload,
            "Cached Python packages and environments managed by uv.",
            "uv may rebuild or redownload package data. Detection only in this alpha.",
            None,
        ),
        target(
            ("bun.cache", "Bun package cache", "JavaScript tooling"),
            user.join(".bun").join("install").join("cache"),
            RiskClass::RebuildOrRedownload,
            "Packages cached by Bun for faster installs.",
            "Bun will download packages again. Detection only in this alpha.",
            None,
        ),
        target(
            (
                "playwright.cache",
                "Playwright browsers",
                "Browser automation",
            ),
            local.join("ms-playwright"),
            RiskClass::RebuildOrRedownload,
            "Browser binaries installed for Playwright tests.",
            "Playwright tests will reinstall missing browsers. Detection only in this alpha.",
            None,
        ),
        target(
            ("docker.local_data", "Docker local data", "Containers"),
            local.join("Docker"),
            RiskClass::ReviewFirst,
            "Docker Desktop storage for images, stopped containers and build cache.",
            "The conservative prune removes stopped containers, unused networks, dangling images and build cache. Volumes are never included.",
            Some(ActionKind::DockerPrune),
        ),
        target(
            ("android.avd", "Android virtual devices", "Android tooling"),
            user.join(".android").join("avd"),
            RiskClass::ReviewFirst,
            "Emulator disks, snapshots and device configuration.",
            "Removing an AVD destroys that virtual device. WinReclaim never deletes this folder directly.",
            None,
        ),
        target(
            ("android.sdk", "Android SDK", "Android tooling"),
            local.join("Android").join("Sdk"),
            RiskClass::ReviewFirst,
            "Platforms, build tools, system images and command-line tools.",
            "Remove unused packages through Android Studio SDK Manager, not by deleting this directory.",
            None,
        ),
        target(
            ("ollama.models", "Ollama models", "Local AI models"),
            user.join(".ollama").join("models"),
            RiskClass::Protected,
            "Local model blobs and manifests intentionally stored for offline inference.",
            "Protected by default. Removing a model requires an explicit future model-management action and a later download to restore it.",
            None,
        ),
        target(
            (
                "browser.chrome",
                "Google browser and app data",
                "Browser profiles",
            ),
            local.join("Google"),
            RiskClass::Protected,
            "Browser profiles, extensions, cookies, caches and application state can coexist here.",
            "Protected because raw folder cleanup can remove profiles, sessions and credentials.",
            None,
        ),
        target(
            ("browser.edge", "Microsoft Edge data", "Browser profiles"),
            local.join("Microsoft").join("Edge"),
            RiskClass::Protected,
            "Edge profiles, extensions, caches and session state.",
            "Protected because raw folder cleanup can remove profiles, sessions and credentials.",
            None,
        ),
        target(
            ("editor.cursor", "Cursor application data", "Editor data"),
            roaming.join("Cursor"),
            RiskClass::ReviewFirst,
            "Editor state, extensions, workspace history and caches are mixed together.",
            "Review manually; raw deletion can reset settings and workspace state.",
            None,
        ),
    ])
}

fn target(
    identity: RuleIdentity,
    path: PathBuf,
    risk_class: RiskClass,
    explanation: &'static str,
    consequence: &'static str,
    action_kind: Option<ActionKind>,
) -> RuleTarget {
    let (rule_id, display_name, category) = identity;
    RuleTarget {
        rule_id,
        display_name,
        category,
        path,
        risk_class,
        explanation,
        consequence,
        confidence: Confidence::High,
        action_kind,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protected_rules_never_offer_actions() {
        let finding = target(
            ("test.protected", "Protected", "Test"),
            PathBuf::from("C:\\protected"),
            RiskClass::Protected,
            "Protected data",
            "Never remove",
            Some(ActionKind::UserTemp),
        )
        .into_finding(10);
        assert!(!finding.action_available);
        assert!(!finding.selected_by_default);
    }
}
