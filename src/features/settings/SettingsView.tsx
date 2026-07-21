import { useEffect, useState } from "react";
import { formatBytes } from "../../lib/format";
import type { AppPreferences, DefaultScanProfile } from "../../lib/settings";
import {
  clearCleanupRecords,
  clearScanHistory,
  getAppDataSummary,
  resetAppData
} from "../../lib/tauri";
import { UpdateControl } from "../update/UpdateControl";
import type { AppDataSummary } from "./settingsTypes";
import "./SettingsView.css";

interface SettingsViewProps {
  preferences: AppPreferences;
  scanning: boolean;
  onPreferencesChange: (preferences: AppPreferences) => void;
  onHistoryCleared: () => void;
  onRecordsCleared: () => void;
  onResetComplete: () => void;
}

const scanProfiles: { value: DefaultScanProfile; label: string; note: string }[] = [
  { value: "quick", label: "Quick", note: "Common temporary files only" },
  { value: "balanced", label: "Balanced", note: "Recommended everyday scan" },
  { value: "deep", label: "Deep", note: "Projects, app data and large folders" },
  { value: "ultra", label: "Ultra", note: "Broadest supported scan" }
];

export function SettingsView({
  preferences,
  scanning,
  onPreferencesChange,
  onHistoryCleared,
  onRecordsCleared,
  onResetComplete
}: SettingsViewProps) {
  const [summary, setSummary] = useState<AppDataSummary | null>(null);
  const [loading, setLoading] = useState(true);
  const [busyAction, setBusyAction] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [resetOpen, setResetOpen] = useState(false);
  const [resetText, setResetText] = useState("");
  const [includeRestoreFiles, setIncludeRestoreFiles] = useState(false);

  useEffect(() => {
    void refreshSummary();
  }, []);

  async function refreshSummary() {
    setLoading(true);
    try {
      setSummary(await getAppDataSummary());
    } catch (error) {
      setMessage(String(error));
    } finally {
      setLoading(false);
    }
  }

  async function runAction(name: string, action: () => Promise<{ removedEntries: number; removedBytes: number }>) {
    setBusyAction(name);
    setMessage(null);
    try {
      const result = await action();
      setMessage(`Removed ${result.removedEntries} stored entries using ${formatBytes(result.removedBytes)}.`);
      await refreshSummary();
    } catch (error) {
      setMessage(String(error));
    } finally {
      setBusyAction(null);
    }
  }

  async function handleClearHistory() {
    if (!window.confirm("Remove every saved scan and History baseline? This does not delete files from your PC.")) return;
    await runAction("history", async () => {
      const result = await clearScanHistory();
      onHistoryCleared();
      return result;
    });
  }

  async function handleClearRecords() {
    if (!window.confirm("Remove all stored cleanup receipts? Restore files are not affected.")) return;
    await runAction("records", async () => {
      const result = await clearCleanupRecords();
      onRecordsCleared();
      return result;
    });
  }

  async function handleReset() {
    if (resetText !== "RESET" || scanning) return;
    setBusyAction("reset");
    setMessage(null);
    try {
      const result = await resetAppData(includeRestoreFiles);
      onResetComplete();
      setResetOpen(false);
      setResetText("");
      setIncludeRestoreFiles(false);
      setMessage(
        `WinReclaim was reset. Removed ${result.removedEntries} stored entries using ${formatBytes(result.removedBytes)}.`
      );
      await refreshSummary();
    } catch (error) {
      setMessage(String(error));
    } finally {
      setBusyAction(null);
    }
  }

  return (
    <section className="page settings-view">
      <header className="page-header simple-page-header">
        <div>
          <span className="page-kicker">Application</span>
          <h1>Settings</h1>
          <p>Control updates, default scan behaviour and data stored by WinReclaim.</p>
        </div>
      </header>

      <section className="surface settings-section">
        <div className="settings-section-head">
          <div>
            <span className="surface-label">General</span>
            <h2>Scan defaults</h2>
          </div>
        </div>

        <label className="settings-field">
          <span>
            <strong>Default scan depth</strong>
            <small>The scan page opens with this profile selected.</small>
          </span>
          <select
            value={preferences.defaultScanProfile}
            onChange={(event) =>
              onPreferencesChange({
                ...preferences,
                defaultScanProfile: event.target.value as DefaultScanProfile
              })
            }
          >
            {scanProfiles.map((profile) => (
              <option key={profile.value} value={profile.value}>
                {profile.label} — {profile.note}
              </option>
            ))}
          </select>
        </label>
      </section>

      <section className="surface settings-section">
        <div className="settings-section-head">
          <div>
            <span className="surface-label">Updates</span>
            <h2>Application updates</h2>
            <p>Only signed WinReclaim packages are accepted by the built-in updater.</p>
          </div>
        </div>

        <label className="settings-toggle-row">
          <span>
            <strong>Check automatically after launch</strong>
            <small>Disabling this stops the background check. Manual checks remain available.</small>
          </span>
          <input
            type="checkbox"
            checked={preferences.automaticUpdateChecks}
            onChange={(event) =>
              onPreferencesChange({
                ...preferences,
                automaticUpdateChecks: event.target.checked
              })
            }
          />
          <i aria-hidden="true" />
        </label>

        <UpdateControl automaticCheck={false} variant="settings" />
      </section>

      <section className="surface settings-section">
        <div className="settings-section-head settings-data-head">
          <div>
            <span className="surface-label">Local data</span>
            <h2>History and records</h2>
            <p>These controls only remove WinReclaim&apos;s own stored data.</p>
          </div>
          <button className="button button-secondary" type="button" onClick={() => void refreshSummary()} disabled={loading}>
            {loading ? "Refreshing…" : "Refresh"}
          </button>
        </div>

        <div className="settings-data-grid">
          <DataMetric label="Saved scans" value={summary ? String(summary.snapshotCount) : "—"} />
          <DataMetric label="Cleanup receipts" value={summary ? String(summary.receiptCount) : "—"} />
          <DataMetric label="Restore entries" value={summary ? String(summary.vaultEntryCount) : "—"} />
          <DataMetric label="Restore storage" value={summary ? formatBytes(summary.vaultBytes) : "—"} />
        </div>

        <div className="settings-action-list">
          <div>
            <span>
              <strong>Clear scan history</strong>
              <small>Removes old scans, charts and comparison baselines.</small>
            </span>
            <button
              className="button button-secondary"
              type="button"
              disabled={busyAction !== null || scanning || !summary?.snapshotCount}
              onClick={() => void handleClearHistory()}
            >
              {busyAction === "history" ? "Clearing…" : "Clear history"}
            </button>
          </div>
          <div>
            <span>
              <strong>Clear cleanup receipts</strong>
              <small>Removes stored execution records without touching Restore files.</small>
            </span>
            <button
              className="button button-secondary"
              type="button"
              disabled={busyAction !== null || scanning || !summary?.receiptCount}
              onClick={() => void handleClearRecords()}
            >
              {busyAction === "records" ? "Clearing…" : "Clear receipts"}
            </button>
          </div>
        </div>

        {summary?.root && (
          <div className="settings-data-path">
            <span>App data location</span>
            <code>{summary.root}</code>
          </div>
        )}
      </section>

      <section className="surface settings-section settings-danger-zone">
        <div className="settings-section-head">
          <div>
            <span className="surface-label">Reset</span>
            <h2>Start WinReclaim fresh</h2>
            <p>Clears scans, history, plans, receipts and saved preferences. Restore files are preserved unless you explicitly include them below.</p>
          </div>
          {!resetOpen && (
            <button className="button button-danger" type="button" disabled={scanning} onClick={() => setResetOpen(true)}>
              Reset WinReclaim
            </button>
          )}
        </div>

        {resetOpen && (
          <div className="settings-reset-confirmation">
            <label className="settings-toggle-row is-dangerous">
              <span>
                <strong>Also permanently delete Restore files</strong>
                <small>This removes quarantined payloads and cannot be undone.</small>
              </span>
              <input
                type="checkbox"
                checked={includeRestoreFiles}
                onChange={(event) => setIncludeRestoreFiles(event.target.checked)}
              />
              <i aria-hidden="true" />
            </label>

            <label className="settings-reset-input">
              <span>Type RESET to confirm</span>
              <input value={resetText} onChange={(event) => setResetText(event.target.value)} autoComplete="off" />
            </label>

            <div className="settings-reset-actions">
              <button className="button button-secondary" type="button" onClick={() => setResetOpen(false)} disabled={busyAction === "reset"}>
                Cancel
              </button>
              <button
                className="button button-danger"
                type="button"
                disabled={resetText !== "RESET" || busyAction !== null || scanning}
                onClick={() => void handleReset()}
              >
                {busyAction === "reset" ? "Resetting…" : "Reset application"}
              </button>
            </div>
          </div>
        )}
      </section>

      {scanning && <p className="settings-notice">Data controls are disabled while a scan is running.</p>}
      {message && <p className="settings-message" role="status">{message}</p>}
    </section>
  );
}

function DataMetric({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}
