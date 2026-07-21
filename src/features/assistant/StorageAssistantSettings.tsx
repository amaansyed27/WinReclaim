import { useEffect, useMemo, useState } from "react";
import { formatBytes } from "../../lib/format";
import {
  getStorageAssistantStatus,
  installStorageAssistant,
  onAssistantDownloadProgress,
  uninstallStorageAssistant
} from "./assistantApi";
import type {
  AssistantDownloadProgress,
  StorageAssistantStatus
} from "./assistantTypes";
import "./StorageAssistant.css";

interface StorageAssistantSettingsProps {
  disabled: boolean;
}

export function StorageAssistantSettings({ disabled }: StorageAssistantSettingsProps) {
  const [status, setStatus] = useState<StorageAssistantStatus | null>(null);
  const [progress, setProgress] = useState<AssistantDownloadProgress | null>(null);
  const [loading, setLoading] = useState(true);
  const [working, setWorking] = useState(false);
  const [message, setMessage] = useState<string | null>(null);

  useEffect(() => {
    let dispose: (() => void) | undefined;
    void refresh();
    onAssistantDownloadProgress(setProgress).then((unlisten) => {
      dispose = unlisten;
    });
    return () => dispose?.();
  }, []);

  const percent = useMemo(() => {
    if (!progress?.totalBytes) return 0;
    return Math.min(100, Math.round((progress.downloadedBytes / progress.totalBytes) * 100));
  }, [progress]);

  async function refresh() {
    setLoading(true);
    try {
      setStatus(await getStorageAssistantStatus());
    } catch (error) {
      setMessage(String(error));
    } finally {
      setLoading(false);
    }
  }

  async function install() {
    setWorking(true);
    setMessage(null);
    setProgress({ phase: "connecting", downloadedBytes: 0, totalBytes: status?.expectedBytes ?? 0 });
    try {
      const next = await installStorageAssistant();
      setStatus(next);
      setMessage("Storage Assistant installed and verified.");
    } catch (error) {
      setMessage(String(error));
      await refresh();
    } finally {
      setWorking(false);
    }
  }

  async function uninstall() {
    if (!window.confirm("Remove the downloaded Storage Assistant model? Scan history and cleanup records are not affected.")) return;
    setWorking(true);
    setMessage(null);
    try {
      const next = await uninstallStorageAssistant();
      setStatus(next);
      setProgress(null);
      setMessage("Storage Assistant model removed.");
    } catch (error) {
      setMessage(String(error));
    } finally {
      setWorking(false);
    }
  }

  const installed = Boolean(status?.installed && status.verified);
  const modelSize = status?.modelBytes || status?.expectedBytes || 0;

  return (
    <section className="surface settings-section assistant-settings-section">
      <div className="settings-section-head assistant-settings-head">
        <div>
          <span className="surface-label">Optional local model</span>
          <h2>Storage Assistant</h2>
          <p>
            Generates a compact storage summary and clearer labels for ambiguous folders. It never changes cleanup safety rules or selections.
          </p>
        </div>
        <span className={`assistant-status-pill ${installed ? "is-ready" : ""}`}>
          {loading ? "Checking…" : installed ? "Installed" : "Not installed"}
        </span>
      </div>

      <div className="assistant-model-card">
        <div>
          <strong>{status?.model ?? "Qwen3.5-2B Q4_K_M"}</strong>
          <span>{status?.runtime ?? "Local CPU inference"}</span>
        </div>
        <div>
          <strong>{modelSize ? formatBytes(modelSize) : "About 1.4 GB"}</strong>
          <span>{status?.license ?? "Apache-2.0"}</span>
        </div>
      </div>

      {working && progress && (
        <div className="assistant-download" aria-live="polite">
          <div>
            <span>{downloadLabel(progress.phase)}</span>
            <strong>{progress.totalBytes ? `${percent}%` : "Starting"}</strong>
          </div>
          <div className="assistant-download-track">
            <span style={{ width: `${Math.max(2, percent)}%` }} />
          </div>
          <small>
            {progress.downloadedBytes ? formatBytes(progress.downloadedBytes) : "0 B"}
            {progress.totalBytes ? ` of ${formatBytes(progress.totalBytes)}` : ""}
          </small>
        </div>
      )}

      <div className="assistant-boundaries">
        <span>Local-only</span>
        <span>Advisory output</span>
        <span>No file-content reading</span>
        <span>No cleanup authority</span>
      </div>

      <p className="assistant-privacy-note">
        {status?.privacyNote ??
          "The model receives only the completed scan report and runs entirely on this PC."}
      </p>

      {status?.modelPath && installed && (
        <div className="settings-data-path">
          <span>Model location</span>
          <code>{status.modelPath}</code>
        </div>
      )}

      <div className="assistant-settings-actions">
        <button
          className="button button-secondary"
          type="button"
          disabled={disabled || working || loading}
          onClick={() => void refresh()}
        >
          Refresh status
        </button>
        {installed ? (
          <button
            className="button button-danger"
            type="button"
            disabled={disabled || working}
            onClick={() => void uninstall()}
          >
            {working ? "Removing…" : "Remove model"}
          </button>
        ) : (
          <button
            className="button button-primary"
            type="button"
            disabled={disabled || working || loading}
            onClick={() => void install()}
          >
            {working ? "Installing…" : "Download and install"}
          </button>
        )}
      </div>

      {message && <p className="settings-message" role="status">{message}</p>}
    </section>
  );
}

function downloadLabel(phase: string): string {
  switch (phase) {
    case "connecting":
      return "Connecting to model host";
    case "downloading":
      return "Downloading model";
    case "verifying":
      return "Verifying SHA-256";
    case "ready":
      return "Ready";
    default:
      return phase;
  }
}
