import { formatBytes, formatDate } from "../../lib/format";
import { recoveryLabel } from "../../lib/plainLanguage";
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
          <span className="page-kicker">Storage history</span>
          <h1>See what changed</h1>
          <p>Compare scans to see what used more space and what freed space.</p>
        </div>
        <button className="button button-secondary" onClick={onRefresh} disabled={loading}>
          {loading ? "Refreshing…" : "Refresh"}
        </button>
      </header>

      {!snapshots.length ? (
        <section className="surface timeline-empty">
          <strong>No storage history yet</strong>
          <span>Run a scan now and another one later. WinReclaim will compare them for you.</span>
          <button className="button button-primary" onClick={onScan}>Start a scan</button>
        </section>
      ) : (
        <>
          <div className="timeline-metrics">
            <Metric
              label="Disk change"
              value={`${positive ? "+" : "−"}${formatBytes(Math.abs(growth))}`}
              note={timeline?.baselineAvailable ? "since your previous scan" : "another scan is needed"}
              tone={positive ? "warning" : "success"}
            />
            <Metric
              label="Can be cleaned"
              value={formatBytes(timeline?.reclaimableGrowthBytes ?? 0)}
              note="space with a verified cleanup action"
              tone="accent"
            />
            <Metric
              label="Saved scans"
              value={String(snapshots.length)}
              note={`latest: ${formatDate(snapshots[snapshots.length - 1].capturedAt)}`}
            />
          </div>

          <section className="surface timeline-chart-card">
            <header>
              <div>
                <span className="surface-label">Used space</span>
                <strong>Disk usage across your scans</strong>
              </div>
              <span>{formatBytes(snapshots[snapshots.length - 1].usedBytes)} used</span>
            </header>
            <StorageLineChart snapshots={snapshots} />
          </section>

          <section className="surface timeline-delta-card">
            <header>
              <div>
                <span className="surface-label">Likely source</span>
                <strong>Biggest changes since your previous scan</strong>
              </div>
              <span>{timeline?.deltas.length ?? 0} changed locations</span>
            </header>
            {!timeline?.baselineAvailable ? (
              <div className="timeline-baseline-note">
                Run another scan later and WinReclaim will show what changed.
              </div>
            ) : !timeline.deltas.length ? (
              <div className="timeline-baseline-note">No noticeable storage changes were found.</div>
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
        <span>{delta.confidenceScore}% sure</span>
        <small>{recoveryLabel(delta.recoveryClass)}</small>
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
