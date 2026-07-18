import { ArrowIcon } from "../../components/Icons";
import { formatBytes } from "../../lib/format";
import type { Finding, RiskClass, ScanReport } from "../../types";
import { FindingRow } from "./FindingRow";

const groups: { id: RiskClass; title: string; note: string }[] = [
  { id: "safe_now", title: "Safe now", note: "Regenerable or disposable data with narrow cleanup rules." },
  { id: "rebuild_or_redownload", title: "Rebuild or redownload later", note: "Useful space, but the tool may need to fetch or rebuild it again." },
  { id: "review_first", title: "Review first", note: "Potentially important environments, containers or project output." },
  { id: "protected", title: "Protected", note: "WinReclaim will not include these in an automatic cleanup plan." }
];

interface FindingsViewProps {
  report: ScanReport;
  selectedIds: Set<string>;
  onToggle: (id: string) => void;
  onBack: () => void;
  onCreatePlan: () => void;
}

export function FindingsView({ report, selectedIds, onToggle, onBack, onCreatePlan }: FindingsViewProps) {
  const selectedBytes = report.findings
    .filter((finding) => selectedIds.has(finding.id))
    .reduce((total, finding) => total + finding.estimatedBytes, 0);

  return (
    <section className="view findings-view">
      <header className="view-header">
        <div>
          <p className="eyebrow">Storage story</p>
          <h1>Not all large folders mean the same thing.</h1>
          <p>Findings are grouped by consequence. Nothing is selected automatically.</p>
        </div>
        <div className="view-stat">
          <strong>{formatBytes(report.findings.reduce((sum, item) => sum + item.estimatedBytes, 0))}</strong>
          <span>classified storage</span>
        </div>
      </header>

      <div className="finding-groups">
        {groups.map((group) => {
          const items = report.findings
            .filter((finding) => finding.riskClass === group.id)
            .sort((a, b) => b.estimatedBytes - a.estimatedBytes);
          if (!items.length) return null;
          return (
            <section className="finding-group" key={group.id}>
              <div className="finding-group-head">
                <div><h2>{group.title}</h2><p>{group.note}</p></div>
                <strong>{formatBytes(items.reduce((sum, item) => sum + item.estimatedBytes, 0))}</strong>
              </div>
              <div className="finding-list">
                {items.map((finding: Finding) => (
                  <FindingRow finding={finding} key={finding.id} selected={selectedIds.has(finding.id)} onToggle={onToggle} />
                ))}
              </div>
            </section>
          );
        })}
      </div>

      <footer className="sticky-action-bar">
        <button className="button button-quiet" onClick={onBack}>Back</button>
        <div><span>{selectedIds.size} selected</span><strong>{formatBytes(selectedBytes)} estimated</strong></div>
        <button className="button button-primary" onClick={onCreatePlan} disabled={!selectedIds.size}>
          Build cleanup plan <ArrowIcon />
        </button>
      </footer>
    </section>
  );
}
