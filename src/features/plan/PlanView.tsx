import { CheckIcon, ShieldIcon } from "../../components/Icons";
import { formatBytes } from "../../lib/format";
import { actionRecoveryLabels } from "../../lib/plainLanguage";
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

  return (
    <section className="page plan-view">
      <header className="page-header">
        <div>
          <span className="page-kicker">Before you clean</span>
          <h1>Review your cleanup</h1>
          <p>Check what will be removed, how much space it may free and whether it can be restored.</p>
        </div>
        <div className="header-metrics">
          <div><span>Items</span><strong>{plan.items.length}</strong></div>
          <div className="is-accent"><span>Space to free</span><strong>{formatBytes(plan.estimatedReclaimBytes)}</strong></div>
        </div>
      </header>

      <section className="surface simulation-panel">
        <div className="simulation-primary">
          <span>Free space after cleanup</span>
          <strong>{formatBytes(simulation.projectedFreeBytes)}</strong>
          <small>{formatBytes(simulation.currentFreeBytes)} free now</small>
        </div>
        <div className="simulation-stats">
          <div><span>Time to restore or download again</span><strong>{simulation.estimatedRecoveryMinutes || 0} min</strong></div>
          <div><span>Blocked items</span><strong>{simulation.protectedItemsTouched}</strong></div>
          <div><span>Items selected</span><strong>{simulation.affectedItems}</strong></div>
        </div>
        <div className="simulation-bar" aria-label="What happens after cleanup">
          <span className="segment-reversible" style={{ width: `${simulation.reversibleBytes / total * 100}%` }} />
          <span className="segment-redownload" style={{ width: `${simulation.redownloadableBytes / total * 100}%` }} />
          <span className="segment-rebuild" style={{ width: `${simulation.rebuildableBytes / total * 100}%` }} />
          <span className="segment-irreversible" style={{ width: `${simulation.irreversibleBytes / total * 100}%` }} />
        </div>
        <div className="simulation-legend">
          <span><i className="legend-reversible" />Can restore {formatBytes(simulation.reversibleBytes)}</span>
          <span><i className="legend-redownload" />Download again {formatBytes(simulation.redownloadableBytes)}</span>
          <span><i className="legend-rebuild" />App recreates {formatBytes(simulation.rebuildableBytes)}</span>
          <span><i className="legend-irreversible" />Cannot undo {formatBytes(simulation.irreversibleBytes)}</span>
        </div>
      </section>

      <section className="surface plan-sheet">
        <div className="plan-sheet-head">
          <div>
            <span>Cleanup {plan.id.slice(0, 8)}</span>
            <strong>Locked cleanup list</strong>
          </div>
          <code>{plan.planHash.slice(0, 20)}…</code>
        </div>

        <div className="plan-items">
          {plan.items.map((item) => (
            <article className="plan-item" key={item.findingId}>
              <span className="plan-check"><CheckIcon /></span>
              <div>
                <div className="plan-item-title">
                  <h3>{item.displayName}</h3>
                  <span>{actionRecoveryLabels[item.actionKind]}</span>
                </div>
                <p>{item.consequence}</p>
              </div>
              <strong>{formatBytes(item.estimatedBytes)}</strong>
            </article>
          ))}
        </div>

        <div className="protected-proof">
          <ShieldIcon />
          <div>
            <strong>Important data is left alone</strong>
            <p>Browser profiles, local AI models, Docker volumes, Android emulators, Windows system folders and project source are excluded.</p>
          </div>
        </div>
      </section>

      {error && <p className="error-banner">{error}</p>}

      <footer className="page-action-row">
        <button className="button button-secondary" onClick={onBack} disabled={executing}>Change selection</button>
        <button className="button button-danger" onClick={onExecute} disabled={executing}>
          {executing ? "Cleaning…" : "Clean selected items"}
        </button>
      </footer>
    </section>
  );
}
