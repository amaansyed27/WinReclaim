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
  active: "Can be restored",
  restored: "Restored",
  partially_restored: "Some files restored",
  expired: "No longer available"
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
          <span className="page-kicker">Restore files</span>
          <h1>Undo a cleanup</h1>
          <p>Some cleaned files are kept on this PC for seven days so you can put them back.</p>
        </div>
        <button className="button button-secondary" onClick={onRefresh} disabled={loading}>
          {loading ? "Refreshing…" : "Refresh"}
        </button>
      </header>

      <div className="vault-metrics">
        <section className="surface vault-metric">
          <span>Files you can restore</span>
          <strong>{formatBytes(reversibleBytes)}</strong>
          <small>{active.length} cleanup groups available</small>
        </section>
        <section className="surface vault-metric">
          <span>Kept for</span>
          <strong>7 days</strong>
          <small>stored only on this PC</small>
        </section>
        <section className="surface vault-metric">
          <span>Existing files</span>
          <strong>Never replaced</strong>
          <small>files already in their original place are skipped</small>
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
            <span className="surface-label">Saved cleanup groups</span>
            <strong>Files available to restore</strong>
          </div>
          <span>{entries.length} groups</span>
        </header>

        {!entries.length ? (
          <div className="vault-empty">
            <strong>Nothing to restore</strong>
            <span>Eligible temporary files and crash reports will appear here after cleanup.</span>
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
                    Cleaned {formatDate(entry.createdAt)} · Available until {formatDate(entry.expiresAt)} · {entry.relativePaths.length} files
                  </p>
                </div>
                <div className="vault-entry-size">
                  <strong>{formatBytes(entry.storedBytes)}</strong>
                  <span>Result {entry.receiptId.slice(0, 8)}</span>
                </div>
                <button
                  className="button button-primary"
                  disabled={entry.status !== "active" || restoringId !== null}
                  onClick={() => onRestore(entry.id)}
                >
                  {restoringId === entry.id ? "Restoring…" : "Restore files"}
                </button>
              </article>
            ))}
          </div>
        )}
      </section>
    </section>
  );
}
