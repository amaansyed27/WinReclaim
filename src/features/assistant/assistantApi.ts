import { invoke } from "@tauri-apps/api/core";
import type { StorageAssistantReport, StorageAssistantStatus } from "./assistantTypes";

export async function getStorageAssistantStatus(): Promise<StorageAssistantStatus> {
  return invoke<StorageAssistantStatus>("get_storage_assistant_status");
}

export async function analyzeStorageReport(scanId: string): Promise<StorageAssistantReport> {
  return invoke<StorageAssistantReport>("analyze_storage_report", {
    request: { scanId }
  });
}
