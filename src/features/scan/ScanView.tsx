import { useEffect, useMemo, useState } from "react";
import { ScanIcon, ShieldIcon } from "../../components/Icons";
import { formatBytes } from "../../lib/format";
import { loadPreferences } from "../../lib/settings";
import { listStorageDrives } from "../../lib/tauri";
import type { DriveInfo, ScanMode, ScanOptions, ScanProgress, ScanReport } from "../../types";

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
    roots: [],
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
    roots: [],
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
    roots: [],
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
    roots: [],
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
  const initialProfile = loadPreferences().defaultScanProfile;
  const [profile, setProfile] = useState<ScanProfile>(initialProfile);
  const [options, setOptions] = useState<ScanOptions>({ ...profileOptions[initialProfile] });
  const [drives, setDrives] = useState<DriveInfo[]>([]);
  const [selectedRoots, setSelectedRoots] = useState<Set<string>>(new Set());
  const [drivesLoading, setDrivesLoading] = useState(true);
  const [driveError, setDriveError] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    listStorageDrives()
      .then((nextDrives) => {
        if (!active) return;
        setDrives(nextDrives);
        const systemDrive = nextDrives.find((drive) => drive.isSystem);
        const firstFixed = nextDrives.find((drive) => drive.kind === "fixed");
        const initial = systemDrive ?? firstFixed ?? nextDrives[0];
        if (initial) setSelectedRoots(new Set([initial.root]));
      })
      .catch((nextError) => {
        if (!active) return;
        setDriveError(String(nextError));
      })
      .finally(() => {
        if (active) setDrivesLoading(false);
      });
    return () => {
      active = false;
    };
  }, []);

  const progressValue = progress?.totalTargets
    ? Math.round((progress.completedTargets / progress.totalTargets) * 100)
    : 0;
  const cleanableBytes = report?.findings
    .filter((finding) => finding.actionAvailable)
    .reduce((sum, finding) => sum + finding.estimatedBytes, 0) ?? 0;
  const cleanableItems = report?.findings.filter((finding) => finding.actionAvailable).length ?? 0;
  const reviewOnlyItems = report?.findings.filter((finding) => !finding.actionAvailable).length ?? 0;
  const ultraLocked = profile === "ultra";
  const selectedDrives = useMemo(
    () => drives.filter((drive) => selectedRoots.has(drive.root)),
    [drives, selectedRoots]
  );
  const systemSelected = selectedDrives.some((drive) => drive.isSystem);

  function setFlag(key: BooleanScanOption, value: boolean) {
    setOptions((current) => ({ ...current, [key]: value }));
  }

  function selectProfile(nextProfile: ScanProfile) {
    setProfile(nextProfile);
    setOptions((current) => ({
      ...profileOptions[nextProfile],
      roots: current.roots
    }));
  }

  function toggleDrive(root: string) {
    setSelectedRoots((current) => {
      const next = new Set(current);
      if (next.has(root)) next.delete(root);
      else next.add(root);
      return next;
    });
  }

  function runScan() {
    onStart({
      ...options,
      roots: [...selectedRoots],
      includeSystemDriveCaches: systemSelected && options.includeSystemDriveCaches
    });
  }

  return (
    <section className="page scan-view simple-scan-view">
      <header className="page-header simple-page-header">
        <div>
          <span className="page-kicker">WinReclaim</span>
          <h1>Free up space safely</h1>
          <p>Choose one or more drives, review the recommendation, then clean. Nothing is removed without confirmation.</p>
        </div>
      </header>

      {!scanning && !report && (
        <DrivePicker
          drives={drives}
          selectedRoots={selectedRoots}
          loading={drivesLoading}
          error={driveError}
          onToggle={toggleDrive}
        />
      )}

      <section className="surface simple-scan-card">
        {!scanning && !report && (
          <div className="simple-scan-start">
            <span className="simple-scan-icon"><ScanIcon /></span>
            <h2>Find files you no longer need</h2>
            <p>
              {selectedDrives.length
                ? `${selectedDrives.length} drive${selectedDrives.length === 1 ? "" : "s"} selected.`
                : "Select at least one drive to begin."}
            </p>
            <button
              className="button button-primary simple-primary-action"
              onClick={runScan}
              disabled={drivesLoading || selectedRoots.size === 0}
            >
              Scan selected drives
            </button>
            <span className="simple-action-note">{profile[0].toUpperCase() + profile.slice(1)} scan selected · change it in Settings</span>
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
              {(report.drives ?? []).length || 1} drive{(report.drives ?? []).length === 1 ? "" : "s"} scanned. {cleanableItems} item{cleanableItems === 1 ? " is" : "s are"} ready for review.
              {reviewOnlyItems > 0 ? ` ${reviewOnlyItems} other large folders are shown separately and will not be removed.` : ""}
            </p>
            <div className="simple-result-actions">
              <button className="button button-primary simple-primary-action" onClick={onContinue}>
                Review storage report
              </button>
              <button className="button button-secondary" onClick={runScan}>Scan again</button>
            </div>
          </div>
        )}

        {(error || driveError) && <p className="error-banner">{error ?? driveError}</p>}
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
                <span>Maximum extra folders per drive</span>
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
              <Toggle label="Windows cache folders" checked={options.includeSystemDriveCaches} disabled={!systemSelected || ultraLocked} onChange={(value) => setFlag("includeSystemDriveCaches", value)} />
            </div>
          </div>
        </details>
      )}

      <section className="simple-safety-strip">
        <ShieldIcon />
        <div>
          <strong>Designed to avoid important files</strong>
          <span>Fixed drives can use verified cleanup actions. Removable and network drives are inspection-only.</span>
        </div>
      </section>
    </section>
  );
}

