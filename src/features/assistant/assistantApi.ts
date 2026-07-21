import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AssistantDownloadProgress,
  StorageAssistantReport,
  StorageAssistantStatus
} from "./assistantTypes";

export async function getStorageAssistantStatus(): Promise<StorageAssistantStatus> {
  return invoke<StorageAssistantStatus>("get_storage_assistant_status");
}

export async function installStorageAssistant(): Promise<StorageAssistantStatus> {
  return invoke<StorageAssistantStatus>("install_storage_assistant");
}

export async function uninstallStorageAssistant(): Promise<StorageAssistantStatus> {
  return invoke<StorageAssistantStatus>("uninstall_storage_assistant");
}

export async function analyzeStorageReport(scanId: string): Promise<StorageAssistantReport> {
  return invoke<StorageAssistantReport>("analyze_storage_report", {
    request: { scanId }
  });
}

export async function onAssistantDownloadProgress(
  handler: (progress: AssistantDownloadProgress) => void
): Promise<UnlistenFn> {
  return listen<AssistantDownloadProgress>("assistant-download-progress", (event) =>
    handler(event.payload)
  );
}
