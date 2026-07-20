import { formatBytes, formatDate } from "../../lib/format";
import type { SnapshotSummary, StorageTimeline, TimelineDelta } from "../../types";

interface TimelineViewProps {
  timeline: StorageTimeline | null;
  loading: boolean;
  onRefresh: () => void;
  onScan: () => void;
}

export function TimelineView({ timeline, loading, onRefresh, onScan }: TimelineViewProps) {
  const snapshots = timeline?.snapshots ?? [];
  const growth = timeline?.totalGrowthBytes ?? null;
  const hasBaseline = Boolean(timeline?.baselineAvailable && growth !== null);
  const positive = (growth ?? 0) >= 0;
  const changes = timeline?.deltas ?? [];
  const comparedWith = timeline?.comparedWithAt ? formatDate(timeline.comparedWithAt) : null;

  return (
    <section className="page timeline-view simple-timeline-view">
      <header className="page-header simple-page-header">
        <div>
          <span className="page-kicker">History</span>
          <h1>See what changed</h1>
          <p>WinReclaim compares scans made with the same depth and options so the results stay meaningful.</p>
        </div>
        <button className="button button-secondary" onClick={onRefresh} disabled={loading}>
          {loading ? "Refreshing…" : "Refresh"}
        </button>
      </header>

      {!snapshots.length ? (
        <section className="surface timeline-empty simple-empty-card">
          <strong>No history yet</strong>
          <span>Run a scan now and another scan later using the same scan profile.</span>
          <button className="button button-primary" onClick={onScan}>Start a scan</button>
        </section>
      ) : (
        <>
          <div className="simple-history-summary">
            <section className={`surface simple-history-metric ${hasBaseline ? (positive ? "is-growth" : "is-smaller") : ""}`}>
              <span>Storage change</span>
              <strong>
                {hasBaseline && growth !== null
                  ? `${positive ? "+" : "−"}${formatBytes(Math.abs(growth))}`
                  : "Not enough data"}
              </strong>
              <small>
                {hasBaseline
                  ? `Compared with ${comparedWith}`
                  : "Run another scan with the same depth and options"}
              </small>
            </section>
            <section className="surface simple-history-metric">
              <span>Most recent scan</span>
              <strong>{formatDate(snapshots[snapshots.length - 1].capturedAt)}</strong>
              <small>{formatBytes(snapshots[snapshots.length - 1].freeBytes)} free</small>
            </section>
          </div>

          <section className="surface timeline-chart-card simple-chart-card">
            <header>
              <div>
                <strong>Disk usage over time</strong>
                <span>{snapshots.length} saved scan{snapshots.length === 1 ? "" : "s"}</span>
              </div>
              <span>{formatBytes(snapshots[snapshots.length - 1].usedBytes)} used</span>
            </header>
            <StorageLineChart snapshots={snapshots} />
          </section>

          <section className="surface simple-change-list">
            <header>
              <div>
                <strong>Biggest measured changes</strong>
                <span>{hasBaseline ? `Compared with ${comparedWith}` : "A matching earlier scan is required"}</span>
              </div>
              <span>{changes.length}</span>
            </header>
            {!hasBaseline ? (
              <div className="timeline-baseline-note">
                Existing scans used different depths or options. Run the same scan profile again to create a valid comparison.
              </div>
            ) : !changes.length ? (
              <div className="timeline-baseline-note">No measured changes were found in the scanned locations.</div>
            ) : (
              <>
                <div className="timeline-delta-list">
                  {changes.slice(0, 10).map((delta) => (
                    <DeltaRow delta={delta} key={delta.key} />
                  ))}
                </div>
                {changes.length > 10 && (
                  <details className="more-history-items">
                    <summary>Show {changes.length - 10} more changes</summary>
                    <div className="timeline-delta-list">
                      {changes.slice(10, 30).map((delta) => (
                        <DeltaRow delta={delta} key={delta.key} />
                      ))}
                    </div>
                  </details>
                )}
              </>
            )}
          </section>
        </>
      )}
    </section>
  );
}

function DeltaRow({ delta }: { delta: TimelineDelta }) {
  const positive = delta.deltaBytes > 0;
  return (
    <article className="timeline-delta-row simple-delta-row">
      <div className={`timeline-delta-mark ${positive ? "is-growth" : "is-reclaimed"}`} />
      <div className="timeline-delta-main">
        <div>
          <strong>{delta.displayName}</strong>
          <span>{delta.category}{delta.actionAvailable ? " · cleanup available" : ""}</span>
        </div>
        <details className="finding-details">
          <summary>More details</summary>
          <div className="finding-details-content">
            <code>{delta.path}</code>
            <p>Previous scan: {formatBytes(delta.previousBytes)} · Current scan: {formatBytes(delta.currentBytes)}</p>
          </div>
        </details>
      </div>
      <div className={`timeline-delta-value ${positive ? "is-growth" : "is-reclaimed"}`}>
        <strong>{positive ? "+" : "−"}{formatBytes(Math.abs(delta.deltaBytes))}</strong>
        <span>{formatBytes(delta.currentBytes)} now</span>
      </div>
    </article>
  );
}

function StorageLineChart({ snapshots }: { snapshots: SnapshotSummary[] }) {
  const width = 900;
  const height = 220;
  const paddingX = 36;
  const paddingY = 24;
  const values = snapshots.map((snapshot) => snapshot.usedBytes);
  const minimum = Math.min(...values);
  const maximum = Math.max(...values);
  const spread = Math.max(1, maximum - minimum);
  const points = snapshots.map((snapshot, index) => {
    const x = snapshots.length === 1
      ? width / 2
      : paddingX + (index / (snapshots.length - 1)) * (width - paddingX * 2);
    const y = height - paddingY - ((snapshot.usedBytes - minimum) / spread) * (height - paddingY * 2);
    return { x, y, snapshot };
  });
  const polyline = points.map((point) => `${point.x},${point.y}`).join(" ");

  return (
    <div className="timeline-chart-wrap">
      <svg className="timeline-chart" viewBox={`0 0 ${width} ${height}`} role="img" aria-label="Storage usage over time">
        <defs>
          <linearGradient id="timelineFill" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0" stopColor="rgba(75, 166, 255, .32)" />
            <stop offset="1" stopColor="rgba(75, 166, 255, 0)" />
          </linearGradient>
        </defs>
        {[0.25, 0.5, 0.75].map((fraction) => (
          <line
            key={fraction}
            x1={paddingX}
            x2={width - paddingX}
            y1={paddingY + fraction * (height - paddingY * 2)}
            y2={paddingY + fraction * (height - paddingY * 2)}
            className="timeline-grid-line"
          />
        ))}
        {points.length > 1 && (
          <polygon
            points={`${paddingX},${height - paddingY} ${polyline} ${width - paddingX},${height - paddingY}`}
            fill="url(#timelineFill)"
          />
        )}
        <polyline points={polyline} className="timeline-line" />
        {points.map((point) => (
          <g key={point.snapshot.id}>
            <circle cx={point.x} cy={point.y} r="4" className="timeline-point" />
            <title>{`${formatDate(point.snapshot.capturedAt)} · ${formatBytes(point.snapshot.usedBytes)}`}</title>
          </g>
        ))}
      </svg>
      <div className="timeline-chart-labels">
        <span>{formatDate(snapshots[0].capturedAt)}</span>
        <span>{formatDate(snapshots[snapshots.length - 1].capturedAt)}</span>
      </div>
    </div>
  );
}
