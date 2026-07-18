import { useEffect, useMemo, useState } from "react";
import { Brand } from "./components/Brand";
import { StepNav } from "./components/StepNav";
import { FindingsView } from "./features/findings/FindingsView";
import { PlanView } from "./features/plan/PlanView";
import { ReceiptView } from "./features/receipt/ReceiptView";
import { ScanView } from "./features/scan/ScanView";
import { UpdateControl } from "./features/update/UpdateControl";
import {
  cancelScan,
  createCleanupPlan,
  executeCleanupPlan,
  getAiStatus,
  interpretCleanupIntent,
  onScanProgress,
  startScan
} from "./lib/tauri";
import type {
  AiStatus,
  CleanupPlan,
  CleanupReceipt,
  ScanProgress,
  ScanReport
} from "./types";

export type AppStep = "scan" | "findings" | "plan" | "receipt";

export function App() {
  const [step, setStep] = useState<AppStep>("scan");
  const [scanning, setScanning] = useState(false);
  const [executing, setExecuting] = useState(false);
  const [intentLoading, setIntentLoading] = useState(false);
  const [progress, setProgress] = useState<ScanProgress | null>(null);
  const [report, setReport] = useState<ScanReport | null>(null);
  const [plan, setPlan] = useState<CleanupPlan | null>(null);
  const [receipt, setReceipt] = useState<CleanupReceipt | null>(null);
  const [aiStatus, setAiStatus] = useState<AiStatus | null>(null);
  const [intentSummary, setIntentSummary] = useState<string | null>(null);
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
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
          privacyNote:
            "Only anonymized category, size, risk and consequence metadata is sent. Paths remain local."
        });
      });
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

  async function handleStartScan() {
    setError(null);
    setScanning(true);
    setProgress(null);
    setPlan(null);
    setReceipt(null);
    setIntentSummary(null);
    setSelectedIds(new Set());
    setStep("scan");

    try {
      const nextReport = await startScan();
      setReport(nextReport);
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
        ? ` Explicitly excluded: ${suggestion.excludedLabels.join(", ")}.`
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
    } catch (executionError) {
      setError(String(executionError));
    } finally {
      setExecuting(false);
    }
  }

  return (
    <div className="app-shell">
      <header className="topbar">
        <Brand />
        <StepNav current={step} />
        <UpdateControl />
      </header>

      <main>
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

        {step === "findings" && report && (
          <FindingsView
            report={report}
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
            onNewScan={() => {
              setReport(null);
              setPlan(null);
              setReceipt(null);
              setIntentSummary(null);
              setSelectedIds(new Set());
              setStep("scan");
            }}
          />
        )}
      </main>
    </div>
  );
}
