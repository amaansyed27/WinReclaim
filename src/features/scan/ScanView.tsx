import { ScanIcon, ShieldIcon } from "../../components/Icons";
import { formatBytes, formatPercent } from "../../lib/format";
import type { ScanProgress, ScanReport } from "../../types";

interface ScanViewProps {
  scanning: boolean;
  progress: ScanProgress | null;
  report: ScanReport | null;
  error: string | null;
  onStart: () => void;
  onCancel: () => void;
  onContinue: () => void;
}

export function ScanView({
  scanning,
  progress,
  report,
  error,
  onStart,
  onCancel,
  onContinue
}: ScanViewProps) {
  const progressValue = progress?.totalTargets
    ? Math.round((progress.completedTargets / progress.totalTargets) * 100)
    : 0;

  return (
    <section className="page scan-view">
      <header className="page-header">
        <div>
          <span className="page-kicker">Storage</span>
          <h1>Scan</h1>
          <p>Inspect known developer, AI, SDK and build locations.</p>
        </div>
        <span className={`status-chip ${scanning ? "is-running" : report ? "is-ready" : ""}`}>
          {scanning ? "Scanning" : report ? "Complete" : "Ready"}
        </span>
      </header>

      <div className="scan-dashboard">
        <section className="surface scan-primary-panel">
          <div className="surface-header">
            <div className="surface-title">
              <span className="surface-icon"><ScanIcon /></span>
              <div>
                <h2>Current Windows profile</h2>
                <span>{scanning ? progress?.phase ?? "Starting" : report ? "Latest scan" : "Not scanned"}</span>
              </div>
            </div>
            {!scanning && !report && (
              <button className="button button-primary" onClick={onStart}>Start scan</button>
            )}
            {scanning && (
              <button className="button button-secondary" onClick={onCancel}>Cancel</button>
            )}
          </div>

          {scanning && (
            <div className="scan-progress-panel">
              <div className="metric-row">
                <div>
                  <span>Progress</span>
                  <strong>{progressValue}%</strong>
                </div>
                <div>
                  <span>Identified</span>
                  <strong>{formatBytes(progress?.discoveredBytes ?? 0)}</strong>
                </div>
                <div>
                  <span>Entries</span>
                  <strong>{(progress?.scannedEntries ?? 0).toLocaleString()}</strong>
                </div>
              </div>
              <div className="progress-track"><span style={{ width: `${Math.max(3, progressValue)}%` }} /></div>
              <p className="path-line">{progress?.currentPath ?? "Preparing storage targets"}</p>
            </div>
          )}

          {report && !scanning && (
            <>
              <div className="disk-overview">
                <div className="disk-copy">
                  <span>Disk usage</span>
                  <strong>{formatBytes(report.disk.usedBytes)}</strong>
                  <small>of {formatBytes(report.disk.totalBytes)}</small>
                </div>
                <div className="disk-meter-wrap">
                  <div className="disk-meter"><span style={{ width: formatPercent(report.disk.usedBytes, report.disk.totalBytes) }} /></div>
                  <div className="disk-meter-labels">
                    <span>{formatBytes(report.disk.freeBytes)} free</span>
                    <span>{report.findings.length} findings</span>
                  </div>
                </div>
              </div>

              <div className="scan-result-actions">
                <button className="button button-primary" onClick={onContinue}>Open findings</button>
                <button className="button button-secondary" onClick={onStart}>Scan again</button>
              </div>
            </>
          )}

          {!scanning && !report && (
            <div className="empty-state compact-empty-state">
              <strong>No scan data yet</strong>
              <span>Discovery is read-only and does not remove files.</span>
            </div>
          )}

          {error && <p className="error-banner">{error}</p>}
        </section>

        <aside className="scan-side-column">
          <section className="surface compact-surface">
            <span className="surface-label">Protection</span>
            <div className="info-row">
              <ShieldIcon />
              <div>
                <strong>Read-only discovery</strong>
                <span>No cleanup runs during a scan.</span>
              </div>
            </div>
            <div className="info-row">
              <span className="info-glyph">01</span>
              <div>
                <strong>Local processing</strong>
                <span>Paths and project names stay on this PC.</span>
              </div>
            </div>
          </section>

          <section className="surface compact-surface">
            <span className="surface-label">Scan scope</span>
            <ul className="scope-list">
              <li>Tool and package caches</li>
              <li>Local models and SDK data</li>
              <li>Project build output</li>
              <li>Browser and app storage</li>
            </ul>
          </section>
        </aside>
      </div>
    </section>
  );
}
