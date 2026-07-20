import { readFileSync } from "node:fs";

const genericUiFiles = [
  "src/components/Sidebar.tsx",
  "src/features/findings/FindingsView.tsx",
  "src/features/findings/FindingRow.tsx",
  "src/features/findings/IntentPlanner.tsx",
  "src/features/plan/PlanView.tsx",
  "src/features/scan/ScanView.tsx",
  "src/features/receipt/ReceiptView.tsx",
  "src/features/vault/VaultView.tsx"
];

const forbiddenExamples = [
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

const violations = [];

for (const file of genericUiFiles) {
  const source = readFileSync(file, "utf8");
  for (const pattern of forbiddenExamples) {
    const match = source.match(pattern);
    if (match) violations.push(`${file}: machine-specific example "${match[0]}"`);
  }
}

if (violations.length) {
  console.error("Generic WinReclaim screens must not advertise one machine's software or folders.");
  console.error("App and tool names belong in detection results and rule data only.\n");
  for (const violation of violations) console.error(`- ${violation}`);
  process.exit(1);
}

console.log("Generic UI copy check passed.");
