import { formatBytes, formatDate } from "../../lib/format";
import type { RestoreResult, VaultEntry } from "../../types";

interface VaultViewProps {
  entries: VaultEntry[];
  loading: boolean;
  restoringId: string | null;
  lastRestore: RestoreResult | null;
  error: string | null;
  onRefresh: () => void;
  onRestore: (id: string) => void;
}

const statusLabels: Record<VaultEntry["status"], string> = {
  active: "Undo available",
  restored: "Restored",
  partially_restored: "Partially restored",
  expired: "Expired"
};

export function VaultView({
  entries,
  loading,
  restoringId,
  lastRestore,
  error,
  onRefresh,
  onRestore
}: VaultViewProps) {
  const active = entries.filter((entry) => entry.status === "active");
  const reversibleBytes = active.reduce((sum, entry) => sum + entry.storedBytes, 0);

  return (
    <section className="page vault-view">
      <header className="page-header">
        <div>
          <span className="page-kicker">Safe Undo Vault</span>
          <h1>Undo cleanup</h1>
          <p>File-based cleanup is quarantined locally for seven days before expiry.</p>
        </div>
        <button className="button button-secondary" onClick={onRefresh} disabled={loading}>
          {loading ? "Refreshing…" : "Refresh"}
        </button>
      </header>

      <div className="vault-metrics">
        <section className="surface vault-metric">
          <span>Undo available</span>
          <strong>{formatBytes(reversibleBytes)}</strong>
          <small>{active.length} active vault entries</small>
        </section>
        <section className="surface vault-metric">
          <span>Retention</span>
          <strong>7 days</strong>
          <small>stored under LocalAppData</small>
        </section>
        <section className="surface vault-metric">
          <span>Conflict policy</span>
          <strong>Never overwrite</strong>
          <small>occupied original paths are skipped</small>
        </section>
      </div>

      {lastRestore && (
        <div className="vault-result-banner">
          <strong>{lastRestore.message}</strong>
          <span>{formatBytes(lastRestore.restoredBytes)} restored · {lastRestore.skippedEntries} skipped</span>
        </div>
      )}
      {error && <p className="error-banner">{error}</p>}

      <section className="surface vault-list-card">
        <header>
          <div>
            <span className="surface-label">Quarantine manifests</span>
            <strong>Reversible cleanup history</strong>
          </div>
          <span>{entries.length} entries</span>
        </header>

        {!entries.length ? (
          <div className="vault-empty">
            <strong>The Undo Vault is empty</strong>
            <span>Eligible temporary files and crash dumps will appear here after cleanup.</span>
          </div>
        ) : (
          <div className="vault-entry-list">
            {entries.map((entry) => (
              <article className={`vault-entry status-${entry.status}`} key={entry.id}>
                <div className="vault-entry-main">
                  <div className="vault-entry-title">
                    <strong>{entry.displayName}</strong>
                    <span>{statusLabels[entry.status]}</span>
                  </div>
                  <code>{entry.originalRoot}</code>
                  <p>
                    Created {formatDate(entry.createdAt)} · Expires {formatDate(entry.expiresAt)} · {entry.relativePaths.length} files
                  </p>
                </div>
                <div className="vault-entry-size">
                  <strong>{formatBytes(entry.storedBytes)}</strong>
                  <span>Receipt {entry.receiptId.slice(0, 8)}</span>
                </div>
                <button
                  className="button button-primary"
                  disabled={entry.status !== "active" || restoringId !== null}
                  onClick={() => onRestore(entry.id)}
                >
                  {restoringId === entry.id ? "Restoring…" : "Restore"}
                </button>
              </article>
            ))}
          </div>
        )}
      </section>
    </section>
  );
}
