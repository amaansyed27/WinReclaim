import { useState } from "react";
import { ScanIcon, ShieldIcon } from "../../components/Icons";
import { formatBytes, formatPercent } from "../../lib/format";
import type { ScanMode, ScanOptions, ScanProgress, ScanReport } from "../../types";

interface ScanViewProps {
  scanning: boolean;
  progress: ScanProgress | null;
  report: ScanReport | null;
  error: string | null;
  onStart: (options: ScanOptions) => void;
  onCancel: () => void;
  onContinue: () => void;
}

const modeCopy: Record<ScanMode, string> = {
  quick: "Known locations and a bounded discovery pass.",
  balanced: "Broader project and unclassified directory discovery.",
  deep: "Largest entry budget and deepest directory traversal."
};

export function ScanView({
  scanning,
  progress,
  report,
  error,
  onStart,
  onCancel,
  onContinue
}: ScanViewProps) {
  const [options, setOptions] = useState<ScanOptions>({
    mode: "balanced",
    includeKnownTargets: true,
    includeProjectOutputs: true,
    discoverUnknown: true,
    includeAppData: true,
    minimumFindingBytes: 256 * 1024 * 1024,
    maxUnknownFindings: 20
  });

  const progressValue = progress?.totalTargets
    ? Math.round((progress.completedTargets / progress.totalTargets) * 100)
    : 0;
  const unknownCount = report?.findings.filter((finding) => finding.ruleId === "dynamic.large_directory").length ?? 0;

  function setFlag(key: keyof ScanOptions, value: boolean) {
    setOptions((current) => ({ ...current, [key]: value }));
  }

  return (
    <section className="page scan-view">
      <header className="page-header">
        <div>
          <span className="page-kicker">Storage</span>
          <h1>Scan</h1>
          <p>Inspect verified locations and discover large unclassified directories.</p>
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
            {!scanning && (
              <button className="button button-primary" onClick={() => onStart(options)}>
                {report ? "Scan again" : "Start scan"}
              </button>
            )}
            {scanning && <button className="button button-secondary" onClick={onCancel}>Cancel</button>}
          </div>

          {scanning && (
            <div className="scan-progress-panel">
              <div className="metric-row">
                <div><span>Progress</span><strong>{progressValue}%</strong></div>
                <div><span>Identified</span><strong>{formatBytes(progress?.discoveredBytes ?? 0)}</strong></div>
                <div><span>Entries</span><strong>{(progress?.scannedEntries ?? 0).toLocaleString()}</strong></div>
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
                    <span>{report.findings.length} findings · {unknownCount} dynamic</span>
                  </div>
                </div>
              </div>
              <div className="scan-result-actions">
                <button className="button button-primary" onClick={onContinue}>Open findings</button>
              </div>
            </>
          )}

          {!scanning && !report && (
            <div className="empty-state compact-empty-state">
              <strong>Ready for discovery</strong>
              <span>Scanning is read-only. Unknown directories are inspection-only.</span>
            </div>
          )}

          {error && <p className="error-banner">{error}</p>}
        </section>

        <aside className="scan-side-column">
          <section className="surface scan-config-panel">
            <span className="surface-label">Scan profile</span>
            <div className="segmented-control" role="group" aria-label="Scan depth">
              {(["quick", "balanced", "deep"] as ScanMode[]).map((mode) => (
                <button
                  key={mode}
                  type="button"
                  className={options.mode === mode ? "is-active" : ""}
                  disabled={scanning}
                  onClick={() => setOptions((current) => ({ ...current, mode }))}
                >
                  {mode[0].toUpperCase() + mode.slice(1)}
                </button>
              ))}
            </div>
            <p className="config-help">{modeCopy[options.mode]}</p>

            <label className="config-field">
              <span>Minimum finding size</span>
              <select
                value={options.minimumFindingBytes}
                disabled={scanning}
                onChange={(event) => setOptions((current) => ({ ...current, minimumFindingBytes: Number(event.target.value) }))}
              >
                <option value={64 * 1024 * 1024}>64 MB</option>
                <option value={256 * 1024 * 1024}>256 MB</option>
                <option value={512 * 1024 * 1024}>512 MB</option>
                <option value={1024 * 1024 * 1024}>1 GB</option>
              </select>
            </label>

            <label className="config-field">
              <span>Maximum dynamic findings</span>
              <select
                value={options.maxUnknownFindings}
                disabled={scanning || !options.discoverUnknown}
                onChange={(event) => setOptions((current) => ({ ...current, maxUnknownFindings: Number(event.target.value) }))}
              >
                <option value={10}>10</option>
                <option value={20}>20</option>
                <option value={40}>40</option>
                <option value={75}>75</option>
              </select>
            </label>

            <div className="scan-toggles">
              <Toggle label="Verified tool locations" checked={options.includeKnownTargets} disabled={scanning} onChange={(value) => setFlag("includeKnownTargets", value)} />
              <Toggle label="Project build output" checked={options.includeProjectOutputs} disabled={scanning} onChange={(value) => setFlag("includeProjectOutputs", value)} />
              <Toggle label="Discover unknown large folders" checked={options.discoverUnknown} disabled={scanning} onChange={(value) => setFlag("discoverUnknown", value)} />
              <Toggle label="Include AppData discovery" checked={options.includeAppData} disabled={scanning || !options.discoverUnknown} onChange={(value) => setFlag("includeAppData", value)} />
            </div>
          </section>

          <section className="surface compact-surface protection-panel">
            <span className="surface-label">Protection</span>
            <div className="info-row">
              <ShieldIcon />
              <div><strong>Read-only discovery</strong><span>Dynamic findings never receive cleanup actions automatically.</span></div>
            </div>
            <div className="info-row">
              <span className="info-glyph">01</span>
              <div><strong>Local processing</strong><span>Paths and project names stay on this PC.</span></div>
            </div>
          </section>
        </aside>
      </div>
    </section>
  );
}

function Toggle({
  label,
  checked,
  disabled,
  onChange
}: {
  label: string;
  checked: boolean;
  disabled: boolean;
  onChange: (value: boolean) => void;
}) {
  return (
    <label className={`toggle-row ${disabled ? "is-disabled" : ""}`}>
      <span>{label}</span>
      <input type="checkbox" checked={checked} disabled={disabled} onChange={(event) => onChange(event.target.checked)} />
      <i aria-hidden="true" />
    </label>
  );
}
