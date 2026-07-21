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

  if (!configured) return null;

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!prompt.trim() || loading) return;
    await onInterpret(prompt.trim());
  }

  return (
    <details className="intent-planner simple-intent-planner">
      <summary>
        <span>Need help choosing?</span>
        <small>Describe what you want to keep or clean</small>
      </summary>
      <div className="simple-intent-content">
        <form onSubmit={submit}>
          <div className="intent-input-row">
            <input
              id="reclaim-intent"
              aria-label="Describe what to clean or keep"
              value={prompt}
              onChange={(event) => setPrompt(event.target.value)}
              maxLength={1000}
              disabled={loading}
              placeholder="Free some space, but keep downloads and work folders"
            />
            <button
              className="button button-secondary"
              type="submit"
              disabled={loading || !prompt.trim()}
            >
              {loading ? "Choosing…" : "Suggest items"}
            </button>
          </div>
        </form>
        {summary && <p className="intent-summary">{summary}</p>}
        <p className="intent-privacy">
          Paths, usernames, folder names and file contents stay local. Routed by {status?.model}. Currently selected: {formatBytes(selectedBytes)}.
        </p>
      </div>
    </details>
  );
}
