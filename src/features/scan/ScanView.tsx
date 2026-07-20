import { useState } from "react";
import { ScanIcon, ShieldIcon } from "../../components/Icons";
import { formatBytes } from "../../lib/format";
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
  quick: "Checks common temporary files. Fastest.",
  balanced: "Checks common cleanup locations. Recommended for most people.",
  deep: "Also inspects build folders, app data and other large locations.",
  ultra: "Checks every supported location with the broadest search. Slowest."
};

const profileOptions: Record<ScanProfile, ScanOptions> = {
  quick: {
    mode: "quick",
    includeKnownTargets: true,
    includeProjectOutputs: false,
    discoverUnknown: false,
    includeAppData: false,
    includeSystemDriveCaches: false,
    minimumFindingBytes: 512 * 1024 * 1024,
    maxUnknownFindings: 10
  },
  balanced: {
    mode: "balanced",
    includeKnownTargets: true,
    includeProjectOutputs: false,
    discoverUnknown: false,
    includeAppData: false,
    includeSystemDriveCaches: true,
    minimumFindingBytes: 512 * 1024 * 1024,
    maxUnknownFindings: 20
  },
  deep: {
    mode: "deep",
    includeKnownTargets: true,
    includeProjectOutputs: true,
    discoverUnknown: true,
    includeAppData: true,
    includeSystemDriveCaches: true,
    minimumFindingBytes: 256 * 1024 * 1024,
    maxUnknownFindings: 40
  },
  ultra: {
    mode: "deep",
    includeKnownTargets: true,
    includeProjectOutputs: true,
    discoverUnknown: true,
    includeAppData: true,
    includeSystemDriveCaches: true,
    minimumFindingBytes: 64 * 1024 * 1024,
    maxUnknownFindings: 100
  }
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
  const [options, setOptions] = useState<ScanOptions>({ ...profileOptions.balanced });

  const progressValue = progress?.totalTargets
    ? Math.round((progress.completedTargets / progress.totalTargets) * 100)
    : 0;
  const cleanableBytes = report?.findings
    .filter((finding) => finding.actionAvailable)
    .reduce((sum, finding) => sum + finding.estimatedBytes, 0) ?? 0;
  const cleanableItems = report?.findings.filter((finding) => finding.actionAvailable).length ?? 0;
  const reviewOnlyItems = report?.findings.filter((finding) => !finding.actionAvailable).length ?? 0;
  const ultraLocked = profile === "ultra";

  function setFlag(key: BooleanScanOption, value: boolean) {
    setOptions((current) => ({ ...current, [key]: value }));
  }

  function selectProfile(nextProfile: ScanProfile) {
    setProfile(nextProfile);
    setOptions({ ...profileOptions[nextProfile] });
  }

  return (
    <section className="page scan-view simple-scan-view">
      <header className="page-header simple-page-header">
        <div>
          <span className="page-kicker">WinReclaim</span>
          <h1>Free up space safely</h1>
          <p>Scan your PC, review the recommendation, then clean. Nothing is removed without your confirmation.</p>
        </div>
      </header>

      <section className="surface simple-scan-card">
        {!scanning && !report && (
          <div className="simple-scan-start">
            <span className="simple-scan-icon"><ScanIcon /></span>
            <h2>Find files you no longer need</h2>
            <p>WinReclaim looks for temporary files, old caches and other space you can safely recover.</p>
            <button className="button button-primary simple-primary-action" onClick={() => onStart(options)}>
              Scan my PC
            </button>
            <span className="simple-action-note">Recommended scan selected · usually no setup needed</span>
          </div>
        )}

        {scanning && (
          <div className="simple-scan-progress">
            <span className="simple-scan-icon is-scanning"><ScanIcon /></span>
            <h2>Looking for space you can free</h2>
            <p>{progress?.phase ?? "Starting scan"}</p>
            <div className="simple-progress-track" aria-label={`${progressValue}% complete`}>
              <span style={{ width: `${Math.max(3, progressValue)}%` }} />
            </div>
            <div className="simple-progress-numbers">
              <strong>{progressValue}%</strong>
              <span>{formatBytes(progress?.discoveredBytes ?? 0)} found so far</span>
            </div>
            <button className="button button-secondary" onClick={onCancel}>Stop scan</button>
          </div>
        )}

        {!scanning && report && (
          <div className="simple-scan-result">
            <span className="simple-result-check" aria-hidden="true">✓</span>
            <span className="page-kicker">Scan complete</span>
            <h2>{formatBytes(cleanableBytes)} can be cleaned</h2>
            <p>
              {cleanableItems} item{cleanableItems === 1 ? " is" : "s are"} ready for review.
              {reviewOnlyItems > 0 ? ` ${reviewOnlyItems} other large folders are shown separately and will not be removed.` : ""}
            </p>
            <div className="simple-result-actions">
              <button className="button button-primary simple-primary-action" onClick={onContinue}>
                See what is safe to clean
              </button>
              <button className="button button-secondary" onClick={() => onStart(options)}>Scan again</button>
            </div>
          </div>
        )}

        {error && <p className="error-banner">{error}</p>}
      </section>

      {!scanning && (
        <details className="surface advanced-scan-settings">
          <summary>Advanced scan options</summary>
          <div className="advanced-scan-content">
            <div>
              <span className="surface-label">Scan depth</span>
              <div className="segmented-control" role="group" aria-label="Scan depth">
                {profiles.map((item) => (
                  <button
                    key={item}
                    type="button"
                    className={profile === item ? "is-active" : ""}
                    onClick={() => selectProfile(item)}
                  >
                    {item[0].toUpperCase() + item.slice(1)}
                  </button>
                ))}
              </div>
              <p className="config-help">{modeCopy[profile]}</p>
            </div>

            <div className="advanced-option-grid">
              <label className="config-field">
                <span>Smallest extra folder to show</span>
                <select
                  value={options.minimumFindingBytes}
                  disabled={ultraLocked}
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
                <span>Maximum extra folders</span>
                <select
                  value={options.maxUnknownFindings}
                  disabled={!options.discoverUnknown || ultraLocked}
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
            </div>

            <div className="scan-toggles advanced-toggle-grid">
              <Toggle label="Known temporary and cache folders" checked={options.includeKnownTargets} disabled={ultraLocked} onChange={(value) => setFlag("includeKnownTargets", value)} />
              <Toggle label="Build and dependency folders" checked={options.includeProjectOutputs} disabled={ultraLocked} onChange={(value) => setFlag("includeProjectOutputs", value)} />
              <Toggle label="Other large folders" checked={options.discoverUnknown} disabled={ultraLocked} onChange={(value) => setFlag("discoverUnknown", value)} />
              <Toggle label="App data" checked={options.includeAppData} disabled={!options.discoverUnknown || ultraLocked} onChange={(value) => setFlag("includeAppData", value)} />
              <Toggle label="Windows cache folders" checked={options.includeSystemDriveCaches} disabled={ultraLocked} onChange={(value) => setFlag("includeSystemDriveCaches", value)} />
            </div>
          </div>
        </details>
      )}

      <section className="simple-safety-strip">
        <ShieldIcon />
        <div>
          <strong>Designed to avoid important files</strong>
          <span>Personal files, installed apps and folders without a verified cleanup rule are never automatically selected.</span>
        </div>
      </section>
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
