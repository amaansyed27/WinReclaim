import { useState } from "react";
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
  const [query, setQuery] = useState("");
  const normalizedQuery = query.trim().toLocaleLowerCase();
  const matchesQuery = (finding: Finding) =>
    !normalizedQuery ||
    [
      finding.displayName,
      finding.category,
      finding.path,
      finding.explanation,
      finding.consequence
    ].some((value) => value.toLocaleLowerCase().includes(normalizedQuery));

  const allRecommended = report.findings
    .filter((finding) => finding.actionAvailable && finding.riskClass === "safe_now")
    .sort((a, b) => b.estimatedBytes - a.estimatedBytes);
  const allOptional = report.findings
    .filter((finding) => finding.actionAvailable && finding.riskClass !== "safe_now")
    .sort((a, b) => b.estimatedBytes - a.estimatedBytes);
  const allReviewOnly = report.findings
    .filter((finding) => !finding.actionAvailable)
    .sort((a, b) => b.estimatedBytes - a.estimatedBytes);
  const rebuildable = allOptional.filter(
    (finding) => finding.riskClass === "rebuild_or_redownload"
  );

  const recommended = allRecommended.filter(matchesQuery);
  const optional = allOptional.filter(matchesQuery);
  const reviewOnly = allReviewOnly.filter(matchesQuery);

  const selectedBytes = report.findings
    .filter((finding) => selectedIds.has(finding.id))
    .reduce((total, finding) => total + finding.estimatedBytes, 0);
  const recommendedBytes = recommended.reduce((sum, finding) => sum + finding.estimatedBytes, 0);
  const optionalBytes = optional.reduce((sum, finding) => sum + finding.estimatedBytes, 0);
  const reviewOnlyBytes = reviewOnly.reduce((sum, finding) => sum + finding.estimatedBytes, 0);
  const allRecommendedBytes = allRecommended.reduce((sum, finding) => sum + finding.estimatedBytes, 0);
  const rebuildableBytes = rebuildable.reduce((sum, finding) => sum + finding.estimatedBytes, 0);
  const visibleCount = recommended.length + optional.length + reviewOnly.length;

  function selectFindings(findings: Finding[]) {
    findings.forEach((finding) => {
      if (!selectedIds.has(finding.id)) onToggle(finding.id);
    });
  }

  function selectRecommended() {
    selectFindings(allRecommended);
  }

  function selectRebuildable() {
    selectFindings(rebuildable);
  }

  function clearSelection() {
    selectedIds.forEach((id) => onToggle(id));
  }

  return (
    <section className="page findings-view simple-findings-view">
      <header className="page-header simple-page-header">
        <div>
          <span className="page-kicker">Step 2 of 3</span>
          <h1>Choose what to clean</h1>
          <p>The safest items are grouped first. Every amount and consequence comes from the completed scan.</p>
        </div>
      </header>

      <section className="surface cleanup-recommendation">
        <div>
          <span className="recommendation-check" aria-hidden="true">✓</span>
          <div>
            <span className="surface-label">Recommended cleanup</span>
            <h2>
              {allRecommended.length
                ? `${formatBytes(allRecommendedBytes)} is ready for review`
                : "No recommended cleanup was found"}
            </h2>
            <p>
              {allRecommended.length
                ? "Temporary locations are measured in full. Locked, active or inaccessible entries are skipped during cleanup."
                : "Optional and inspection-only findings are still listed below when the scan discovered them."}
            </p>
          </div>
        </div>
        <div className="recommendation-actions">
          <button className="button button-primary" onClick={selectRecommended} disabled={!allRecommended.length}>Use recommendation</button>
          <button className="button button-secondary" onClick={selectRebuildable} disabled={!rebuildable.length}>
            Select rebuildable caches ({formatBytes(rebuildableBytes)})
          </button>
          <button className="button button-secondary" onClick={clearSelection} disabled={!selectedIds.size}>Clear selection</button>
        </div>
      </section>

      <section className="surface finding-filter-bar" aria-label="Filter scan results">
        <label htmlFor="finding-search">
          <span>Search results</span>
          <input
            id="finding-search"
            type="search"
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder="Search by name, category or path"
          />
        </label>
        <span>{normalizedQuery ? `${visibleCount} of ${report.findings.length} locations` : `${report.findings.length} locations`}</span>
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
        note="Verified cleanup actions with the lowest expected impact."
        items={recommended}
        bytes={recommendedBytes}
        passports={passports}
        selectedIds={selectedIds}
        onToggle={onToggle}
      />

      {optional.length > 0 && (
        <FindingSection
          title="Optional cleanup"
          note="Rebuildable, redownloadable or destructive actions. Review the stated consequence before selecting them."
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
                <strong>{reviewOnly.length} locations WinReclaim will not clean</strong>
                <small>They use {formatBytes(reviewOnlyBytes)}. These are shown for inspection only.</small>
              </span>
            </div>
            <span>Show locations</span>
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

      {normalizedQuery && visibleCount === 0 && (
        <section className="surface simple-empty-card">
          <strong>No matching locations</strong>
          <span>Try a folder name, tool name, category or part of a path.</span>
        </section>
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
