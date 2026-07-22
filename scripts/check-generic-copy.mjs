import { existsSync, readFileSync } from "node:fs";

const violations = [];
const read = (path) => readFileSync(path, "utf8");
const requireText = (path, values, label) => {
  const source = read(path);
  for (const value of values) {
    if (!source.includes(value)) violations.push(`${path}: missing ${label} "${value}"`);
  }
  return source;
};

const genericUiFiles = [
  "src/App.tsx",
  "src/components/Sidebar.tsx",
  "src/features/assistant/StorageAssistantPanel.tsx",
  "src/features/findings/FindingsView.tsx",
  "src/features/findings/FindingRow.tsx",
  "src/features/findings/IntentPlanner.tsx",
  "src/features/plan/PlanView.tsx",
  "src/features/scan/ScanView.tsx",
  "src/features/receipt/ReceiptView.tsx",
  "src/features/settings/SettingsView.tsx",
  "src/features/timeline/TimelineView.tsx",
  "src/features/vault/VaultView.tsx",
  "src/lib/storageGroups.ts"
];

const forbiddenMachineExamples = [
  /\bAmaan\b/i,
  /\bOllama\b/i,
  /\bHugging\s*Face\b/i,
  /\bAndroid emulator/i,
  /\blocal AI model/i,
  /\bDocker volume/i,
  /\baish(?:-model|-ours)?\b/i,
  /\bken(?:-probe|-research)?\b/i,
  /\blive-runtime\b/i
];
const forbiddenRuntimeClaims = [
  /confidenceScore/,
  /estimatedRecoveryMinutes/,
  /protectedSummary/,
  /totalGrowthBytes\s*\?\?\s*0/,
  /for seven days/i,
  /kept for 7 days/i
];

for (const file of genericUiFiles) {
  const source = read(file);
  for (const pattern of forbiddenMachineExamples) {
    const match = source.match(pattern);
    if (match) violations.push(`${file}: machine-specific example "${match[0]}"`);
  }
  for (const pattern of forbiddenRuntimeClaims) {
    const match = source.match(pattern);
    if (match) violations.push(`${file}: static runtime claim "${match[0]}"`);
  }
}

const frontendTypes = read("src/types.ts");
for (const field of ["confidenceScore", "estimatedRecoveryMinutes", "protectedSummary", "reclaimableGrowthBytes"]) {
  if (frontendTypes.includes(field)) violations.push(`src/types.ts: obsolete field "${field}"`);
}
for (const required of ["\"prefetch\"", "\"generic_directory\"", "DriveInfo", "DriveKind", "roots: string[]", "drives: DriveInfo[]"]) {
  if (!frontendTypes.includes(required)) violations.push(`src/types.ts: missing contract "${required}"`);
}

const profileBackend = requireText("src-tauri/src/scanner/profile.rs", [
  "ActionKind::Prefetch",
  "ActionKind::GenericDirectory",
  "project_output_descriptor",
  "resolve_roots",
  "selected_drive_info",
  "aggregate_disk",
  "apply_drive_safety"
], "scan safeguard");
if (profileBackend.includes("eligible_temp_size") || profileBackend.includes("temp_minimum_age")) {
  violations.push("src-tauri/src/scanner/profile.rs: Temp is still age-filtered");
}

requireText("src-tauri/src/platform/windows/filesystem.rs", [
  "list_drives",
  "GetLogicalDrives",
  "GetVolumeInformationW",
  "volume_id",
  "same_drive"
], "volume discovery");

const filesystemBackend = requireText("src-tauri/src/actions/filesystem.rs", [
  "clean_user_temp",
  "clean_prefetch",
  "clean_generic_directory",
  "is_verified_project_output",
  "is_safe_dynamic_cache"
], "cleanup validation");
if (filesystemBackend.includes("temp_minimum_age")) {
  violations.push("src-tauri/src/actions/filesystem.rs: Temp cleanup is still age-filtered");
}

