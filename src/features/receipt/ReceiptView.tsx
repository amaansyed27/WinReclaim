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
    context.fillText("reclaimed from this Windows profile", 76, 288);
    context.fillStyle = "#65b4ff";
    context.font = "600 26px Segoe UI";
    context.fillText(`${successful} completed · ${skipped} skipped`, 76, 386);
    context.fillStyle = "#8ea4bf";
    context.font = "500 24px Segoe UI";
    context.fillText(`${formatBytes(reversible)} available in Undo Vault`, 76, 438);
    context.fillText("Local scan · simulated plan · verified receipt", 76, 526);
    const link = document.createElement("a");
    link.download = `winreclaim-${receipt.id.slice(0, 8)}.png`;
    link.href = canvas.toDataURL("image/png");
    link.click();
  }

  return (
    <section className="page receipt-view">
      <header className="page-header">
        <div>
          <span className="page-kicker">Completed</span>
          <h1>Cleanup receipt</h1>
          <p>{formatDate(receipt.completedAt)}</p>
        </div>
        <span className="status-chip is-ready">Verified</span>
      </header>

      <div className="receipt-summary-grid">
        <section className="surface receipt-primary-summary">
          <span className="receipt-summary-icon"><ReceiptIcon /></span>
          <span>Measured reclaim</span>
          <strong>{formatBytes(receipt.actualReclaimedBytes)}</strong>
          <small>{successful} actions completed · {skipped} entries skipped</small>
        </section>
        <section className="surface receipt-metric"><span>Estimated</span><strong>{formatBytes(receipt.estimatedReclaimBytes)}</strong></section>
        <section className="surface receipt-metric"><span>Undo Vault</span><strong>{formatBytes(reversible)}</strong></section>
      </div>

      {receipt.vaultEntryIds.length > 0 && (
        <button className="vault-callout" onClick={onOpenVault}>
          <div>
            <strong>{receipt.vaultEntryIds.length} reversible cleanup manifest{receipt.vaultEntryIds.length === 1 ? "" : "s"}</strong>
            <span>Restore eligible files for seven days without overwriting existing paths.</span>
          </div>
          <span>Open Undo Vault →</span>
        </button>
      )}

      <section className="surface receipt-detail">
        <header>
          <div>
            <strong>Execution results</strong>
            <span>Receipt {receipt.id.slice(0, 8)}</span>
          </div>
          <code>{receipt.planHash.slice(0, 20)}…</code>
        </header>

        <div className="receipt-lines">
          {receipt.results.map((result) => (
            <div className="receipt-line" key={result.findingId}>
              <div>
                <div className="receipt-result-title">
                  <strong>{result.displayName}</strong>
                  <i className={`recovery-badge recovery-${result.recoveryClass}`}>
                    {result.recoveryClass.replaceAll("_", " ")}
                  </i>
                </div>
                <span>{result.message}</span>
              </div>
              <span className={result.success ? "status-success" : "status-failed"}>
                {result.success ? "Completed" : "Failed"}
              </span>
            </div>
          ))}
        </div>

        <div className="receipt-protected">
          <ShieldIcon />
          <div>
            <strong>Protected categories untouched</strong>
            <p>{receipt.protectedSummary.join(" · ")}</p>
          </div>
        </div>
      </section>

      <footer className="page-action-row">
        <button className="button button-primary" onClick={downloadShareCard}>Export card</button>
        <button className="button button-secondary" onClick={copyJson}>Copy JSON</button>
        <button className="button button-quiet" onClick={onNewScan}>New scan</button>
      </footer>
    </section>
  );
}
