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
          <span>Optional helper</span>
          <h2 id="intent-title">Tell WinReclaim what you want</h2>
        </div>
        <strong>{selectedBytes ? formatBytes(selectedBytes) : "Nothing selected"}</strong>
      </div>

      <form onSubmit={submit}>
        <div className="intent-input-row">
          <input
            id="reclaim-intent"
            aria-label="Describe what to clean or keep"
            value={prompt}
            onChange={(event) => setPrompt(event.target.value)}
            maxLength={1000}
            disabled={!configured || loading}
            placeholder="Free about 20 GB, but keep my local AI models and Android emulators"
          />
          <button
            className="button button-secondary"
            type="submit"
            disabled={!configured || loading || !prompt.trim()}
          >
            {loading ? "Choosing…" : "Choose for me"}
          </button>
        </div>
      </form>

      {!configured && (
        <p className="intent-status">Smart suggestions are off. Add <code>OPENAI_API_KEY</code> to enable them.</p>
      )}
      {summary && <p className="intent-summary">{summary}</p>}
      <p className="intent-privacy">File names and folder paths stay on this PC.</p>
    </section>
  );
}
