import { useState } from "react";
import type { FormEvent } from "react";
import { formatBytes } from "../../lib/format";
import type { AiStatus } from "../../types";
import "./IntentPlanner.css";

interface IntentPlannerProps {
  status: AiStatus | null;
  loading: boolean;
  summary: string | null;
  selectedBytes: number;
  onInterpret: (prompt: string) => Promise<void>;
}

export function IntentPlanner({
  status,
  loading,
  summary,
  selectedBytes,
  onInterpret
}: IntentPlannerProps) {
  const [prompt, setPrompt] = useState("");
  const configured = status?.configured ?? false;

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!configured || !prompt.trim() || loading) return;
    await onInterpret(prompt.trim());
  }

  return (
    <section className="intent-planner" aria-labelledby="intent-title">
      <div className="intent-heading">
        <div>
          <span>Optional smart selection</span>
          <h2 id="intent-title">Describe what to keep or reclaim</h2>
        </div>
        <strong>{selectedBytes ? formatBytes(selectedBytes) : "No selection"}</strong>
      </div>

      <form onSubmit={submit}>
        <div className="intent-input-row">
          <input
            id="reclaim-intent"
            aria-label="Reclaim by intent"
            value={prompt}
            onChange={(event) => setPrompt(event.target.value)}
            maxLength={1000}
            disabled={!configured || loading}
            placeholder="Free 20 GB, keep Ollama and Android emulators"
          />
          <button
            className="button button-secondary"
            type="submit"
            disabled={!configured || loading || !prompt.trim()}
          >
            {loading ? "Working…" : "Suggest"}
          </button>
        </div>
      </form>

      {!configured && (
        <p className="intent-status"><code>OPENAI_API_KEY</code> is not configured.</p>
      )}
      {summary && <p className="intent-summary">{summary}</p>}
      <p className="intent-privacy">Only labels, sizes, risk and consequences are sent. Paths stay local.</p>
    </section>
  );
}