requireText("src-tauri/src/scanner/discovery.rs", [
  "dynamic.portable_cache",
  "is_portable_cache_name",
  "ActionKind::GenericDirectory"
], "portable cache safeguard");
requireText("src-tauri/src/insights/mod.rs", [
  "scope_fingerprint",
  "compared_with_at",
  "total_growth_bytes: None"
], "history safeguard");
requireText("src-tauri/src/policy.rs", [
  "volume_id",
  "file_system",
  "total_bytes",
  "normalized_roots.sort",
  "volumes.sort"
], "scope identity safeguard");

const appDataBackend = requireText("src-tauri/src/app_data.rs", [
  "DATA_GENERATION",
  "clear_scan_history",
  "clear_cleanup_records",
  "include_restore_files",
  "remove_retired_local_assistant",
  "storage-assistant"
], "migration/reset safeguard");
if (!appDataBackend.includes("if !request.include_restore_files")) {
  violations.push("src-tauri/src/app_data.rs: reset no longer preserves Restore files by default");
}
if (appDataBackend.includes('name == "models"')) {
  violations.push("src-tauri/src/app_data.rs: retired local model directory is still preserved");
}

requireText("src-tauri/src/commands/mod.rs", [
  "get_app_data_summary",
  "clear_scan_history",
  "clear_cleanup_records",
  "reset_app_data",
  "list_storage_drives",
  "combined_free_bytes"
], "command");
requireText("src/features/scan/ScanView.tsx", [
  "listStorageDrives",
  "selectedRoots",
  "DrivePicker",
  "Inspection only"
], "drive-selection behavior");
requireText("src/features/findings/FindingsView.tsx", [
  "StorageOverview",
  "groupInspectionFindings",
  "driveFilter",
  "storageGroups",
  "StorageAssistantPanel"
], "report behavior");

for (const retired of [
  "src-tauri/src/assistant/download.rs",
  "src-tauri/src/assistant/inference.rs",
  "src-tauri/src/assistant/prompt.rs",
  "src-tauri/src/cloud.rs",
  "src-tauri/src/intent/openai.rs",
  "src-tauri/src/intent/openrouter.rs",
  "src/features/assistant/StorageAssistantSettings.tsx",
  "landing-page/api/assistant.js",
  "scripts/test-assistant-proxy.mjs"
]) {
  if (existsSync(retired)) violations.push(`${retired}: retired model/provider implementation still exists`);
}

requireText("src-tauri/src/assistant/mod.rs", [
  "WinReclaim deterministic analysis",
  "storage-rules-v1",
  "available: true",
  "No network request",
  "aggregate_categories",
  "advisory_only: true"
], "deterministic storage brief safeguard");
requireText("src-tauri/src/intent/rules.rs", [
  "WinReclaim intent rules v1",
  "parse_target_bytes",
  "accepts_rebuildable",
  "explicitly_accepts_review_first",
  "category_is_excluded",
  "No prompt"
], "deterministic intent safeguard");
requireText("src-tauri/src/intent/selector.rs", [
  "remote_used: false",
  "validate_risk_classes",
  "validate_exclusions"
], "local intent selector safeguard");

const cargoManifest = read("src-tauri/Cargo.toml");
for (const retiredDependency of ["llama-cpp-2", "reqwest =", 'zip = { version = "2"']) {
  if (cargoManifest.includes(retiredDependency)) {
    violations.push(`src-tauri/Cargo.toml: retired runtime/network dependency remains "${retiredDependency}"`);
  }
}

const appLib = requireText("src-tauri/src/lib.rs", [
  "get_storage_assistant_status",
  "analyze_storage_report"
], "assistant command");
for (const retiredCommand of ["install_storage_assistant", "uninstall_storage_assistant"]) {
  if (appLib.includes(retiredCommand)) violations.push(`src-tauri/src/lib.rs: retired command remains "${retiredCommand}"`);
}
if (appLib.includes("mod cloud")) {
  violations.push("src-tauri/src/lib.rs: retired cloud module is still registered");
}

if (violations.length) {
  console.error("WinReclaim product-integrity checks failed.\n");
  for (const violation of violations) console.error(`- ${violation}`);
  process.exit(1);
}

console.log("WinReclaim product-integrity checks passed.");
