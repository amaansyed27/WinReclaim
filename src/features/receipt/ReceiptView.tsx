import { ReceiptIcon, ShieldIcon } from "../../components/Icons";
import { formatBytes, formatDate } from "../../lib/format";
import type { CleanupReceipt } from "../../types";

interface ReceiptViewProps {
  receipt: CleanupReceipt;
  onNewScan: () => void;
  onOpenVault: () => void;
}

export function ReceiptView({ receipt, onNewScan, onOpenVault }: ReceiptViewProps) {
  const successful = receipt.results.filter((result) => result.success).length;
  const failed = receipt.results.filter((result) => !result.success).length;
  const skipped = receipt.results.reduce((sum, result) => sum + result.skippedEntries, 0);
  const reversible = receipt.results
    .filter((result) => result.recoveryClass === "reversible")
    .reduce((sum, result) => sum + Math.max(0, result.measuredBytesBefore - result.measuredBytesAfter), 0);

  async function copyJson() {
    await navigator.clipboard.writeText(JSON.stringify(receipt, null, 2));
  }

  function downloadShareCard() {
    const canvas = document.createElement("canvas");
    canvas.width = 1200;
    canvas.height = 630;
    const context = canvas.getContext("2d");
    if (!context) return;
    context.fillStyle = "#07111f";
    context.fillRect(0, 0, canvas.width, canvas.height);
    const gradient = context.createRadialGradient(1020, 0, 40, 1020, 0, 520);
    gradient.addColorStop(0, "rgba(62, 143, 255, 0.38)");
    gradient.addColorStop(1, "rgba(7, 17, 31, 0)");
    context.fillStyle = gradient;
    context.fillRect(0, 0, canvas.width, canvas.height);
    context.fillStyle = "#eaf3ff";
    context.font = "650 38px Segoe UI";
    context.fillText("WinReclaim", 72, 90);
    context.font = "700 88px Segoe UI";
    context.fillText(formatBytes(receipt.actualReclaimedBytes), 72, 238);
    context.fillStyle = "#8ea4bf";
    context.font = "500 32px Segoe UI";
    context.fillText("freed from this Windows account", 76, 288);
    context.fillStyle = "#65b4ff";
    context.font = "600 26px Segoe UI";
    context.fillText(`${successful} completed · ${skipped} skipped`, 76, 386);
    context.fillStyle = "#8ea4bf";
    context.font = "500 24px Segoe UI";
    context.fillText(`${formatBytes(reversible)} can be restored`, 76, 438);
    context.fillText("Local scan · reviewed cleanup · checked result", 76, 526);
    const link = document.createElement("a");
    link.download = `winreclaim-${receipt.id.slice(0, 8)}.png`;
    link.href = canvas.toDataURL("image/png");
    link.click();
  }

  return (
    <section className="page receipt-view simple-receipt-view">
      <header className="page-header simple-page-header">
        <div>
          <span className="page-kicker">Cleanup complete</span>
          <h1>You freed {formatBytes(receipt.actualReclaimedBytes)}</h1>
          <p>{failed ? `${failed} item${failed === 1 ? "" : "s"} could not be completed.` : "Everything selected was processed."}</p>
        </div>
      </header>

      <section className="surface simple-complete-card">
        <span className="receipt-summary-icon"><ReceiptIcon /></span>
        <span className="simple-result-check" aria-hidden="true">✓</span>
        <strong>{formatBytes(receipt.actualReclaimedBytes)}</strong>
        <span>space freed</span>
        <small>{successful} item{successful === 1 ? "" : "s"} cleaned{skipped ? ` · ${skipped} locked or active files skipped` : ""}</small>
      </section>

      {receipt.vaultEntryIds.length > 0 && (
        <button className="vault-callout simple-restore-callout" onClick={onOpenVault}>
          <div>
            <strong>Changed your mind?</strong>
            <span>{formatBytes(reversible)} can be restored for seven days.</span>
          </div>
          <span>Restore files →</span>
        </button>
      )}

      <section className="surface simple-results-list">
        <header>
          <strong>What happened</strong>
          <span>{receipt.results.length} item{receipt.results.length === 1 ? "" : "s"}</span>
        </header>
        <div className="receipt-lines">
          {receipt.results.map((result) => (
            <div className="receipt-line simple-receipt-line" key={result.findingId}>
              <div>
                <strong>{result.displayName}</strong>
                <span>{result.message}</span>
              </div>
              <span className={result.success ? "status-success" : "status-failed"}>
                {result.success ? "Done" : "Not completed"}
              </span>
            </div>
          ))}
        </div>
      </section>

      <section className="simple-protection-note">
        <ShieldIcon />
        <div>
          <strong>Important files stayed untouched</strong>
          <span>WinReclaim only processed the items you confirmed.</span>
        </div>
      </section>

      <details className="surface cleanup-technical-details">
        <summary>Technical details</summary>
        <div className="cleanup-technical-content">
          <p>Completed {formatDate(receipt.completedAt)}</p>
          <p>Expected: {formatBytes(receipt.estimatedReclaimBytes)}</p>
          <p>Result ID: {receipt.id.slice(0, 8)}</p>
          <code>{receipt.planHash.slice(0, 20)}…</code>
          {receipt.protectedSummary.length > 0 && <p>{receipt.protectedSummary.join(" · ")}</p>}
          <button className="button button-secondary" onClick={copyJson}>Copy technical details</button>
        </div>
      </details>

      <footer className="page-action-row simple-complete-actions">
        <button className="button button-primary" onClick={onNewScan}>Back to cleanup</button>
        <button className="button button-secondary" onClick={downloadShareCard}>Save summary image</button>
      </footer>
    </section>
  );
}
