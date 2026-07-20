import type { ActionKind, RecoveryClass, RiskClass } from "../types";

export const riskCopy: Record<RiskClass, { title: string; note: string }> = {
  safe_now: {
    title: "Safe to remove",
    note: "Temporary or disposable files covered by a verified cleanup rule."
  },
  rebuild_or_redownload: {
    title: "Can be downloaded again",
    note: "Removing this frees space. The app or tool can download or recreate it later."
  },
  review_first: {
    title: "Check before removing",
    note: "This may contain useful app data, project files or settings. Read the details first."
  },
  protected: {
    title: "Not removable here",
    note: "WinReclaim shows the size, but automatic cleanup is disabled for this item."
  }
};

export const recoveryLabels: Record<RecoveryClass, string> = {
  reversible: "Can be restored",
  redownloadable: "Downloaded again later",
  rebuildable: "Recreated by the app",
  irreversible: "Cannot be undone",
  protected: "Not removable"
};

export const actionRecoveryLabels: Record<ActionKind, string> = {
  user_temp: "Can be restored for 7 days",
  system_temp: "Cannot be undone",
  prefetch: "Windows recreates it",
  recycle_bin: "Cannot be undone",
  crash_dumps: "Can be restored for 7 days",
  huggingface_prune: "Downloaded again when needed",
  npm_cache: "Downloaded again when needed",
  docker_prune: "Cannot be undone"
};

export function recoveryLabel(recoveryClass: RecoveryClass): string {
  return recoveryLabels[recoveryClass];
}
