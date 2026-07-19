from __future__ import annotations

from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    target = Path(path)
    text = target.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"Expected one match in {path}, found {count}: {old[:100]!r}")
    target.write_text(text.replace(old, new, 1), encoding="utf-8", newline="\n")


replace_once(
    "src/features/plan/PlanView.tsx",
    '''const recoveryLabels: Record<ActionKind, string> = {
  user_temp: "Undo Vault",
  crash_dumps: "Undo Vault",
  huggingface_prune: "Redownloadable",
  npm_cache: "Redownloadable",
  docker_prune: "Irreversible"
};''',
    '''const recoveryLabels: Record<ActionKind, string> = {
  user_temp: "Undo Vault",
  system_temp: "Irreversible",
  prefetch: "Rebuildable",
  recycle_bin: "Irreversible",
  crash_dumps: "Undo Vault",
  huggingface_prune: "Redownloadable",
  npm_cache: "Redownloadable",
  docker_prune: "Irreversible"
};''',
)

replace_once(
    "src-tauri/src/planner/mod.rs",
    '''            ActionKind::HuggingfacePrune | ActionKind::NpmCache => {
                simulation.redownloadable_bytes = simulation
                    .redownloadable_bytes
                    .saturating_add(item.estimated_bytes);
                simulation.estimated_recovery_minutes =
                    simulation.estimated_recovery_minutes.saturating_add(8);
            }
            ActionKind::DockerPrune => {''',
    '''            ActionKind::Prefetch => {
                simulation.rebuildable_bytes = simulation
                    .rebuildable_bytes
                    .saturating_add(item.estimated_bytes);
                simulation.estimated_recovery_minutes =
                    simulation.estimated_recovery_minutes.saturating_add(2);
            }
            ActionKind::HuggingfacePrune | ActionKind::NpmCache => {
                simulation.redownloadable_bytes = simulation
                    .redownloadable_bytes
                    .saturating_add(item.estimated_bytes);
                simulation.estimated_recovery_minutes =
                    simulation.estimated_recovery_minutes.saturating_add(8);
            }
            ActionKind::SystemTemp | ActionKind::RecycleBin | ActionKind::DockerPrune => {''',
)

replace_once(
    "src-tauri/src/scanner/profile.rs",
    '''        findings.extend(system_caches.findings);
        completed_targets += 1;
    }
''',
    '''        findings.extend(system_caches.findings);
    }
''',
)

replace_once(
    "src-tauri/src/platform/windows/system_storage.rs",
    '''        let trimmed = text.trim_end_matches(|character| character == '\\' || character == '/');''',
    '''        let trimmed = text.trim_end_matches(['\\', '/']);''',
)

print("Classic cleanup enum, progress and Clippy integration fixed")
