import { CheckIcon, ShieldIcon } from "../../components/Icons";
import { formatBytes } from "../../lib/format";
import type { CleanupPlan } from "../../types";

interface PlanViewProps {
  plan: CleanupPlan;
  executing: boolean;
  error: string | null;
  onBack: () => void;
  onExecute: () => void;
}

export function PlanView({ plan, executing, error, onBack, onExecute }: PlanViewProps) {
  const simulation = plan.simulation;
  const total = Math.max(1, simulation.estimatedReclaimBytes);
  const restorableBytes = simulation.reversibleBytes;

  return (
    <section className="page plan-view simple-plan-view">
      <header className="page-header simple-page-header">
        <div>
          <span className="page-kicker">Step 3 of 3</span>
          <h1>Confirm cleanup</h1>
          <p>Review the short list below. Nothing else will be touched.</p>
        </div>
      </header>

      <section className="surface simple-confirmation-card">
        <span className="simple-result-check" aria-hidden="true">✓</span>
        <span className="surface-label">Ready to clean</span>
        <h2>Free about {formatBytes(plan.estimatedReclaimBytes)}</h2>
        <p>{plan.items.length} item{plan.items.length === 1 ? "" : "s"} will be cleaned.</p>
        {restorableBytes > 0 && (
          <span className="confirmation-restore-note">
            {formatBytes(restorableBytes)} can be restored for seven days.
          </span>
        )}
      </section>

      <section className="surface simple-plan-list">
        <header>
          <strong>What will be cleaned</strong>
          <span>{plan.items.length} item{plan.items.length === 1 ? "" : "s"}</span>
        </header>
        <div className="plan-items">
          {plan.items.map((item) => (
            <article className="plan-item simple-plan-item" key={item.findingId}>
              <span className="plan-check"><CheckIcon /></span>
              <div>
                <h3>{item.displayName}</h3>
                <p>{item.consequence}</p>
              </div>
              <strong>{formatBytes(item.estimatedBytes)}</strong>
            </article>
          ))}
        </div>
      </section>

      <section className="simple-protection-note">
        <ShieldIcon />
        <div>
          <strong>Your important files are not included</strong>
          <span>Browser profiles, local AI models, Android emulators, project source and unknown folders stay untouched.</span>
        </div>
      </section>

      <details className="surface cleanup-technical-details">
        <summary>Technical details</summary>
        <div className="cleanup-technical-content">
          <div className="simulation-stats">
            <div><span>Free space now</span><strong>{formatBytes(simulation.currentFreeBytes)}</strong></div>
            <div><span>Free space after cleanup</span><strong>{formatBytes(simulation.projectedFreeBytes)}</strong></div>
            <div><span>Cannot be undone</span><strong>{formatBytes(simulation.irreversibleBytes)}</strong></div>
          </div>
          <div className="simulation-bar" aria-label="Cleanup recovery breakdown">
            <span className="segment-reversible" style={{ width: `${simulation.reversibleBytes / total * 100}%` }} />
            <span className="segment-redownload" style={{ width: `${simulation.redownloadableBytes / total * 100}%` }} />
            <span className="segment-rebuild" style={{ width: `${simulation.rebuildableBytes / total * 100}%` }} />
            <span className="segment-irreversible" style={{ width: `${simulation.irreversibleBytes / total * 100}%` }} />
          </div>
          <code>Plan {plan.id.slice(0, 8)} · {plan.planHash.slice(0, 20)}…</code>
        </div>
      </details>

      {error && <p className="error-banner">{error}</p>}

      <footer className="page-action-row simple-confirm-actions">
        <button className="button button-secondary" onClick={onBack} disabled={executing}>Back</button>
        <button className="button button-danger simple-primary-action" onClick={onExecute} disabled={executing}>
          {executing ? "Cleaning…" : `Free ${formatBytes(plan.estimatedReclaimBytes)} now`}
        </button>
      </footer>
    </section>
  );
}
