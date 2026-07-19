export type RiskClass =
  | "safe_now"
  | "rebuild_or_redownload"
  | "review_first"
  | "protected";

export type ActionKind =
  | "user_temp"
  | "crash_dumps"
  | "huggingface_prune"
  | "npm_cache"
  | "docker_prune";

export type ScanMode = "quick" | "balanced" | "deep";

export interface ScanOptions {
  mode: ScanMode;
  includeKnownTargets: boolean;
  includeProjectOutputs: boolean;
  discoverUnknown: boolean;
  includeAppData: boolean;
  minimumFindingBytes: number;
  maxUnknownFindings: number;
}

export interface DiskSnapshot {
  root: string;
  totalBytes: number;
  freeBytes: number;
  usedBytes: number;
}

export interface Finding {
  id: string;
  ruleId: string;
  displayName: string;
  category: string;
  path: string;
  estimatedBytes: number;
  riskClass: RiskClass;
  explanation: string;
  consequence: string;
  confidence: "high" | "medium" | "low";
  actionKind?: ActionKind | null;
  actionAvailable: boolean;
  selectedByDefault: boolean;
}

export interface ScanReport {
  scanId: string;
  startedAt: string;
  completedAt: string;
  root: string;
  disk: DiskSnapshot;
  findings: Finding[];
  scannedEntries: number;
  skippedEntries: number;
  errors: string[];
}

export interface ScanProgress {
  phase: string;
  currentPath?: string | null;
  completedTargets: number;
  totalTargets: number;
  discoveredBytes: number;
  scannedEntries: number;
}

export interface AiStatus {
  configured: boolean;
  model: string;
  privacyNote: string;
}

export interface IntentSuggestion {
  selectedFindingIds: string[];
  targetReclaimBytes?: number | null;
  estimatedReclaimBytes: number;
  allowedRiskClasses: RiskClass[];
  excludedLabels: string[];
  summary: string;
  model: string;
  remoteUsed: boolean;
}

export interface CleanupPlanItem {
  findingId: string;
  displayName: string;
  category: string;
  path: string;
  estimatedBytes: number;
  riskClass: RiskClass;
  consequence: string;
  actionKind: ActionKind;
}

export interface CleanupPlan {
  id: string;
  scanId: string;
  createdAt: string;
  estimatedReclaimBytes: number;
  items: CleanupPlanItem[];
  ruleSetVersion: string;
  planHash: string;
}

export interface ActionResult {
  findingId: string;
  displayName: string;
  estimatedBytes: number;
  measuredBytesBefore: number;
  measuredBytesAfter: number;
  deletedEntries: number;
  skippedEntries: number;
  success: boolean;
  message: string;
}

export interface CleanupReceipt {
  id: string;
  planId: string;
  planHash: string;
  startedAt: string;
  completedAt: string;
  diskFreeBefore: number;
  diskFreeAfter: number;
  actualReclaimedBytes: number;
  estimatedReclaimBytes: number;
  results: ActionResult[];
  protectedSummary: string[];
  ruleSetVersion: string;
}
