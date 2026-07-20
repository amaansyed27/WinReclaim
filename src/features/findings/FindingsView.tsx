import { ArrowIcon } from "../../components/Icons";
import { formatBytes } from "../../lib/format";
import { riskCopy } from "../../lib/plainLanguage";
import type { AiStatus, Finding, ReclaimPassport, RiskClass, ScanReport } from "../../types";
import { FindingRow } from "./FindingRow";
import { IntentPlanner } from "./IntentPlanner";

const groupOrder: RiskClass[] = [
  "safe_now",
  "rebuild_or_redownload",
  "review_first",
  "protected"
];

interface FindingsViewProps {
  report: ScanReport;
  passports: Map<string, ReclaimPassport>;
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
  passports,
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
          <span className="page-kicker">What WinReclaim found</span>
          <h1>Choose what to clean</h1>
          <p>Each item explains what it is, what happens after removal and whether it can be restored.</p>
        </div>
        <div className="header-metrics">
          <div><span>Space found</span><strong>{formatBytes(totalBytes)}</strong></div>
          <div><span>Can clean</span><strong>{actionable}</strong></div>
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
        {groupOrder.map((groupId) => {
          const group = riskCopy[groupId];
          const items = report.findings
            .filter((finding) => finding.riskClass === groupId)
            .sort((a, b) => b.estimatedBytes - a.estimatedBytes);
          if (!items.length) return null;

          return (
            <section className={`finding-group group-${groupId}`} key={groupId}>
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
                    passport={passports.get(finding.id)}
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
          Review cleanup <ArrowIcon />
        </button>
      </footer>
    </section>
  );
}
