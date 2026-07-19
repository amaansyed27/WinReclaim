import { CheckIcon, ShieldIcon } from "../../components/Icons";
import { formatBytes } from "../../lib/format";
import type { ActionKind, CleanupPlan } from "../../types";

interface PlanViewProps {
  plan: CleanupPlan;
  executing: boolean;
  error: string | null;
  onBack: () => void;
  onExecute: () => void;
}

const recoveryLabels: Record<ActionKind, string> = {
  user_temp: "Undo Vault",
  crash_dumps: "Undo Vault",
  huggingface_prune: "Redownloadable",
  npm_cache: "Redownloadable",
  docker_prune: "Irreversible"
};

export function PlanView({ plan, executing, error, onBack, onExecute }: PlanViewProps) {
  const simulation = plan.simulation;
  const total = Math.max(1, simulation.estimatedReclaimBytes);

  return (
    <section className="page plan-view">
      <header className="page-header">
        <div>
          <span className="page-kicker">Reclaim Simulation</span>
          <h1>Review impact</h1>
          <p>Preview free space, recovery cost and reversibility before execution.</p>
        </div>
        <div className="header-metrics">
          <div><span>Actions</span><strong>{plan.items.length}</strong></div>
          <div className="is-accent"><span>Estimated</span><strong>{formatBytes(plan.estimatedReclaimBytes)}</strong></div>
        </div>
      </header>

      <section className="surface simulation-panel">
        <div className="simulation-primary">
          <span>Projected free space</span>
          <strong>{formatBytes(simulation.projectedFreeBytes)}</strong>
          <small>{formatBytes(simulation.currentFreeBytes)} currently free</small>
        </div>
        <div className="simulation-stats">
          <div><span>Recovery time</span><strong>{simulation.estimatedRecoveryMinutes || 0} min</strong></div>
          <div><span>Protected touched</span><strong>{simulation.protectedItemsTouched}</strong></div>
          <div><span>Affected items</span><strong>{simulation.affectedItems}</strong></div>
        </div>
        <div className="simulation-bar" aria-label="Reclaim reversibility breakdown">
          <span className="segment-reversible" style={{ width: `${simulation.reversibleBytes / total * 100}%` }} />
          <span className="segment-redownload" style={{ width: `${simulation.redownloadableBytes / total * 100}%` }} />
          <span className="segment-rebuild" style={{ width: `${simulation.rebuildableBytes / total * 100}%` }} />
          <span className="segment-irreversible" style={{ width: `${simulation.irreversibleBytes / total * 100}%` }} />
        </div>
        <div className="simulation-legend">
          <span><i className="legend-reversible" />Undo Vault {formatBytes(simulation.reversibleBytes)}</span>
          <span><i className="legend-redownload" />Redownload {formatBytes(simulation.redownloadableBytes)}</span>
          <span><i className="legend-rebuild" />Rebuild {formatBytes(simulation.rebuildableBytes)}</span>
          <span><i className="legend-irreversible" />Irreversible {formatBytes(simulation.irreversibleBytes)}</span>
        </div>
      </section>

      <section className="surface plan-sheet">
        <div className="plan-sheet-head">
          <div>
            <span>Plan {plan.id.slice(0, 8)}</span>
            <strong>Immutable cleanup plan</strong>
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
                  <span>{recoveryLabels[item.actionKind]}</span>
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
            <strong>Protected categories remain outside this plan</strong>
            <p>Browser profiles, Ollama models, Docker volumes, Android emulators, Windows directories and project source are excluded.</p>
          </div>
        </div>
      </section>

      {error && <p className="error-banner">{error}</p>}

      <footer className="page-action-row">
        <button className="button button-secondary" onClick={onBack} disabled={executing}>Change selection</button>
        <button className="button button-danger" onClick={onExecute} disabled={executing}>
          {executing ? "Executing…" : "Execute simulated plan"}
        </button>
      </footer>
    </section>
  );
}
