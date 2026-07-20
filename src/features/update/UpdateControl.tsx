import { useEffect, useMemo, useState } from "react";
import {
  checkForAppUpdate,
  currentAppVersion,
  installPendingUpdate,
  type AvailableUpdate,
  type UpdateProgress
} from "../../lib/updater";
import "./UpdateControl.css";

type UpdatePhase =
  | "idle"
  | "checking"
  | "available"
  | "downloading"
  | "up_to_date"
  | "error";

interface UpdateControlProps {
  automaticCheck?: boolean;
  variant?: "compact" | "settings";
}

export function UpdateControl({
  automaticCheck = true,
  variant = "compact"
}: UpdateControlProps) {
  const [phase, setPhase] = useState<UpdatePhase>("idle");
  const [currentVersion, setCurrentVersion] = useState("1.0.0");
  const [available, setAvailable] = useState<AvailableUpdate | null>(null);
  const [progress, setProgress] = useState<UpdateProgress | null>(null);
  const [message, setMessage] = useState<string | null>(null);

  useEffect(() => {
    currentAppVersion().then(setCurrentVersion).catch(() => undefined);

    if (!automaticCheck) return undefined;
    const timer = window.setTimeout(() => {
      void runCheck(false);
    }, 3_000);

    return () => window.clearTimeout(timer);
  }, [automaticCheck]);

  async function runCheck(manual: boolean) {
    if (phase === "checking" || phase === "downloading") return;

    setPhase("checking");
    setMessage(null);

    try {
      const update = await checkForAppUpdate();
      setAvailable(update);

      if (update) {
        setPhase("available");
      } else {
        setPhase(manual ? "up_to_date" : "idle");
        if (manual) setMessage(`WinReclaim ${currentVersion} is current.`);
      }
    } catch (error) {
      if (manual) {
        setPhase("error");
        setMessage(cleanError(error));
      } else {
        setPhase("idle");
      }
    }
  }

  async function installUpdate() {
    if (!available || phase === "downloading") return;

    setPhase("downloading");
    setMessage("Downloading the signed update…");
    setProgress({ downloadedBytes: 0, totalBytes: null, percent: null });

    try {
      await installPendingUpdate((nextProgress) => {
        setProgress(nextProgress);
      });
    } catch (error) {
      setPhase("error");
      setMessage(cleanError(error));
    }
  }

  const buttonLabel = useMemo(() => {
    if (phase === "checking") return "Checking…";
    if (phase === "downloading") {
      return progress?.percent == null ? "Updating…" : `Updating ${progress.percent}%`;
    }
    if (phase === "available" && available) return `Update v${available.version}`;
    return variant === "settings" ? `Check for updates · v${currentVersion}` : `v${currentVersion}`;
  }, [available, currentVersion, phase, progress, variant]);

  const expanded =
    phase === "available" ||
    phase === "downloading" ||
    phase === "error" ||
    phase === "up_to_date";

  return (
    <div className={`update-control update-control-${variant} ${expanded ? "is-expanded" : ""}`}>
      <button
        className={`update-trigger ${phase === "available" ? "has-update" : ""}`}
        type="button"
        disabled={phase === "checking" || phase === "downloading"}
        onClick={() => {
          if (phase === "available") void installUpdate();
          else void runCheck(true);
        }}
        aria-label={
          phase === "available"
            ? `Install WinReclaim ${available?.version}`
            : "Check for WinReclaim updates"
        }
      >
        <span className="update-dot" aria-hidden="true" />
        {buttonLabel}
      </button>

      {expanded && (
        <div className="update-popover" role="status">
          {available && phase !== "error" ? (
            <>
              <div className="update-popover-head">
                <div>
                  <span>Signed release available</span>
                  <strong>WinReclaim {available.version}</strong>
                </div>
                {phase === "available" && (
                  <button type="button" onClick={() => void installUpdate()}>
                    Install and restart
                  </button>
                )}
              </div>
              {available.notes && <p>{available.notes}</p>}
              {phase === "downloading" && (
                <div className="update-progress" aria-label="Update download progress">
                  <span style={{ width: `${progress?.percent ?? 8}%` }} />
                </div>
              )}
              <small>The package is verified against WinReclaim&apos;s embedded updater key before installation.</small>
            </>
          ) : (
            <>
              <strong>{phase === "error" ? "Update check failed" : "You&apos;re up to date"}</strong>
              {message && <p>{message}</p>}
              <button className="update-dismiss" type="button" onClick={() => setPhase("idle")}>
                Dismiss
              </button>
            </>
          )}
        </div>
      )}
    </div>
  );
}

function cleanError(error: unknown): string {
  const text = String(error);
  return text.replace(/^Error:\s*/i, "").slice(0, 240);
}
