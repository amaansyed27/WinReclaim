import { useEffect, useState } from "react";
import { ScanIcon } from "../../components/Icons";
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

  const available = Boolean(status?.available);

  return (
    <section className="surface assistant-report-panel" aria-busy={loading}>
      <div className="assistant-report-head">
        <div className="assistant-title-group">
          <span className="assistant-panel-icon" aria-hidden="true"><ScanIcon /></span>
          <div>
            <span className="surface-label">Local deterministic insight</span>
            <h2>Storage Brief</h2>
            <p>Builds a concise explanation from the completed scan without a model, account, API key, or network request.</p>
          </div>
        </div>
        <span className={`assistant-status-pill ${available ? "is-ready" : ""}`}>
          <i aria-hidden="true" />
          {available ? "Ready offline" : "Unavailable"}
        </span>
      </div>

      {available && !assistantReport && !loading && (
        <div className="assistant-run-row">
          <div className="assistant-model-summary">
            <strong>{status?.provider}</strong>
            <span>Uses {status?.model} · instant local rules · no download required</span>
          </div>
          <button className="button button-primary assistant-run-button" onClick={() => void analyze()}>
            Generate brief
          </button>
        </div>
      )}

      {!available && !loading && (
        <div className="assistant-empty-state">
          <div>
            <strong>Storage Brief is unavailable</strong>
            <span>The deterministic scan and cleanup workflow still works normally.</span>
          </div>
        </div>
      )}

      {loading && (
        <div className="assistant-loading-state" role="status" aria-live="polite">
          <span className="assistant-spinner" aria-hidden="true" />
          <div>
            <strong>Building the local storage brief</strong>
            <span>WinReclaim is aggregating measured totals, safety classes, and verified action counts.</span>
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

          <div className="assistant-report-footer">
            <span>Engine: {assistantReport.model}</span>
            <button className="button button-secondary" onClick={() => void analyze()}>
              Refresh
            </button>
          </div>
        </div>
      )}

      {error && !loading && (
        <div className="assistant-error" role="alert">
          <div>
            <strong>Brief generation failed</strong>
            <span>The scan is unaffected. Retry or continue with the measured findings below.</span>
          </div>
          <button className="button button-secondary" onClick={() => void analyze()} disabled={!available}>Retry</button>
          <details>
            <summary>Technical details</summary>
            <code>{error}</code>
          </details>
        </div>
      )}

      <p className="assistant-privacy-note">
        {status?.privacyNote ?? "The brief is generated locally and does not contact a remote service."}
      </p>
      <p className="assistant-advisory-note">
        Explanatory only. The brief cannot enable cleanup, select findings, alter safety labels, or delete files.
      </p>
    </section>
  );
}
