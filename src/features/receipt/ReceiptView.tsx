import { ReceiptIcon, ShieldIcon } from "../../components/Icons";
import { formatBytes, formatDate } from "../../lib/format";
import type { CleanupReceipt } from "../../types";

interface ReceiptViewProps {
  receipt: CleanupReceipt;
  onNewScan: () => void;
}

export function ReceiptView({ receipt, onNewScan }: ReceiptViewProps) {
  const successful = receipt.results.filter((result) => result.success).length;
  const skipped = receipt.results.reduce((sum, result) => sum + result.skippedEntries, 0);

  async function copyJson() {
    await navigator.clipboard.writeText(JSON.stringify(receipt, null, 2));
  }

  function downloadShareCard() {
    const canvas = document.createElement("canvas");
    canvas.width = 1200;
    canvas.height = 630;
    const context = canvas.getContext("2d");
    if (!context) return;
    context.fillStyle = "#f3efe4";
    context.fillRect(0, 0, canvas.width, canvas.height);
    context.fillStyle = "#18201b";
    context.font = "700 40px system-ui";
    context.fillText("WinReclaim", 72, 90);
    context.font = "700 92px system-ui";
    context.fillText(formatBytes(receipt.actualReclaimedBytes), 72, 242);
    context.font = "500 34px system-ui";
    context.fillText("reclaimed from this Windows profile", 76, 292);
    context.fillStyle = "#2f6f4e";
    context.font = "600 28px system-ui";
    context.fillText(`${successful} actions completed · ${skipped} locked entries skipped`, 76, 388);
    context.fillStyle = "#4d554e";
    context.font = "500 25px system-ui";
    context.fillText("0 protected categories touched", 76, 440);
    context.fillText("Local scan · deterministic cleanup · verified receipt", 76, 528);
    const link = document.createElement("a");
    link.download = `winreclaim-${receipt.id.slice(0, 8)}.png`;
    link.href = canvas.toDataURL("image/png");
    link.click();
  }

  return (
    <section className="view receipt-view">
      <div className="receipt-hero">
        <div className="receipt-icon-wrap"><ReceiptIcon /></div>
        <p className="eyebrow">Cleanup receipt</p>
        <h1>{formatBytes(receipt.actualReclaimedBytes)} reclaimed.</h1>
        <p>{successful} actions completed. {skipped} locked or active entries skipped. The result was measured against actual free space.</p>
      </div>

      <div className="receipt-paper">
        <header>
          <div><strong>WinReclaim receipt</strong><span>{formatDate(receipt.completedAt)}</span></div>
          <code>{receipt.planHash.slice(0, 20)}…</code>
        </header>
        <div className="receipt-totals">
          <div><span>Estimated</span><strong>{formatBytes(receipt.estimatedReclaimBytes)}</strong></div>
          <div><span>Measured</span><strong>{formatBytes(receipt.actualReclaimedBytes)}</strong></div>
          <div><span>Free after</span><strong>{formatBytes(receipt.diskFreeAfter)}</strong></div>
        </div>
        <div className="receipt-lines">
          {receipt.results.map((result) => (
            <div className="receipt-line" key={result.findingId}>
              <div><strong>{result.displayName}</strong><span>{result.message}</span></div>
              <span className={result.success ? "status-success" : "status-failed"}>{result.success ? "Completed" : "Failed"}</span>
            </div>
          ))}
        </div>
        <div className="receipt-protected">
          <ShieldIcon />
          <div><strong>Protected categories untouched</strong><p>{receipt.protectedSummary.join(" · ")}</p></div>
        </div>
      </div>

      <div className="receipt-actions">
        <button className="button button-primary" onClick={downloadShareCard}>Export share card</button>
        <button className="button button-secondary" onClick={copyJson}>Copy JSON receipt</button>
        <button className="button button-quiet" onClick={onNewScan}>Start a new scan</button>
      </div>
    </section>
  );
}
