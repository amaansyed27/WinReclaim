import { useEffect, useMemo, useState } from "react";
import { Sidebar } from "./components/Sidebar";
import { WindowTitlebar } from "./components/WindowTitlebar";
import { FindingsView } from "./features/findings/FindingsView";
import { PlanView } from "./features/plan/PlanView";
import { ReceiptView } from "./features/receipt/ReceiptView";
import { ScanView } from "./features/scan/ScanView";
import { TimelineView } from "./features/timeline/TimelineView";
import { VaultView } from "./features/vault/VaultView";
import {
  cancelScan,
  createCleanupPlan,
  executeCleanupPlan,
  getAiStatus,
  getReclaimPassports,
  getStorageTimeline,
  interpretCleanupIntent,
  listVaultEntries,
  onScanProgress,
  restoreVaultEntry,
  startScan
} from "./lib/tauri";
import type {
  AiStatus,
  CleanupPlan,
  CleanupReceipt,
  ReclaimPassport,
  RestoreResult,
  ScanOptions,
  ScanProgress,
  ScanReport,
  StorageTimeline,
  VaultEntry
} from "./types";

export type AppStep = "scan" | "timeline" | "findings" | "plan" | "receipt" | "vault";

const pageTitles: Record<AppStep, string> = {
  scan: "Storage scan",
  timeline: "Storage Time Machine",
  findings: "Reclaim Passports",
  plan: "Reclaim Simulation",
  receipt: "Cleanup receipt",
  vault: "Undo Vault"
};

