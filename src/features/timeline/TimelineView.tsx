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
  const growth = timeline?.totalGrowthBytes ?? 0;
  const positive = growth >= 0;

  return (
    <section className="page timeline-view">
      <header className="page-header">
        <div>
          <span className="page-kicker">Storage Time Machine</span>
          <h1>What changed?</h1>
          <p>Compare local scan snapshots and trace where storage grew or disappeared.</p>
        </div>
        <button className="button button-secondary" onClick={onRefresh} disabled={loading}>
          {loading ? "Refreshing…" : "Refresh"}
        </button>
      </header>

      {!snapshots.length ? (
        <section className="surface timeline-empty">
          <strong>No storage history yet</strong>
          <span>Run a scan now, then scan again later to create a change baseline.</span>
          <button className="button button-primary" onClick={onScan}>Open scan</button>
        </section>
      ) : (
        <>
          <div className="timeline-metrics">
            <Metric
              label="Profile change"
              value={`${positive ? "+" : "−"}${formatBytes(Math.abs(growth))}`}
              note={timeline?.baselineAvailable ? "since the previous scan" : "baseline required"}
              tone={positive ? "warning" : "success"}
            />
            <Metric
              label="Reclaimable growth"
              value={formatBytes(timeline?.reclaimableGrowthBytes ?? 0)}
              note="growth backed by executable adapters"
              tone="accent"
            />
            <Metric
              label="Snapshots"
              value={String(snapshots.length)}
              note={`${formatDate(snapshots[snapshots.length - 1].capturedAt)} latest`}
            />
          </div>

          <section className="surface timeline-chart-card">
            <header>
              <div>
                <span className="surface-label">Profile usage</span>
                <strong>Storage growth across scans</strong>
              </div>
              <span>{formatBytes(snapshots[snapshots.length - 1].usedBytes)} used</span>
            </header>
            <StorageLineChart snapshots={snapshots} />
          </section>

          <section className="surface timeline-delta-card">
            <header>
              <div>
                <span className="surface-label">Attribution</span>
                <strong>Largest changes since the previous scan</strong>
              </div>
              <span>{timeline?.deltas.length ?? 0} changed locations</span>
            </header>
            {!timeline?.baselineAvailable ? (
              <div className="timeline-baseline-note">
                Run another scan later to compare this machine against the current baseline.
              </div>
            ) : !timeline.deltas.length ? (
              <div className="timeline-baseline-note">No classified storage changes were detected.</div>
            ) : (
              <div className="timeline-delta-list">
                {timeline.deltas.slice(0, 30).map((delta) => (
                  <DeltaRow delta={delta} key={delta.key} />
                ))}
              </div>
            )}
          </section>
        </>
      )}
    </section>
  );
}

function Metric({
  label,
  value,
  note,
  tone = "neutral"
}: {
  label: string;
  value: string;
  note: string;
  tone?: "neutral" | "accent" | "success" | "warning";
}) {
  return (
    <section className={`surface timeline-metric tone-${tone}`}>
      <span>{label}</span>
      <strong>{value}</strong>
      <small>{note}</small>
    </section>
  );
}

function DeltaRow({ delta }: { delta: TimelineDelta }) {
  const positive = delta.deltaBytes > 0;
  return (
    <article className="timeline-delta-row">
      <div className={`timeline-delta-mark ${positive ? "is-growth" : "is-reclaimed"}`} />
      <div className="timeline-delta-main">
        <div>
          <strong>{delta.displayName}</strong>
          <span>{delta.owner}</span>
        </div>
        <code>{delta.path}</code>
      </div>
      <div className="timeline-delta-evidence">
        <span>{delta.confidenceScore}% attribution</span>
        <small>{delta.recoveryClass.replaceAll("_", " ")}</small>
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
