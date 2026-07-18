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
    <section className="view plan-view">
      <header className="view-header">
        <div>
          <p className="eyebrow">Immutable review plan</p>
          <h1>Know the consequence before anything changes.</h1>
          <p>The engine will execute exactly this hashed plan and nothing else.</p>
        </div>
        <div className="view-stat view-stat-accent">
          <strong>{formatBytes(plan.estimatedReclaimBytes)}</strong>
          <span>estimated reclaim</span>
        </div>
      </header>

      <div className="plan-sheet">
        <div className="plan-sheet-head">
          <div><span>Plan {plan.id.slice(0, 8)}</span><strong>{plan.items.length} approved actions</strong></div>
          <code>{plan.planHash.slice(0, 18)}…</code>
        </div>
        <div className="plan-items">
          {plan.items.map((item) => (
            <article className="plan-item" key={item.findingId}>
              <CheckIcon />
              <div><h3>{item.displayName}</h3><p>{item.consequence}</p></div>
              <strong>{formatBytes(item.estimatedBytes)}</strong>
            </article>
          ))}
        </div>
        <div className="protected-proof">
          <ShieldIcon />
          <div>
            <strong>Protected by design</strong>
            <p>Prefetch, browser profiles, Ollama models, Docker volumes, Android emulators, Windows directories and active project source are outside this plan.</p>
          </div>
        </div>
      </div>

      {error && <p className="error-banner">{error}</p>}
      <footer className="plan-actions">
        <button className="button button-quiet" onClick={onBack} disabled={executing}>Change selection</button>
        <button className="button button-danger" onClick={onExecute} disabled={executing}>
          {executing ? "Verifying and cleaning…" : "Execute reviewed plan"}
        </button>
      </footer>
    </section>
  );
}