export function App() {
  const [step, setStep] = useState<AppStep>("scan");
  const [scanning, setScanning] = useState(false);
  const [executing, setExecuting] = useState(false);
  const [intentLoading, setIntentLoading] = useState(false);
  const [timelineLoading, setTimelineLoading] = useState(false);
  const [vaultLoading, setVaultLoading] = useState(false);
  const [restoringId, setRestoringId] = useState<string | null>(null);
  const [progress, setProgress] = useState<ScanProgress | null>(null);
  const [report, setReport] = useState<ScanReport | null>(null);
  const [plan, setPlan] = useState<CleanupPlan | null>(null);
  const [receipt, setReceipt] = useState<CleanupReceipt | null>(null);
  const [aiStatus, setAiStatus] = useState<AiStatus | null>(null);
  const [intentSummary, setIntentSummary] = useState<string | null>(null);
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [passports, setPassports] = useState<Map<string, ReclaimPassport>>(new Map());
  const [timeline, setTimeline] = useState<StorageTimeline | null>(null);
  const [vaultEntries, setVaultEntries] = useState<VaultEntry[]>([]);
  const [lastRestore, setLastRestore] = useState<RestoreResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let dispose: (() => void) | undefined;
    onScanProgress(setProgress).then((unlisten) => {
      dispose = unlisten;
    });
    return () => dispose?.();
  }, []);

  useEffect(() => {
    getAiStatus()
      .then(setAiStatus)
      .catch(() => {
        setAiStatus({
          configured: false,
          model: "gpt-5.6",
          privacyNote: "Only anonymized category, size, risk and consequence metadata is sent. Paths remain local."
        });
      });
    void refreshTimeline();
    void refreshVault();
  }, []);

  const actionFindingIds = useMemo(
    () =>
      new Set(
        report?.findings
          .filter((finding) => finding.actionAvailable)
          .map((finding) => finding.id) ?? []
      ),
    [report]
  );

  const availableSteps = useMemo(() => {
    const steps = new Set<AppStep>(["scan", "timeline", "vault"]);
    if (report) steps.add("findings");
    if (plan) steps.add("plan");
    if (receipt) steps.add("receipt");
    return steps;
  }, [plan, receipt, report]);

  function navigate(next: AppStep) {
    if (!availableSteps.has(next)) return;
    setStep(next);
    if (next === "timeline") void refreshTimeline();
    if (next === "vault") void refreshVault();
  }

  async function refreshTimeline() {
    setTimelineLoading(true);
    try {
      setTimeline(await getStorageTimeline());
    } catch (timelineError) {
      setError(String(timelineError));
    } finally {
      setTimelineLoading(false);
    }
  }

  async function refreshVault() {
    setVaultLoading(true);
    try {
      setVaultEntries(await listVaultEntries());
    } catch (vaultError) {
      setError(String(vaultError));
    } finally {
      setVaultLoading(false);
    }
  }

  async function handleStartScan(options: ScanOptions) {
    setError(null);
    setScanning(true);
    setProgress(null);
    setPlan(null);
    setReceipt(null);
    setIntentSummary(null);
    setSelectedIds(new Set());
    setPassports(new Map());
    setStep("scan");

    try {
      const nextReport = await startScan(options);
      setReport(nextReport);
      try {
        const nextPassports = await getReclaimPassports(nextReport.scanId);
        setPassports(new Map(nextPassports.map((passport) => [passport.findingId, passport])));
      } catch (passportError) {
        setError(`Scan completed, but Reclaim Passports failed: ${passportError}`);
      }
      await refreshTimeline();
    } catch (scanError) {
      setError(String(scanError));
    } finally {
      setScanning(false);
    }
  }

  async function handleCancelScan() {
    await cancelScan();
  }

  function toggleFinding(id: string) {
    if (!actionFindingIds.has(id)) return;
    setSelectedIds((current) => {
      const next = new Set(current);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }

  async function handleInterpretIntent(prompt: string) {
    if (!report) return;
    setError(null);
    setIntentLoading(true);

    try {
      const suggestion = await interpretCleanupIntent(report.scanId, prompt);
      const validSelection = suggestion.selectedFindingIds.filter((id) =>
        actionFindingIds.has(id)
      );
      setSelectedIds(new Set(validSelection));

      const exclusionNote = suggestion.excludedLabels.length
        ? ` Excluded: ${suggestion.excludedLabels.join(", ")}.`
        : "";
      setIntentSummary(`${suggestion.summary}${exclusionNote}`);
    } catch (intentError) {
      setIntentSummary(null);
      setError(String(intentError));
    } finally {
      setIntentLoading(false);
    }
  }

  async function handleCreatePlan() {
    if (!report || !selectedIds.size) return;
    setError(null);
    try {
      const nextPlan = await createCleanupPlan(report.scanId, [...selectedIds]);
      setPlan(nextPlan);
      setStep("plan");
    } catch (planError) {
      setError(String(planError));
    }
  }

  async function handleExecute() {
    if (!plan) return;
    setError(null);
    setExecuting(true);
    try {
      const nextReceipt = await executeCleanupPlan(plan.id, plan.planHash);
      setReceipt(nextReceipt);
      setStep("receipt");
      await refreshVault();
    } catch (executionError) {
      setError(String(executionError));
    } finally {
      setExecuting(false);
    }
  }

  async function handleRestore(id: string) {
    setError(null);
    setLastRestore(null);
    setRestoringId(id);
    try {
      setLastRestore(await restoreVaultEntry(id));
      await refreshVault();
    } catch (restoreError) {
      setError(String(restoreError));
    } finally {
      setRestoringId(null);
    }
  }

  function resetWorkflow() {
    setReport(null);
    setPlan(null);
    setReceipt(null);
    setIntentSummary(null);
    setSelectedIds(new Set());
    setPassports(new Map());
    setStep("scan");
  }

  return (
    <div className="desktop-app">
      <WindowTitlebar pageTitle={pageTitles[step]} />
      <div className="desktop-workspace">
        <Sidebar
          current={step}
          available={availableSteps}
          scanning={scanning}
          onNavigate={navigate}
        />

        <main className="workspace-main">
          <div className="workspace-scroll">
            {step === "scan" && (
              <ScanView
                scanning={scanning}
                progress={progress}
                report={report}
                error={error}
                onStart={handleStartScan}
                onCancel={handleCancelScan}
                onContinue={() => setStep("findings")}
              />
            )}

            {step === "timeline" && (
              <TimelineView
                timeline={timeline}
                loading={timelineLoading}
                onRefresh={refreshTimeline}
                onScan={() => setStep("scan")}
              />
            )}

            {step === "findings" && report && (
              <FindingsView
                report={report}
                passports={passports}
                selectedIds={selectedIds}
                aiStatus={aiStatus}
                intentLoading={intentLoading}
                intentSummary={intentSummary}
                onInterpretIntent={handleInterpretIntent}
                onToggle={toggleFinding}
                onBack={() => setStep("scan")}
                onCreatePlan={handleCreatePlan}
              />
            )}

            {step === "plan" && plan && (
              <PlanView
                plan={plan}
                executing={executing}
                error={error}
                onBack={() => setStep("findings")}
                onExecute={handleExecute}
              />
            )}

            {step === "receipt" && receipt && (
              <ReceiptView
                receipt={receipt}
                onOpenVault={() => setStep("vault")}
                onNewScan={resetWorkflow}
              />
            )}

            {step === "vault" && (
              <VaultView
                entries={vaultEntries}
                loading={vaultLoading}
                restoringId={restoringId}
                lastRestore={lastRestore}
                error={error}
                onRefresh={refreshVault}
                onRestore={handleRestore}
              />
            )}
          </div>
        </main>
      </div>
    </div>
  );
}
