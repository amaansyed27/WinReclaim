import { existsSync, readFileSync } from "node:fs";

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

const violations = [];

for (const file of genericUiFiles) {
  const source = readFileSync(file, "utf8");
  for (const pattern of forbiddenMachineExamples) {
    const match = source.match(pattern);
    if (match) violations.push(`${file}: machine-specific example "${match[0]}"`);
  }
  for (const pattern of forbiddenRuntimeClaims) {
    const match = source.match(pattern);
    if (match) violations.push(`${file}: static runtime claim "${match[0]}"`);
  }
}

const frontendTypes = readFileSync("src/types.ts", "utf8");
for (const field of ["confidenceScore", "estimatedRecoveryMinutes", "protectedSummary", "reclaimableGrowthBytes"]) {
  if (frontendTypes.includes(field)) violations.push(`src/types.ts: obsolete field "${field}"`);
}
for (const action of ["prefetch", "generic_directory"]) {
  if (!frontendTypes.includes(`"${action}"`)) {
    violations.push(`src/types.ts: missing action kind "${action}"`);
  }
}
for (const required of ["DriveInfo", "DriveKind", "roots: string[]", "drives: DriveInfo[]"]) {
  if (!frontendTypes.includes(required)) {
    violations.push(`src/types.ts: missing multi-drive type "${required}"`);
  }
}

const profileBackend = readFileSync("src-tauri/src/scanner/profile.rs", "utf8");
if (profileBackend.includes("eligible_temp_size") || profileBackend.includes("temp_minimum_age")) {
  violations.push("src-tauri/src/scanner/profile.rs: Temp is still age-filtered instead of measuring the full root");
}
for (const required of [
  "ActionKind::Prefetch",
  "ActionKind::GenericDirectory",
  "project_output_descriptor",
  "resolve_roots",
  "selected_drive_info",
  "aggregate_disk",
  "apply_drive_safety"
]) {
  if (!profileBackend.includes(required)) {
    violations.push(`src-tauri/src/scanner/profile.rs: missing scan safeguard "${required}"`);
  }
}

const platformBackend = readFileSync("src-tauri/src/platform/windows/filesystem.rs", "utf8");
for (const required of ["list_drives", "GetLogicalDrives", "GetVolumeInformationW", "volume_id", "same_drive"]) {
  if (!platformBackend.includes(required)) {
    violations.push(`src-tauri/src/platform/windows/filesystem.rs: missing volume discovery "${required}"`);
  }
}

const filesystemBackend = readFileSync("src-tauri/src/actions/filesystem.rs", "utf8");
for (const required of ["clean_user_temp", "clean_prefetch", "clean_generic_directory", "is_verified_project_output", "is_safe_dynamic_cache"]) {
  if (!filesystemBackend.includes(required)) {
    violations.push(`src-tauri/src/actions/filesystem.rs: missing cleanup validation "${required}"`);
  }
}
if (filesystemBackend.includes("temp_minimum_age")) {
  violations.push("src-tauri/src/actions/filesystem.rs: Temp cleanup is still age-filtered");
}

const discoveryBackend = readFileSync("src-tauri/src/scanner/discovery.rs", "utf8");
for (const required of ["dynamic.portable_cache", "is_portable_cache_name", "ActionKind::GenericDirectory"]) {
  if (!discoveryBackend.includes(required)) {
    violations.push(`src-tauri/src/scanner/discovery.rs: missing portable cache behavior "${required}"`);
  }
}

const timelineBackend = readFileSync("src-tauri/src/insights/mod.rs", "utf8");
for (const required of ["scope_fingerprint", "compared_with_at", "total_growth_bytes: None"]) {
  if (!timelineBackend.includes(required)) {
    violations.push(`src-tauri/src/insights/mod.rs: missing history safeguard "${required}"`);
  }
}

const policyBackend = readFileSync("src-tauri/src/policy.rs", "utf8");
for (const required of ["volume_id", "file_system", "total_bytes", "normalized_roots.sort", "volumes.sort"]) {
  if (!policyBackend.includes(required)) {
    violations.push(`src-tauri/src/policy.rs: scan scope is not keyed by stable volume identity "${required}"`);
  }
}

const appDataBackend = readFileSync("src-tauri/src/app_data.rs", "utf8");
for (const required of [
  "DATA_GENERATION",
  "clear_scan_history",
  "clear_cleanup_records",
  "include_restore_files",
  "remove_retired_local_assistant",
  "storage-assistant"
]) {
  if (!appDataBackend.includes(required)) {
    violations.push(`src-tauri/src/app_data.rs: missing migration/reset safeguard "${required}"`);
  }
}
if (!appDataBackend.includes("if !request.include_restore_files")) {
  violations.push("src-tauri/src/app_data.rs: factory reset no longer preserves Restore files by default");
}
if (appDataBackend.includes('name == "models"')) {
  violations.push("src-tauri/src/app_data.rs: retired local model directory is still preserved by reset");
}

