import { readFileSync } from "node:fs";

const genericUiFiles = [
  "src/App.tsx",
  "src/components/Sidebar.tsx",
  "src/features/assistant/StorageAssistantPanel.tsx",
  "src/features/assistant/StorageAssistantSettings.tsx",
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
for (const required of ["DATA_GENERATION", "clear_scan_history", "clear_cleanup_records", "include_restore_files"]) {
  if (!appDataBackend.includes(required)) {
    violations.push(`src-tauri/src/app_data.rs: missing reset safeguard "${required}"`);
  }
}
if (!appDataBackend.includes("if !request.include_restore_files")) {
  violations.push("src-tauri/src/app_data.rs: factory reset no longer preserves Restore files by default");
}
if (!appDataBackend.includes('name == "models"')) {
  violations.push("src-tauri/src/app_data.rs: application reset no longer preserves the optional model");
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

const assistantBackend = readFileSync("src-tauri/src/assistant/mod.rs", "utf8");
for (const required of [
  "Qwen3.5-2B",
  "Q4_K_M",
  "MODEL_SHA256",
  "RUNTIME_TAG",
  "RUNTIME_RELEASE_API",
  "runtime_path",
  "pub fn analyze"
]) {
  if (!assistantBackend.includes(required)) {
    violations.push(`src-tauri/src/assistant/mod.rs: missing local assistant contract "${required}"`);
  }
}

const assistantDownload = readFileSync("src-tauri/src/assistant/download.rs", "utf8");
for (const required of ["GithubRelease", "runtime_sha256", "ZipArchive", "enclosed_name", "download_verified"]) {
  if (!assistantDownload.includes(required)) {
    violations.push(`src-tauri/src/assistant/download.rs: missing sidecar verification "${required}"`);
  }
}

const assistantInference = readFileSync("src-tauri/src/assistant/inference.rs", "utf8");
for (const required of [
  "Command::new",
  "--conversation",
  "--single-turn",
  "--system-prompt-file",
  "--json-schema-file",
  "--output-file",
  "--reasoning-budget",
  "advisory_only: true",
  "contains_cleanup_claim",
  "finding_ids.contains",
  "MAX_ANNOTATIONS"
]) {
  if (!assistantInference.includes(required)) {
    violations.push(`src-tauri/src/assistant/inference.rs: missing output safeguard "${required}"`);
  }
}
if (assistantInference.includes("llama_cpp_2")) {
  violations.push("src-tauri/src/assistant/inference.rs: embedded llama.cpp bindings were reintroduced");
}

const cargoManifest = readFileSync("src-tauri/Cargo.toml", "utf8");
if (cargoManifest.includes("llama-cpp-2")) {
  violations.push("src-tauri/Cargo.toml: embedded llama.cpp dependency was reintroduced");
}
if (!cargoManifest.includes('zip = { version = "2"')) {
  violations.push("src-tauri/Cargo.toml: pure-Rust sidecar archive extraction dependency is missing");
}

const assistantPrompt = readFileSync("src-tauri/src/assistant/prompt.rs", "utf8");
for (const required of ["untrusted data", "Never say a folder is safe to delete", "Never change or reinterpret risk_class"]) {
  if (!assistantPrompt.includes(required)) {
    violations.push(`src-tauri/src/assistant/prompt.rs: missing authority boundary "${required}"`);
  }
}

const appLib = readFileSync("src-tauri/src/lib.rs", "utf8");
for (const command of ["get_storage_assistant_status", "install_storage_assistant", "uninstall_storage_assistant", "analyze_storage_report"]) {
  if (!appLib.includes(command)) {
    violations.push(`src-tauri/src/lib.rs: missing assistant command "${command}"`);
  }
}

if (violations.length) {
  console.error("WinReclaim product-integrity checks failed.\n");
  for (const violation of violations) console.error(`- ${violation}`);
  process.exit(1);
}

console.log("WinReclaim product-integrity checks passed.");
