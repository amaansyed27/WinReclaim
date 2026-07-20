import { ArrowIcon, ShieldIcon } from "../../components/Icons";
import { formatBytes } from "../../lib/format";
import type { AiStatus, Finding, ReclaimPassport, ScanReport } from "../../types";
import { FindingRow } from "./FindingRow";
import { IntentPlanner } from "./IntentPlanner";

interface FindingsViewProps {
  report: ScanReport;
  passports: Map<string, ReclaimPassport>;
  selectedIds: Set<string>;
  aiStatus: AiStatus | null;
  intentLoading: boolean;
  intentSummary: string | null;
  onInterpretIntent: (prompt: string) => Promise<void>;
  onToggle: (id: string) => void;
  onSelectRecommended: () => void;
  onClearSelection: () => void;
  onBack: () => void;
  onCreatePlan: () => void;
}

export function FindingsView({
  report,
  passports,
  selectedIds,
  aiStatus,
  intentLoading,
  intentSummary,
  onInterpretIntent,
  onToggle,
  onSelectRecommended,
  onClearSelection,
  onBack,
  onCreatePlan
}: FindingsViewProps) {
  const recommended = report.findings
    .filter((finding) => finding.actionAvailable && finding.riskClass === "safe_now")
    .sort((a, b) => b.estimatedBytes - a.estimatedBytes);
  const optional = report.findings
    .filter((finding) => finding.actionAvailable && finding.riskClass !== "safe_now")
    .sort((a, b) => b.estimatedBytes - a.estimatedBytes);
  const reviewOnly = report.findings
    .filter((finding) => !finding.actionAvailable)
    .sort((a, b) => b.estimatedBytes - a.estimatedBytes);

  const selectedBytes = report.findings
    .filter((finding) => selectedIds.has(finding.id))
    .reduce((total, finding) => total + finding.estimatedBytes, 0);
  const recommendedBytes = recommended.reduce((sum, finding) => sum + finding.estimatedBytes, 0);
  const optionalBytes = optional.reduce((sum, finding) => sum + finding.estimatedBytes, 0);
  const reviewOnlyBytes = reviewOnly.reduce((sum, finding) => sum + finding.estimatedBytes, 0);

  return (
    <section className="page findings-view simple-findings-view">
      <header className="page-header simple-page-header">
        <div>
          <span className="page-kicker">Step 2 of 3</span>
          <h1>Choose what to clean</h1>
          <p>The safest items are selected for you. You can change the selection before anything is removed.</p>
        </div>
      </header>

      <section className="surface cleanup-recommendation">
        <div>
          <span className="recommendation-check" aria-hidden="true">✓</span>
          <div>
            <span className="surface-label">Recommended cleanup</span>
            <h2>{formatBytes(recommendedBytes)} is safe to clean</h2>
            <p>These are temporary files or crash reports that WinReclaim can restore for seven days when supported.</p>
          </div>
        </div>
        <div className="recommendation-actions">
          <button className="button button-primary" onClick={onSelectRecommended}>Use recommendation</button>
          <button className="button button-secondary" onClick={onClearSelection} disabled={!selectedIds.size}>Clear selection</button>
        </div>
      </section>

      {aiStatus?.configured && (
        <IntentPlanner
          status={aiStatus}
          loading={intentLoading}
          summary={intentSummary}
          selectedBytes={selectedBytes}
          onInterpret={onInterpretIntent}
        />
      )}

      <FindingSection
        title="Recommended"
        note="Safe choices for most people."
        items={recommended}
        bytes={recommendedBytes}
        passports={passports}
        selectedIds={selectedIds}
        onToggle={onToggle}
      />

      {optional.length > 0 && (
        <FindingSection
          title="Optional cleanup"
          note="These caches can be downloaded or recreated later. Cleaning them may make the next app or developer-tool run slower."
          items={optional}
          bytes={optionalBytes}
          passports={passports}
          selectedIds={selectedIds}
          onToggle={onToggle}
        />
      )}

      {reviewOnly.length > 0 && (
        <details className="surface review-only-folders">
          <summary>
            <div>
              <ShieldIcon />
              <span>
                <strong>{reviewOnly.length} large folders WinReclaim will not clean</strong>
                <small>They use {formatBytes(reviewOnlyBytes)}. Open this only when you want to inspect them yourself.</small>
              </span>
            </div>
            <span>Show folders</span>
          </summary>
          <div className="review-only-content">
            {reviewOnly.map((finding) => (
              <FindingRow
                finding={finding}
                passport={passports.get(finding.id)}
                key={finding.id}
                selected={false}
                onToggle={onToggle}
              />
            ))}
          </div>
        </details>
      )}

      <footer className="sticky-action-bar simple-sticky-action-bar">
        <button className="button button-secondary" onClick={onBack}>Back</button>
        <div className="selection-summary">
          <span>{selectedIds.size ? `${selectedIds.size} selected` : "Nothing selected"}</span>
          <strong>{formatBytes(selectedBytes)}</strong>
        </div>
        <button
          className="button button-primary"
          onClick={onCreatePlan}
          disabled={!selectedIds.size}
        >
          Continue <ArrowIcon />
        </button>
      </footer>
    </section>
  );
}

function FindingSection({
  title,
  note,
  items,
  bytes,
  passports,
  selectedIds,
  onToggle
}: {
  title: string;
  note: string;
  items: Finding[];
  bytes: number;
  passports: Map<string, ReclaimPassport>;
  selectedIds: Set<string>;
  onToggle: (id: string) => void;
}) {
  if (!items.length) return null;

  return (
    <section className="finding-group simple-finding-group">
      <div className="finding-group-head">
        <div>
          <h2>{title}</h2>
          <p>{note}</p>
        </div>
        <div className="group-summary">
          <span>{items.length} item{items.length === 1 ? "" : "s"}</span>
          <strong>{formatBytes(bytes)}</strong>
        </div>
      </div>
      <div className="finding-list">
        {items.map((finding) => (
          <FindingRow
            finding={finding}
            passport={passports.get(finding.id)}
            key={finding.id}
            selected={selectedIds.has(finding.id)}
            onToggle={onToggle}
          />
        ))}
      </div>
    </section>
  );
}
