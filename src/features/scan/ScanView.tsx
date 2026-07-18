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
    <section className="view scan-view">
      <div className="hero-copy">
        <p className="eyebrow">Your disk, explained.</p>
        <h1>Find what grew. Understand the cost. Reclaim it safely.</h1>
        <p className="hero-description">
          WinReclaim maps developer tools, local AI models, SDKs, caches and build
          output without uploading your file tree or guessing what should be deleted.
        </p>
      </div>

      <div className="scan-panel">
        <div className="scan-panel-head">
          <div className="scan-emblem">
            <ScanIcon />
          </div>
          <div>
            <p className="kicker">Current user profile</p>
            <h2>{scanning ? "Reading storage signals" : report ? "Scan complete" : "Ready to inspect"}</h2>
          </div>
        </div>

        {scanning && (
          <div className="progress-block">
            <div className="progress-meta">
              <span>{progress?.phase ?? "Starting scanner"}</span>
              <strong>{progressValue}%</strong>
            </div>
            <div className="progress-track">
              <span style={{ width: `${Math.max(3, progressValue)}%` }} />
            </div>
            <p className="path-line">{progress?.currentPath ?? "Preparing known storage locations"}</p>
            <div className="scan-stats-inline">
              <span>{formatBytes(progress?.discoveredBytes ?? 0)} identified</span>
              <span>{(progress?.scannedEntries ?? 0).toLocaleString()} entries checked</span>
            </div>
          </div>
        )}

        {report && !scanning && (
          <div className="disk-summary">
            <div className="disk-number">
              <strong>{formatBytes(report.disk.usedBytes)}</strong>
              <span>used of {formatBytes(report.disk.totalBytes)}</span>
            </div>
            <div className="disk-meter">
              <span style={{ width: formatPercent(report.disk.usedBytes, report.disk.totalBytes) }} />
            </div>
            <div className="disk-summary-foot">
              <span>{formatBytes(report.disk.freeBytes)} free</span>
              <span>{report.findings.length} meaningful findings</span>
            </div>
          </div>
        )}

        {error && <p className="error-banner">{error}</p>}

        <div className="scan-actions">
          {!scanning && !report && (
            <button className="button button-primary" onClick={onStart}>Scan my profile</button>
          )}
          {scanning && (
            <button className="button button-secondary" onClick={onCancel}>Cancel scan</button>
          )}
          {report && !scanning && (
            <>
              <button className="button button-primary" onClick={onContinue}>Review findings</button>
              <button className="button button-quiet" onClick={onStart}>Scan again</button>
            </>
          )}
        </div>
      </div>

      <div className="trust-line">
        <ShieldIcon />
        <span>Local scan. Deterministic rules. Nothing is removed during discovery.</span>
      </div>
    </section>
  );
}