const commandBackend = readFileSync("src-tauri/src/commands/mod.rs", "utf8");
for (const required of [
  "get_app_data_summary",
  "clear_scan_history",
  "clear_cleanup_records",
  "reset_app_data",
  "list_storage_drives",
  "combined_free_bytes"
]) {
  if (!commandBackend.includes(required)) {
    violations.push(`src-tauri/src/commands/mod.rs: missing command "${required}"`);
  }
}

const scanView = readFileSync("src/features/scan/ScanView.tsx", "utf8");
for (const required of ["listStorageDrives", "selectedRoots", "DrivePicker", "Inspection only"]) {
  if (!scanView.includes(required)) {
    violations.push(`src/features/scan/ScanView.tsx: missing drive-selection behavior "${required}"`);
  }
}

const findingsView = readFileSync("src/features/findings/FindingsView.tsx", "utf8");
for (const required of ["StorageOverview", "groupInspectionFindings", "driveFilter", "storageGroups", "StorageAssistantPanel"]) {
  if (!findingsView.includes(required)) {
    violations.push(`src/features/findings/FindingsView.tsx: missing report behavior "${required}"`);
  }
}

for (const retired of [
  "src-tauri/src/assistant/download.rs",
  "src-tauri/src/assistant/inference.rs",
  "src-tauri/src/assistant/prompt.rs",
  "src-tauri/src/intent/openai.rs",
  "src/features/assistant/StorageAssistantSettings.tsx"
]) {
  if (existsSync(retired)) violations.push(`${retired}: retired local/direct-provider implementation still exists`);
}

const assistantBackend = readFileSync("src-tauri/src/assistant/mod.rs", "utf8");
for (const required of [
  "openrouter/free",
  "cloud::request",
  "StorageSummaryPayload",
  "contains_cleanup_claim",
  "advisory_only: true",
  "Paths, usernames, folder names, project names and file contents stay on this PC"
]) {
  if (!assistantBackend.includes(required)) {
    violations.push(`src-tauri/src/assistant/mod.rs: missing cloud assistant safeguard "${required}"`);
  }
}

const cloudBackend = readFileSync("src-tauri/src/cloud.rs", "utf8");
for (const required of [
  "https://winreclaim.vercel.app/api/assistant",
  "X-WinReclaim-Client",
  "WINRECLAIM_ASSISTANT_URL",
  "Duration::from_secs(60)"
]) {
  if (!cloudBackend.includes(required)) {
    violations.push(`src-tauri/src/cloud.rs: missing cloud transport contract "${required}"`);
  }
}

const intentBackend = readFileSync("src-tauri/src/intent/openrouter.rs", "utf8");
for (const required of [
  "openrouter/free",
  "cloud::request",
  "candidate_id",
  "risk_class",
  "paths, usernames, folder names, project names and file contents stay local"
]) {
  if (!intentBackend.includes(required)) {
    violations.push(`src-tauri/src/intent/openrouter.rs: missing intent privacy contract "${required}"`);
  }
}

const proxy = readFileSync("landing-page/api/assistant.js", "utf8");
for (const required of [
  "OPENROUTER_API_KEY",
  'const MODEL = "openrouter/free"',
  "response_format",
  "require_parameters: true",
  "RATE_LIMIT",
  "x-winreclaim-client",
  "Never claim anything is safe to delete or remove",
  "Paths"
]) {
  if (!proxy.includes(required)) {
    violations.push(`landing-page/api/assistant.js: missing proxy restriction "${required}"`);
  }
}
if (/sk-or-v1-[A-Za-z0-9_-]+/.test(proxy)) {
  violations.push("landing-page/api/assistant.js: OpenRouter key was committed to source");
}

const cargoManifest = readFileSync("src-tauri/Cargo.toml", "utf8");
if (cargoManifest.includes("llama-cpp-2") || cargoManifest.includes('zip = { version = "2"')) {
  violations.push("src-tauri/Cargo.toml: retired local-model runtime dependency remains");
}

const appLib = readFileSync("src-tauri/src/lib.rs", "utf8");
for (const command of ["get_storage_assistant_status", "analyze_storage_report"]) {
  if (!appLib.includes(command)) {
    violations.push(`src-tauri/src/lib.rs: missing assistant command "${command}"`);
  }
}
for (const retiredCommand of ["install_storage_assistant", "uninstall_storage_assistant"]) {
  if (appLib.includes(retiredCommand)) {
    violations.push(`src-tauri/src/lib.rs: retired command remains "${retiredCommand}"`);
  }
}

if (violations.length) {
  console.error("WinReclaim product-integrity checks failed.\n");
  for (const violation of violations) console.error(`- ${violation}`);
  process.exit(1);
}

console.log("WinReclaim product-integrity checks passed.");
