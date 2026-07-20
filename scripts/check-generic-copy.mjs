import { readFileSync } from "node:fs";

const genericUiFiles = [
  "src/App.tsx",
  "src/components/Sidebar.tsx",
  "src/features/findings/FindingsView.tsx",
  "src/features/findings/FindingRow.tsx",
  "src/features/findings/IntentPlanner.tsx",
  "src/features/plan/PlanView.tsx",
  "src/features/scan/ScanView.tsx",
  "src/features/receipt/ReceiptView.tsx",
  "src/features/timeline/TimelineView.tsx",
  "src/features/vault/VaultView.tsx"
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

const actionFiles = [
  "src-tauri/src/actions/mod.rs",
  "src-tauri/src/actions/filesystem.rs"
];
for (const file of actionFiles) {
  const source = readFileSync(file, "utf8");
  for (const pattern of [/ActionKind::Prefetch/, /clean_prefetch/]) {
    if (pattern.test(source)) violations.push(`${file}: executable Prefetch cleanup remains`);
  }
}

const timelineBackend = readFileSync("src-tauri/src/insights/mod.rs", "utf8");
for (const required of ["scope_fingerprint", "compared_with_at", "total_growth_bytes: None"]) {
  if (!timelineBackend.includes(required)) {
    violations.push(`src-tauri/src/insights/mod.rs: missing history safeguard "${required}"`);
  }
}

if (violations.length) {
  console.error("WinReclaim product-integrity checks failed.\n");
  for (const violation of violations) console.error(`- ${violation}`);
  process.exit(1);
}

console.log("WinReclaim product-integrity checks passed.");
