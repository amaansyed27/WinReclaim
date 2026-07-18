import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AiStatus,
  CleanupPlan,
  CleanupReceipt,
  IntentSuggestion,
  ScanProgress,
  ScanReport
} from "../types";

export async function startScan(): Promise<ScanReport> {
  return invoke<ScanReport>("start_scan", { request: {} });
}

export async function cancelScan(): Promise<void> {
  return invoke("cancel_scan");
}

export async function getAiStatus(): Promise<AiStatus> {
  return invoke<AiStatus>("get_ai_status");
}

export async function interpretCleanupIntent(
  scanId: string,
  prompt: string
): Promise<IntentSuggestion> {
  return invoke<IntentSuggestion>("interpret_cleanup_intent", {
    request: { scanId, prompt }
  });
}

export async function createCleanupPlan(
  scanId: string,
  findingIds: string[]
): Promise<CleanupPlan> {
  return invoke<CleanupPlan>("create_cleanup_plan", {
    request: { scanId, findingIds }
  });
}

export async function executeCleanupPlan(
  planId: string,
  planHash: string
): Promise<CleanupReceipt> {
  return invoke<CleanupReceipt>("execute_cleanup_plan", {
    request: { planId, planHash }
  });
}

export async function listReceipts(): Promise<CleanupReceipt[]> {
  return invoke<CleanupReceipt[]>("list_receipts");
}

export async function onScanProgress(
  handler: (progress: ScanProgress) => void
): Promise<UnlistenFn> {
  return listen<ScanProgress>("scan-progress", (event) => handler(event.payload));
}
