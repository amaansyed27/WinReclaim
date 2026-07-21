import { useEffect, useMemo, useState } from "react";
import { ScanIcon } from "../../components/Icons";
import { formatBytes } from "../../lib/format";
import type { ScanReport } from "../../types";
import { analyzeStorageReport, getStorageAssistantStatus } from "./assistantApi";
import type { StorageAssistantReport, StorageAssistantStatus } from "./assistantTypes";
import "./StorageAssistant.css";

interface StorageAssistantPanelProps {
  report: ScanReport;
}

export function StorageAssistantPanel({ report }: StorageAssistantPanelProps) {
  const [status, setStatus] = useState<StorageAssistantStatus | null>(null);
  const [assistantReport, setAssistantReport] = useState<StorageAssistantReport | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setAssistantReport(null);
    setError(null);
    getStorageAssistantStatus().then(setStatus).catch((statusError) => setError(String(statusError)));
  }, [report.scanId]);

  const findingById = useMemo(
    () => new Map(report.findings.map((finding) => [finding.id, finding])),
    [report.findings]
  );

  async function analyze() {
    setLoading(true);
    setError(null);
    try {
      setAssistantReport(await analyzeStorageReport(report.scanId));
      setStatus(await getStorageAssistantStatus());
    } catch (analysisError) {
      setError(String(analysisError));
    } finally {
      setLoading(false);
    }
  }

  const installed = Boolean(status?.installed && status.verified);

  return (
    <section className="surface assistant-report-panel" aria-busy={loading}>
      <div className="assistant-report-head">
        <div className="assistant-title-group">
          <span className="assistant-panel-icon" aria-hidden="true"><ScanIcon /></span>
          <div>
            <span className="surface-label">Local insight</span>
            <h2>Storage Assistant</h2>
            <p>Turns this scan into a short, structured explanation without changing any cleanup decision.</p>
          </div>
        </div>
        <span className={`assistant-status-pill ${installed ? "is-ready" : ""}`}>
          <i aria-hidden="true" />
          {installed ? "Ready" : "Needs setup"}
        </span>
      </div>

      {!installed && (
        <div className="assistant-empty-state">
          <div>
            <strong>Optional local model is not installed</strong>
            <span>Install it from Settings once. The model and runtime stay on this PC.</span>
          </div>
          <span className="assistant-size-note">About 1.4 GB</span>
        </div>
      )}

      {installed && !assistantReport && !loading && (
        <div className="assistant-run-row">
          <div className="assistant-model-summary">
            <strong>{status?.model}</strong>
            <span>Processes scan metadata only · no file contents · no cloud request</span>
          </div>
          <button className="button button-primary assistant-run-button" disabled={loading} onClick={() => void analyze()}>
            Generate summary
          </button>
        </div>
      )}

      {loading && (
        <div className="assistant-loading-state" role="status" aria-live="polite">
          <span className="assistant-spinner" aria-hidden="true" />
          <div>
            <strong>Analyzing this scan locally</strong>
            <span>Building a constrained JSON report with Qwen3.5-2B. This can take a moment on CPU.</span>
          </div>
        </div>
      )}

      {assistantReport && !loading && (
        <div className="assistant-report-content">
          <div className="assistant-summary-block">
            <span>Summary</span>
            <p>{assistantReport.summary}</p>
          </div>

          {assistantReport.observations.length > 0 && (
            <div className="assistant-observations">
              <span>What stands out</span>
              <ul>
                {assistantReport.observations.map((observation) => (
                  <li key={observation}>{observation}</li>
                ))}
              </ul>
            </div>
          )}

          {assistantReport.annotations.length > 0 && (
            <div className="assistant-annotations">
              <div>
                <span>Clarified folders</span>
                <small>Suggested labels only. Cleanup classification is unchanged.</small>
              </div>
              {assistantReport.annotations.map((annotation) => {
                const finding = findingById.get(annotation.findingId);
                if (!finding) return null;
                return (
                  <article key={annotation.findingId}>
                    <div>
                      <strong>{annotation.suggestedName}</strong>
                      <span>{annotation.group} · {Math.round(annotation.confidence * 100)}% confidence</span>
                    </div>
                    <strong>{formatBytes(finding.estimatedBytes)}</strong>
                    <p>{annotation.explanation}</p>
                    <code>{finding.path}</code>
                  </article>
                );
              })}
            </div>
          )}

          <div className="assistant-report-footer">
            <span>{assistantReport.model}</span>
            <button className="button button-secondary" disabled={loading} onClick={() => void analyze()}>
              Regenerate
            </button>
          </div>
        </div>
      )}

      {error && !loading && (
        <div className="assistant-error" role="alert">
          <div>
            <strong>Summary generation failed</strong>
            <span>The scan is unaffected. Retry once; if it persists, reinstall the assistant from Settings.</span>
          </div>
          <button className="button button-secondary" onClick={() => void analyze()} disabled={!installed}>Retry</button>
          <details>
            <summary>Technical details</summary>
            <code>{error}</code>
          </details>
        </div>
      )}

      <p className="assistant-advisory-note">
        Advisory only. The assistant cannot enable cleanup, select findings, alter safety labels or delete files.
      </p>
    </section>
  );
}
