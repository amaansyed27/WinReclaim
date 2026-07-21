import { useMemo, useState } from "react";
import { ArrowIcon, ShieldIcon } from "../../components/Icons";
import { formatBytes } from "../../lib/format";
import {
  driveForFinding,
  friendlyFindingName,
  groupForFinding,
  storageGroups,
  type StorageGroupId
} from "../../lib/storageGroups";
import type { AiStatus, DriveInfo, Finding, ReclaimPassport, ScanReport } from "../../types";
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
  const drives = report.drives ?? [];
  const [query, setQuery] = useState("");
  const [driveFilter, setDriveFilter] = useState<string>("all");
  const normalizedQuery = query.trim().toLocaleLowerCase();

  const matchesQuery = (finding: Finding) =>
    !normalizedQuery ||
    [
      friendlyFindingName(finding),
      finding.displayName,
      finding.category,
      finding.path,
      finding.explanation,
      finding.consequence,
      storageGroups.find((group) => group.id === groupForFinding(finding))?.label ?? ""
    ].some((value) => value.toLocaleLowerCase().includes(normalizedQuery));

  const matchesDrive = (finding: Finding) => {
    if (driveFilter === "all") return true;
    return driveForFinding(finding, drives)?.root === driveFilter;
  };

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

  const recommended = allRecommended.filter(matchesDrive).filter(matchesQuery);
  const optional = allOptional.filter(matchesDrive).filter(matchesQuery);
  const reviewOnly = allReviewOnly.filter(matchesDrive).filter(matchesQuery);

  const selectedBytes = report.findings
    .filter((finding) => selectedIds.has(finding.id))
    .reduce((total, finding) => total + finding.estimatedBytes, 0);
  const recommendedBytes = recommended.reduce((sum, finding) => sum + finding.estimatedBytes, 0);
  const optionalBytes = optional.reduce((sum, finding) => sum + finding.estimatedBytes, 0);
  const reviewOnlyBytes = reviewOnly.reduce((sum, finding) => sum + finding.estimatedBytes, 0);
  const allRecommendedBytes = allRecommended.reduce((sum, finding) => sum + finding.estimatedBytes, 0);
  const rebuildableBytes = rebuildable.reduce((sum, finding) => sum + finding.estimatedBytes, 0);
  const visibleCount = recommended.length + optional.length + reviewOnly.length;

  const inspectionGroups = useMemo(
    () => groupInspectionFindings(reviewOnly, drives),
    [reviewOnly, drives]
  );

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
          <h1>Storage report and cleanup</h1>
          <p>Drive totals are shown first, followed by deterministic storage categories and verified cleanup actions.</p>
        </div>
      </header>

      <StorageOverview report={report} />

      {drives.length > 1 && (
        <section className="surface drive-filter" aria-label="Filter results by drive">
          <span>Show findings for</span>
          <div>
            <button
              type="button"
              className={driveFilter === "all" ? "is-active" : ""}
              onClick={() => setDriveFilter("all")}
            >
              All drives
            </button>
            {drives.map((drive) => (
              <button
                type="button"
                key={drive.volumeId}
                className={driveFilter === drive.root ? "is-active" : ""}
                onClick={() => setDriveFilter(drive.root)}
              >
                {drive.root.replace("\\", "")} {drive.label ? `· ${drive.label}` : ""}
              </button>
            ))}
          </div>
        </section>
      )}

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
            placeholder="Search by name, category, owner or path"
          />
        </label>
        <span>{normalizedQuery || driveFilter !== "all" ? `${visibleCount} of ${report.findings.length} locations` : `${report.findings.length} locations`}</span>
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
                <strong>{reviewOnly.length} protected or inspection-only locations</strong>
                <small>Reported rows total {formatBytes(reviewOnlyBytes)} and can overlap. Drive totals above are authoritative.</small>
              </span>
            </div>
            <span>Show categories</span>
          </summary>
          <div className="review-only-content categorized-inspection">
            {inspectionGroups.map((group) => (
              <section className="inspection-category" key={`${group.driveRoot}-${group.id}`}>
                <div className="inspection-category-head">
                  <div>
                    <span>{group.driveLabel}</span>
                    <h3>{group.label}</h3>
                    <p>{group.description}</p>
                  </div>
                  <div>
                    <span>{group.items.length} location{group.items.length === 1 ? "" : "s"}</span>
                    <strong>{formatBytes(group.bytes)}</strong>
                  </div>
                </div>
                <div className="finding-list">
                  {group.items.map((finding) => (
                    <FindingRow
                      finding={{ ...finding, displayName: friendlyFindingName(finding) }}
                      passport={passports.get(finding.id)}
                      key={finding.id}
                      selected={false}
                      onToggle={onToggle}
                    />
                  ))}
                </div>
              </section>
            ))}
          </div>
        </details>
      )}

      {(normalizedQuery || driveFilter !== "all") && visibleCount === 0 && (
        <section className="surface simple-empty-card">
          <strong>No matching locations</strong>
          <span>Try another drive, folder name, tool name, category or part of a path.</span>
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

function StorageOverview({ report }: { report: ScanReport }) {
  const drives = report.drives ?? [];
  const grouped = storageGroups
    .map((definition) => {
      const items = report.findings.filter((finding) => groupForFinding(finding) === definition.id);
      return {
        ...definition,
        items,
        bytes: items.reduce((total, finding) => total + finding.estimatedBytes, 0),
        largest: items[0]
      };
    })
    .filter((group) => group.items.length > 0)
    .sort((a, b) => b.bytes - a.bytes);

  return (
    <section className="surface storage-overview">
      <div className="storage-overview-head">
        <div>
          <span className="surface-label">Drive overview</span>
          <h2>{formatBytes(report.disk.usedBytes)} used across {drives.length || 1} drive{drives.length === 1 ? "" : "s"}</h2>
          <p>{formatBytes(report.disk.freeBytes)} free. Folder rows below can overlap; volume totals do not.</p>
        </div>
      </div>

      <div className="overview-drive-grid">
        {(drives.length ? drives : [{
          root: report.disk.root,
          label: "Selected drive",
          fileSystem: "",
          volumeId: report.disk.root,
          totalBytes: report.disk.totalBytes,
          freeBytes: report.disk.freeBytes,
          usedBytes: report.disk.usedBytes,
          isSystem: true,
          kind: "fixed" as const
        }]).map((drive) => {
          const percent = drive.totalBytes ? Math.round((drive.usedBytes / drive.totalBytes) * 100) : 0;
          return (
            <article key={drive.volumeId}>
              <div>
                <strong>{drive.root.replace("\\", "")}</strong>
                <span>{drive.label || (drive.isSystem ? "Windows" : "Local drive")}</span>
              </div>
              <b>{formatBytes(drive.usedBytes)} used</b>
              <div className="overview-drive-bar"><span style={{ width: `${Math.min(100, percent)}%` }} /></div>
              <small>{formatBytes(drive.freeBytes)} free · {percent}% used</small>
            </article>
          );
        })}
      </div>

      <div className="storage-category-grid">
        {grouped.map((group) => (
          <article key={group.id}>
            <span>{group.label}</span>
            <strong>{formatBytes(group.bytes)}</strong>
            <small>{group.items.length} reported location{group.items.length === 1 ? "" : "s"}{group.largest ? ` · largest ${friendlyFindingName(group.largest)}` : ""}</small>
          </article>
        ))}
      </div>
    </section>
  );
}

function groupInspectionFindings(findings: Finding[], drives: DriveInfo[]) {
  const groups = new Map<string, {
    id: StorageGroupId;
    label: string;
    description: string;
    driveRoot: string;
    driveLabel: string;
    items: Finding[];
    bytes: number;
  }>();

  for (const finding of findings) {
    const id = groupForFinding(finding);
    const definition = storageGroups.find((group) => group.id === id)!;
    const drive = driveForFinding(finding, drives);
    const driveRoot = drive?.root ?? "Other";
    const key = `${driveRoot}|${id}`;
    const group = groups.get(key) ?? {
      id,
      label: definition.label,
      description: definition.description,
      driveRoot,
      driveLabel: drive ? `${drive.root.replace("\\", "")} ${drive.label || ""}`.trim() : "Other paths",
      items: [],
      bytes: 0
    };
    group.items.push(finding);
    group.bytes += finding.estimatedBytes;
    groups.set(key, group);
  }

  return [...groups.values()]
    .map((group) => ({
      ...group,
      items: group.items.sort((a, b) => b.estimatedBytes - a.estimatedBytes)
    }))
    .sort((a, b) => b.bytes - a.bytes);
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
            finding={{ ...finding, displayName: friendlyFindingName(finding) }}
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
