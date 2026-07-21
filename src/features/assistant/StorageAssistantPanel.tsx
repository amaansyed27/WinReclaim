import { useEffect, useMemo, useState } from "react";
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
    <section className="surface assistant-report-panel">
      <div className="assistant-report-head">
        <div>
          <span className="surface-label">Optional local analysis</span>
          <h2>Storage Assistant</h2>
          <p>
            Produces an OpenCode-style summary from the completed scan. Measurements, protection and cleanup actions remain deterministic.
          </p>
        </div>
        <span className={`assistant-status-pill ${installed ? "is-ready" : ""}`}>
          {installed ? "Local model ready" : "Model not installed"}
        </span>
      </div>

      {!installed && (
        <div className="assistant-empty-state">
          <strong>Install the optional model from Settings</strong>
          <span>The download is about 1.4 GB and can be removed independently later.</span>
        </div>
      )}

      {installed && !assistantReport && (
        <div className="assistant-run-row">
          <div>
            <strong>{status?.model}</strong>
            <span>Only scan metadata is processed locally. File contents are never sent to the model.</span>
          </div>
          <button className="button button-secondary" disabled={loading} onClick={() => void analyze()}>
            {loading ? "Analyzing…" : "Generate storage summary"}
          </button>
        </div>
      )}

      {assistantReport && (
        <div className="assistant-report-content">
          <div className="assistant-summary-block">
            <span>Summary</span>
            <p>{assistantReport.summary}</p>
          </div>

          {assistantReport.observations.length > 0 && (
            <div className="assistant-observations">
              <span>Key observations</span>
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
                <span>Suggested labels for unclear folders</span>
                <small>Labels are inferred and do not change cleanup classification.</small>
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
              {loading ? "Analyzing…" : "Regenerate"}
            </button>
          </div>
        </div>
      )}

      <p className="assistant-advisory-note">
        Advisory only. The assistant cannot mark data safe, enable cleanup, select findings or execute deletion.
      </p>
      {error && <p className="error-banner">{error}</p>}
    </section>
  );
}
