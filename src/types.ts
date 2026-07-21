export type RiskClass =
  | "safe_now"
  | "rebuild_or_redownload"
  | "review_first"
  | "protected";

export type ActionKind =
  | "user_temp"
  | "system_temp"
  | "prefetch"
  | "generic_directory"
  | "recycle_bin"
  | "crash_dumps"
  | "huggingface_prune"
  | "npm_cache"
  | "docker_prune";

export type RecoveryClass =
  | "reversible"
  | "redownloadable"
  | "rebuildable"
  | "irreversible"
  | "protected";

export type DriveKind = "fixed" | "removable" | "network" | "optical" | "ram_disk" | "other";
export type ScanMode = "quick" | "balanced" | "deep";

export interface DriveInfo {
  root: string;
  label: string;
  fileSystem: string;
  volumeId: string;
  totalBytes: number;
  freeBytes: number;
  usedBytes: number;
  isSystem: boolean;
  kind: DriveKind;
}

export interface ScanOptions {
  roots: string[];
  mode: ScanMode;
  includeKnownTargets: boolean;
  includeProjectOutputs: boolean;
  discoverUnknown: boolean;
  includeAppData: boolean;
  includeSystemDriveCaches: boolean;
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
  drives: DriveInfo[];
  scopeFingerprint: string;
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

export interface PlanSimulation {
  currentFreeBytes: number;
  projectedFreeBytes: number;
  estimatedReclaimBytes: number;
  reversibleBytes: number;
  redownloadableBytes: number;
  rebuildableBytes: number;
  irreversibleBytes: number;
  affectedItems: number;
}

export interface CleanupPlan {
  id: string;
  scanId: string;
  createdAt: string;
  estimatedReclaimBytes: number;
  items: CleanupPlanItem[];
  simulation: PlanSimulation;
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
  recoveryClass: RecoveryClass;
  vaultEntryIds: string[];
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
  vaultEntryIds: string[];
  ruleSetVersion: string;
}

export interface ReclaimPassport {
  findingId: string;
  lastChangedAt?: string | null;
  recoveryClass: RecoveryClass;
  recoveryMethod: string;
  activityNote: string;
}

export interface SnapshotSummary {
  id: string;
  scanId: string;
  capturedAt: string;
  usedBytes: number;
  freeBytes: number;
}

export interface TimelineDelta {
  key: string;
  displayName: string;
  category: string;
  path: string;
  previousBytes: number;
  currentBytes: number;
  deltaBytes: number;
  actionAvailable: boolean;
}

export interface StorageTimeline {
  snapshots: SnapshotSummary[];
  deltas: TimelineDelta[];
  totalGrowthBytes?: number | null;
  comparedWithAt?: string | null;
  baselineAvailable: boolean;
}

export type VaultStatus = "active" | "restored" | "partially_restored" | "expired";

export interface VaultEntry {
  id: string;
  receiptId: string;
  findingId: string;
  displayName: string;
  originalRoot: string;
  payloadRoot: string;
  relativePaths: string[];
  storedBytes: number;
  createdAt: string;
  expiresAt: string;
  restoredAt?: string | null;
  status: VaultStatus;
}

export interface RestoreResult {
  vaultEntryId: string;
  restoredEntries: number;
  skippedEntries: number;
  restoredBytes: number;
  status: VaultStatus;
  message: string;
}

export interface AppDataSummary {
  root: string;
  snapshotCount: number;
  receiptCount: number;
  vaultEntryCount: number;
  vaultBytes: number;
}

export interface AppDataMutation {
  removedEntries: number;
  removedBytes: number;
  includedRestoreFiles: boolean;
}