function DrivePicker({
  drives,
  selectedRoots,
  loading,
  error,
  onToggle
}: {
  drives: DriveInfo[];
  selectedRoots: Set<string>;
  loading: boolean;
  error: string | null;
  onToggle: (root: string) => void;
}) {
  return (
    <section className="surface drive-picker" aria-label="Choose drives to scan">
      <div className="drive-picker-head">
        <div>
          <span className="surface-label">Scan scope</span>
          <h2>Choose drives</h2>
          <p>Fixed local drives support cleanup. Removable and network drives are scanned for inspection only.</p>
        </div>
        <span>{selectedRoots.size} selected</span>
      </div>

      {loading && <div className="drive-picker-state">Detecting mounted drives…</div>}
      {!loading && error && <div className="drive-picker-state">Drive discovery failed. Restart WinReclaim and try again.</div>}
      {!loading && !error && (
        <div className="drive-grid">
          {drives.map((drive) => {
            const selected = selectedRoots.has(drive.root);
            const inspectionOnly = drive.kind === "network" || drive.kind === "removable" || drive.kind === "optical" || drive.kind === "other";
            const usedPercent = drive.totalBytes ? Math.round((drive.usedBytes / drive.totalBytes) * 100) : 0;
            return (
              <label className={`drive-card ${selected ? "is-selected" : ""}`} key={`${drive.volumeId}-${drive.root}`}>
                <input
                  type="checkbox"
                  checked={selected}
                  onChange={() => onToggle(drive.root)}
                />
                <div className="drive-card-title">
                  <strong>{drive.root.replace("\\", "")}</strong>
                  <span>{drive.label || (drive.isSystem ? "Windows" : "Local drive")}</span>
                </div>
                <div className="drive-card-meta">
                  <span>{formatBytes(drive.usedBytes)} used</span>
                  <span>{formatBytes(drive.freeBytes)} free</span>
                </div>
                <div className="drive-usage" aria-label={`${usedPercent}% used`}>
                  <span style={{ width: `${Math.min(100, usedPercent)}%` }} />
                </div>
                <small>
                  {drive.isSystem ? "System drive" : inspectionOnly ? "Inspection only" : drive.fileSystem || "Fixed drive"}
                </small>
              </label>
            );
          })}
        </div>
      )}
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
