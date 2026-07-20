import { formatBytes, formatDate } from "../../lib/format";
import { recoveryLabel, riskCopy } from "../../lib/plainLanguage";
import type { Finding, ReclaimPassport } from "../../types";

interface FindingRowProps {
  finding: Finding;
  passport?: ReclaimPassport;
  selected: boolean;
  onToggle: (id: string) => void;
}

export function FindingRow({ finding, passport, selected, onToggle }: FindingRowProps) {
  return (
    <article className={`finding-row risk-${finding.riskClass}`}>
      <button
        type="button"
        className={`finding-select ${selected ? "is-selected" : ""}`}
        disabled={!finding.actionAvailable || finding.riskClass === "protected"}
        onClick={() => onToggle(finding.id)}
        aria-label={`${selected ? "Remove" : "Add"} ${finding.displayName} ${selected ? "from" : "to"} the cleanup selection`}
      >
        <span />
      </button>
      <div className="finding-main">
        <div className="finding-title-line">
          <h3>{finding.displayName}</h3>
          <span className={`risk-label risk-label-${finding.riskClass}`}>
            {riskCopy[finding.riskClass].title}
          </span>
        </div>
        <p>{finding.explanation}</p>
        <code>{finding.path}</code>
        <p className="consequence"><strong>What happens:</strong> {finding.consequence}</p>
        {passport && (
          <div className="passport-strip">
            <div><span>Made by</span><strong>{passport.owner}</strong></div>
            <div><span>After cleanup</span><strong>{recoveryLabel(passport.recoveryClass)}</strong></div>
            <div><span>How sure?</span><strong>{passport.confidenceScore}%</strong></div>
            <div>
              <span>Last changed</span>
              <strong>{passport.lastChangedAt ? formatDate(passport.lastChangedAt) : "Unknown"}</strong>
            </div>
            <p>{passport.activityNote}. {passport.recoveryMethod}</p>
          </div>
        )}
      </div>
      <div className="finding-size">
        <strong>{formatBytes(finding.estimatedBytes)}</strong>
        <span>{finding.actionAvailable ? "Can be cleaned" : "Information only"}</span>
      </div>
    </article>
  );
}
