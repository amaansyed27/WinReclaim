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

type BooleanScanOption =
  | "includeKnownTargets"
  | "includeProjectOutputs"
  | "discoverUnknown"
  | "includeAppData"
  | "includeSystemDriveCaches";

type ScanProfile = ScanMode | "ultra";

const profiles: ScanProfile[] = ["quick", "balanced", "deep", "ultra"];

const modeCopy: Record<ScanProfile, string> = {
  quick: "Checks the most common places. Fastest.",
  balanced: "Checks common places and project folders. Best for most people.",
  deep: "Looks through more folders. Finds more, but takes longer.",
  ultra: "Checks every supported location with the broadest search. Slowest."
};

const defaultOptions: ScanOptions = {
  mode: "balanced",
  includeKnownTargets: true,
  includeProjectOutputs: true,
  discoverUnknown: true,
  includeAppData: true,
  includeSystemDriveCaches: true,
  minimumFindingBytes: 256 * 1024 * 1024,
  maxUnknownFindings: 20
};

const ultraOptions: ScanOptions = {
  mode: "deep",
  includeKnownTargets: true,
  includeProjectOutputs: true,
  discoverUnknown: true,
  includeAppData: true,
  includeSystemDriveCaches: true,
  minimumFindingBytes: 64 * 1024 * 1024,
  maxUnknownFindings: 100
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
  const [profile, setProfile] = useState<ScanProfile>("balanced");
  const [options, setOptions] = useState<ScanOptions>(defaultOptions);

  const progressValue = progress?.totalTargets
    ? Math.round((progress.completedTargets / progress.totalTargets) * 100)
    : 0;
  const unknownCount =
    report?.findings.filter((finding) => finding.ruleId === "dynamic.large_directory").length ?? 0;
  const ultraLocked = profile === "ultra";

  function setFlag(key: BooleanScanOption, value: boolean) {
    setOptions((current) => ({ ...current, [key]: value }));
  }

  function selectProfile(nextProfile: ScanProfile) {
    setProfile(nextProfile);
    if (nextProfile === "ultra") {
      setOptions(ultraOptions);
      return;
    }
    setOptions((current) => ({ ...current, mode: nextProfile }));
  }

  return (
    <section className="page scan-view">
      <header className="page-header">
        <div>
          <span className="page-kicker">Find space to free</span>
          <h1>Scan your PC</h1>
          <p>WinReclaim only looks for files. Nothing is removed until you review and confirm.</p>
        </div>
        <span className={`status-chip ${scanning ? "is-running" : report ? "is-ready" : ""}`}>
          {scanning ? "Scanning" : report ? "Complete" : "Ready"}
        </span>
      </header>

      <div className="scan-dashboard">
        <section className="surface scan-primary-panel">
          <div className="surface-header">
            <div className="surface-title">
              <span className="surface-icon">
                <ScanIcon />
              </span>
              <div>
                <h2>This Windows account</h2>
                <span>
                  {scanning ? progress?.phase ?? "Starting" : report ? "Latest scan" : "Not scanned yet"}
                </span>
              </div>
            </div>
            {!scanning ? (
              <button className="button button-primary" onClick={() => onStart(options)}>
                {report ? "Scan again" : "Start scan"}
              </button>
            ) : (
              <button className="button button-secondary" onClick={onCancel}>
                Stop scan
              </button>
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
                  <span>Space found</span>
                  <strong>{formatBytes(progress?.discoveredBytes ?? 0)}</strong>
                </div>
                <div>
                  <span>Files checked</span>
                  <strong>{(progress?.scannedEntries ?? 0).toLocaleString()}</strong>
                </div>
              </div>
              <div className="progress-track">
                <span style={{ width: `${Math.max(3, progressValue)}%` }} />
              </div>
              <p className="path-line">{progress?.currentPath ?? "Preparing folders"}</p>
            </div>
          )}

          {report && !scanning && (
            <>
              <div className="disk-overview">
                <div className="disk-copy">
                  <span>Space currently used</span>
                  <strong>{formatBytes(report.disk.usedBytes)}</strong>
                  <small>of {formatBytes(report.disk.totalBytes)}</small>
                </div>
                <div className="disk-meter-wrap">
                  <div className="disk-meter">
                    <span
                      style={{
                        width: formatPercent(report.disk.usedBytes, report.disk.totalBytes)
                      }}
                    />
                  </div>
                  <div className="disk-meter-labels">
                    <span>{formatBytes(report.disk.freeBytes)} free</span>
                    <span>
                      {report.findings.length} items found · {unknownCount} extra folders
                    </span>
                  </div>
                </div>
              </div>
              <div className="scan-result-actions">
                <button className="button button-primary" onClick={onContinue}>
                  Review what was found
                </button>
              </div>
            </>
          )}

          {!scanning && !report && (
            <div className="empty-state compact-empty-state">
              <strong>Ready to scan</strong>
              <span>Unknown folders are shown for information only and are never cleaned automatically.</span>
            </div>
          )}

          {error && <p className="error-banner">{error}</p>}
        </section>

        <aside className="scan-side-column">
          <section className="surface scan-config-panel">
            <span className="surface-label">How thorough?</span>
            <div className="segmented-control" role="group" aria-label="Scan thoroughness">
              {profiles.map((item) => (
                <button
                  key={item}
                  type="button"
                  className={profile === item ? "is-active" : ""}
                  disabled={scanning}
                  onClick={() => selectProfile(item)}
                >
                  {item[0].toUpperCase() + item.slice(1)}
                </button>
              ))}
            </div>
            <p className="config-help">{modeCopy[profile]}</p>

            <label className="config-field">
              <span>Smallest folder to show</span>
              <select
                value={options.minimumFindingBytes}
                disabled={scanning || ultraLocked}
                onChange={(event) =>
                  setOptions((current) => ({
                    ...current,
                    minimumFindingBytes: Number(event.target.value)
                  }))
                }
              >
                <option value={64 * 1024 * 1024}>64 MB</option>
                <option value={256 * 1024 * 1024}>256 MB</option>
                <option value={512 * 1024 * 1024}>512 MB</option>
                <option value={1024 * 1024 * 1024}>1 GB</option>
              </select>
            </label>

            <label className="config-field">
              <span>Maximum extra folders to show</span>
              <select
                value={options.maxUnknownFindings}
                disabled={scanning || !options.discoverUnknown || ultraLocked}
                onChange={(event) =>
                  setOptions((current) => ({
                    ...current,
                    maxUnknownFindings: Number(event.target.value)
                  }))
                }
              >
                <option value={10}>10</option>
                <option value={20}>20</option>
                <option value={40}>40</option>
                <option value={75}>75</option>
                <option value={100}>100</option>
              </select>
            </label>

            <div className="scan-toggles">
              <Toggle
                label="Known app and tool folders"
                checked={options.includeKnownTargets}
                disabled={scanning || ultraLocked}
                onChange={(value) => setFlag("includeKnownTargets", value)}
              />
              <Toggle
                label="Project build files"
                checked={options.includeProjectOutputs}
                disabled={scanning || ultraLocked}
                onChange={(value) => setFlag("includeProjectOutputs", value)}
              />
              <Toggle
                label="Find other large folders"
                checked={options.discoverUnknown}
                disabled={scanning || ultraLocked}
                onChange={(value) => setFlag("discoverUnknown", value)}
              />
              <Toggle
                label="Look inside app data"
                checked={options.includeAppData}
                disabled={scanning || !options.discoverUnknown || ultraLocked}
                onChange={(value) => setFlag("includeAppData", value)}
              />
              <Toggle
                label="Check Windows cache folders"
                checked={options.includeSystemDriveCaches}
                disabled={scanning || ultraLocked}
                onChange={(value) => setFlag("includeSystemDriveCaches", value)}
              />
            </div>
          </section>

          <section className="surface compact-surface protection-panel">
            <span className="surface-label">Safety</span>
            <div className="info-row">
              <ShieldIcon />
              <div>
                <strong>Unknown folders are not cleaned</strong>
                <span>Only locations with a verified cleanup rule can be selected.</span>
              </div>
            </div>
            <div className="info-row">
              <span className="info-glyph">01</span>
              <div>
                <strong>Everything stays on this PC</strong>
                <span>Folder paths and project names are not uploaded.</span>
              </div>
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
      <input
        type="checkbox"
        checked={checked}
        disabled={disabled}
        onChange={(event) => onChange(event.target.checked)}
      />
      <i aria-hidden="true" />
    </label>
  );
}
