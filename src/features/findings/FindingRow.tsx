import { formatBytes, formatDate } from "../../lib/format";
import { recoveryLabel } from "../../lib/plainLanguage";
import type { Finding, ReclaimPassport } from "../../types";

interface FindingRowProps {
  finding: Finding;
  passport?: ReclaimPassport;
  selected: boolean;
  onToggle: (id: string) => void;
}

const simpleLabels: Record<Finding["riskClass"], string> = {
  safe_now: "Recommended",
  rebuild_or_redownload: "Optional",
  review_first: "Review only",
  protected: "Protected"
};

export function FindingRow({ finding, passport, selected, onToggle }: FindingRowProps) {
  const canClean = finding.actionAvailable && finding.riskClass !== "protected";

  return (
    <article className={`finding-row simple-finding-row risk-${finding.riskClass} ${selected ? "is-selected" : ""}`}>
      {canClean ? (
        <button
          type="button"
          className={`finding-select ${selected ? "is-selected" : ""}`}
          onClick={() => onToggle(finding.id)}
          aria-label={`${selected ? "Remove" : "Add"} ${finding.displayName} ${selected ? "from" : "to"} cleanup`}
        >
          <span>{selected ? "✓" : ""}</span>
        </button>
      ) : (
        <span className="finding-lock" aria-hidden="true">—</span>
      )}

      <div className="finding-main">
        <div className="finding-title-line">
          <h3>{finding.displayName}</h3>
          <span className={`risk-label risk-label-${finding.riskClass}`}>
            {simpleLabels[finding.riskClass]}
          </span>
        </div>
        <p>{finding.explanation}</p>
        <p className="simple-consequence">
          {canClean ? finding.consequence : "WinReclaim will not remove this folder."}
        </p>

        <details className="finding-details">
          <summary>More details</summary>
          <div className="finding-details-content">
            <code>{finding.path}</code>
            {passport && (
              <>
                <dl>
                  <div><dt>After cleanup</dt><dd>{recoveryLabel(passport.recoveryClass)}</dd></div>
                  <div><dt>Last changed</dt><dd>{passport.lastChangedAt ? formatDate(passport.lastChangedAt) : "Unavailable"}</dd></div>
                </dl>
                <p>{passport.activityNote}. {passport.recoveryMethod}</p>
              </>
            )}
          </div>
        </details>
      </div>

      <div className="finding-size">
        <strong>{formatBytes(finding.estimatedBytes)}</strong>
        <span>{canClean ? (selected ? "Selected" : "Not selected") : "Not cleaned"}</span>
      </div>
    </article>
  );
}
