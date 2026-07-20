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
  active: "Ready to restore",
  restored: "Already restored",
  partially_restored: "Partly restored",
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
  const nextExpiry = active
    .map((entry) => entry.expiresAt)
    .sort((left, right) => new Date(left).getTime() - new Date(right).getTime())[0];

  return (
    <section className="page vault-view simple-vault-view">
      <header className="page-header simple-page-header">
        <div>
          <span className="page-kicker">Restore files</span>
          <h1>Undo a recent cleanup</h1>
          <p>Each saved cleanup shows its actual expiration date and original location.</p>
        </div>
        <button className="button button-secondary" onClick={onRefresh} disabled={loading}>
          {loading ? "Refreshing…" : "Refresh"}
        </button>
      </header>

      <section className="surface simple-restore-summary">
        <div>
          <span className="surface-label">Available to restore</span>
          <strong>{formatBytes(reversibleBytes)}</strong>
          <p>{active.length} cleanup group{active.length === 1 ? "" : "s"} can still be restored.</p>
        </div>
        <span>{nextExpiry ? `Next expiry ${formatDate(nextExpiry)}` : "Nothing currently available"}</span>
      </section>

      {lastRestore && (
        <div className="vault-result-banner">
          <strong>{lastRestore.message}</strong>
          <span>{formatBytes(lastRestore.restoredBytes)} restored · {lastRestore.skippedEntries} skipped</span>
        </div>
      )}
      {error && <p className="error-banner">{error}</p>}

      <section className="surface simple-vault-list">
        <header>
          <strong>Recent cleanups</strong>
          <span>{entries.length}</span>
        </header>

        {!entries.length ? (
          <div className="vault-empty">
            <strong>Nothing to restore</strong>
            <span>Restorable files will appear here after a cleanup.</span>
          </div>
        ) : (
          <div className="vault-entry-list">
            {entries.map((entry) => (
              <article className={`vault-entry simple-vault-entry status-${entry.status}`} key={entry.id}>
                <div className="vault-entry-main">
                  <div className="vault-entry-title">
                    <strong>{entry.displayName}</strong>
                    <span>{statusLabels[entry.status]}</span>
                  </div>
                  <p>Cleaned {formatDate(entry.createdAt)} · {entry.relativePaths.length} file{entry.relativePaths.length === 1 ? "" : "s"}</p>
                  <details className="finding-details">
                    <summary>More details</summary>
                    <div className="finding-details-content">
                      <code>{entry.originalRoot}</code>
                      <p>Available until {formatDate(entry.expiresAt)}</p>
                    </div>
                  </details>
                </div>
                <div className="vault-entry-size">
                  <strong>{formatBytes(entry.storedBytes)}</strong>
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
