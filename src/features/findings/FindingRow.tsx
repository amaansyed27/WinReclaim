import { formatBytes, formatDate } from "../../lib/format";
import type { Finding, ReclaimPassport, RiskClass } from "../../types";

const riskLabels: Record<RiskClass, string> = {
  safe_now: "Safe now",
  rebuild_or_redownload: "Rebuild later",
  review_first: "Review first",
  protected: "Protected"
};

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
        aria-label={`${selected ? "Remove" : "Add"} ${finding.displayName} from plan`}
      >
        <span />
      </button>
      <div className="finding-main">
        <div className="finding-title-line">
          <h3>{finding.displayName}</h3>
          <span className={`risk-label risk-label-${finding.riskClass}`}>
            {riskLabels[finding.riskClass]}
          </span>
        </div>
        <p>{finding.explanation}</p>
        <code>{finding.path}</code>
        <p className="consequence">{finding.consequence}</p>
        {passport && (
          <div className="passport-strip">
            <div><span>Owner</span><strong>{passport.owner}</strong></div>
            <div><span>Recovery</span><strong>{passport.recoveryClass.replaceAll("_", " ")}</strong></div>
            <div><span>Confidence</span><strong>{passport.confidenceScore}%</strong></div>
            <div>
              <span>Last change</span>
              <strong>{passport.lastChangedAt ? formatDate(passport.lastChangedAt) : "Unknown"}</strong>
            </div>
            <p>{passport.activityNote} · {passport.recoveryMethod}</p>
          </div>
        )}
      </div>
      <div className="finding-size">
        <strong>{formatBytes(finding.estimatedBytes)}</strong>
        <span>{finding.actionAvailable ? "action available" : "inspection only"}</span>
      </div>
    </article>
  );
}
