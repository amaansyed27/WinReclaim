import { FormEvent, useState } from "react";
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
          <span>Optional · {status?.model ?? "GPT-5.6"}</span>
          <h2 id="intent-title">Describe the boundary, not the files.</h2>
        </div>
        <strong>{selectedBytes ? formatBytes(selectedBytes) : "No suggestion yet"}</strong>
      </div>

      <form onSubmit={submit}>
        <label htmlFor="reclaim-intent">Reclaim by intent</label>
        <div className="intent-input-row">
          <input
            id="reclaim-intent"
            value={prompt}
            onChange={(event) => setPrompt(event.target.value)}
            maxLength={1000}
            disabled={!configured || loading}
            placeholder="Free around 20 GB, but do not touch Ollama, browser profiles or Android emulators."
          />
          <button
            className="button button-secondary"
            type="submit"
            disabled={!configured || loading || !prompt.trim()}
          >
            {loading ? "Interpreting…" : "Suggest selection"}
          </button>
        </div>
      </form>

      {!configured && (
        <p className="intent-status">
          Set <code>OPENAI_API_KEY</code> before launching the app to enable this optional feature.
        </p>
      )}
      {summary && <p className="intent-summary">{summary}</p>}
      <p className="intent-privacy">
        {status?.privacyNote ??
          "Only anonymized category, size, risk and consequence metadata is sent. Paths remain local."}
      </p>
    </section>
  );
}
