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
  return (
    <section className="page plan-view">
      <header className="page-header">
        <div>
          <span className="page-kicker">Final check</span>
          <h1>Review plan</h1>
          <p>Only the actions listed below will run.</p>
        </div>
        <div className="header-metrics">
          <div><span>Actions</span><strong>{plan.items.length}</strong></div>
          <div className="is-accent"><span>Estimated</span><strong>{formatBytes(plan.estimatedReclaimBytes)}</strong></div>
        </div>
      </header>

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
                <h3>{item.displayName}</h3>
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
          {executing ? "Cleaning…" : "Execute plan"}
        </button>
      </footer>
    </section>
  );
}
