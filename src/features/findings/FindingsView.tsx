import { ArrowIcon } from "../../components/Icons";
import { formatBytes } from "../../lib/format";
import type { AiStatus, Finding, RiskClass, ScanReport } from "../../types";
import { FindingRow } from "./FindingRow";
import { IntentPlanner } from "./IntentPlanner";

const groups: { id: RiskClass; title: string; note: string }[] = [
  { id: "safe_now", title: "Safe now", note: "Disposable data with narrow cleanup rules." },
  { id: "rebuild_or_redownload", title: "Rebuild later", note: "The owning tool may fetch or rebuild this data." },
  { id: "review_first", title: "Review first", note: "Environments, containers or project output." },
  { id: "protected", title: "Protected", note: "Inspection only. Automatic cleanup is disabled." }
];

interface FindingsViewProps {
  report: ScanReport;
  selectedIds: Set<string>;
  aiStatus: AiStatus | null;
  intentLoading: boolean;
  intentSummary: string | null;
  onInterpretIntent: (prompt: string) => Promise<void>;
  onToggle: (id: string) => void;
  onBack: () => void;
  onCreatePlan: () => void;
}

export function FindingsView({
  report,
  selectedIds,
  aiStatus,
  intentLoading,
  intentSummary,
  onInterpretIntent,
  onToggle,
  onBack,
  onCreatePlan
}: FindingsViewProps) {
  const selectedBytes = report.findings
    .filter((finding) => selectedIds.has(finding.id))
    .reduce((total, finding) => total + finding.estimatedBytes, 0);
  const totalBytes = report.findings.reduce((sum, item) => sum + item.estimatedBytes, 0);
  const actionable = report.findings.filter((finding) => finding.actionAvailable).length;

  return (
    <section className="page findings-view">
      <header className="page-header">
        <div>
          <span className="page-kicker">Scan results</span>
          <h1>Findings</h1>
          <p>Select only the storage you are prepared to rebuild or redownload.</p>
        </div>
        <div className="header-metrics">
          <div><span>Classified</span><strong>{formatBytes(totalBytes)}</strong></div>
          <div><span>Actionable</span><strong>{actionable}</strong></div>
        </div>
      </header>

      <IntentPlanner
        status={aiStatus}
        loading={intentLoading}
        summary={intentSummary}
        selectedBytes={selectedBytes}
        onInterpret={onInterpretIntent}
      />

      <div className="finding-groups">
        {groups.map((group) => {
          const items = report.findings
            .filter((finding) => finding.riskClass === group.id)
            .sort((a, b) => b.estimatedBytes - a.estimatedBytes);
          if (!items.length) return null;

          return (
            <section className={`finding-group group-${group.id}`} key={group.id}>
              <div className="finding-group-head">
                <div>
                  <h2>{group.title}</h2>
                  <p>{group.note}</p>
                </div>
                <div className="group-summary">
                  <span>{items.length} items</span>
                  <strong>{formatBytes(items.reduce((sum, item) => sum + item.estimatedBytes, 0))}</strong>
                </div>
              </div>
              <div className="finding-list">
                {items.map((finding: Finding) => (
                  <FindingRow
                    finding={finding}
                    key={finding.id}
                    selected={selectedIds.has(finding.id)}
                    onToggle={onToggle}
                  />
                ))}
              </div>
            </section>
          );
        })}
      </div>

      <footer className="sticky-action-bar">
        <button className="button button-secondary" onClick={onBack}>Back to scan</button>
        <div className="selection-summary">
          <span>{selectedIds.size} selected</span>
          <strong>{formatBytes(selectedBytes)}</strong>
        </div>
        <button
          className="button button-primary"
          onClick={onCreatePlan}
          disabled={!selectedIds.size}
        >
          Review plan <ArrowIcon />
        </button>
      </footer>
    </section>
  );
}
