# Architecture

WinReclaim is a Windows-first Tauri 2 application organized around strict boundaries between discovery, interpretation, judgment, planning and execution.

The architecture is designed so that a UI bug, remote model output or broad filesystem discovery cannot become arbitrary deletion authority.

## System overview

```text
React + TypeScript webview
  │
  │ typed Tauri commands and events
  ▼
Rust application boundary
  ├─ drive enumeration and scan orchestration
  ├─ bounded scanner and sizing
  ├─ deterministic rule classification
  ├─ timeline and Reclaim Passport insights
  ├─ optional privacy-bounded cloud summaries and intent constraints
  ├─ immutable cleanup planner and simulation
  ├─ compiled cleanup adapters
  ├─ measured receipts
  ├─ compressed Undo Vault and restore engine
  └─ signed updater
```

A separate Vercel function exposes the fixed WinReclaim cloud-assistant contract and calls OpenRouter's `openrouter/free` router. The desktop app never contains the OpenRouter credential.

## Frontend

The React application is organized by feature under `src/features`.

Major areas:

- `scan` — drive selection, scan profile and progress;
- `timeline` — Storage Time Machine history and deltas;
- `findings` — grouped findings, Reclaim Passports and selection;
- `assistant` — optional cloud summary presentation and failure states;
- `plan` — simulation and final confirmation;
- `receipt` — measured execution results;
- `vault` — restore entries and outcomes;
- `settings` — scan defaults, updates and application-data controls;
- `update` — signed update checks and installation.

The frontend is responsible for presentation and user intent. It is not trusted to define filesystem authority.

Normal Tauri wrappers live in `src/lib/tauri.ts`. The frontend sends typed options, scan IDs, finding IDs, plan IDs and hashes. It does not submit arbitrary deletion paths or provider credentials.

## Rust module map

The desktop backend is registered in `src-tauri/src/lib.rs`.

### `commands`

The main Tauri command boundary. Commands validate request shape, coordinate application state, avoid exposing internal paths and keep mutation authority in backend modules.

See [command-api.md](command-api.md).

### `domain`

Shared Rust domain types for drives, scans, findings, plans, receipts, timelines, vault entries and assistant results.

Domain objects distinguish:

- measured versus estimated values;
- safety class versus action availability;
- recovery class versus restore availability;
- persisted identity versus display labels.

### `scanner`

Performs bounded filesystem inspection for selected drives and scan profiles.

Responsibilities include known-target discovery, project-output discovery, optional AppData and dynamic large-folder discovery, Windows cache discovery, size measurement, cancellation, progress events, reparse-point refusal and exclusion of WinReclaim-owned state.

The scanner does not execute cleanup.

### `rules`

Converts deterministic filesystem evidence into semantic findings. Rule data cannot contain arbitrary shell commands or deletion globs. Protected policy overrides an accidentally attached action.

See [rules.md](rules.md) and [rule-authoring.md](rule-authoring.md).

### `policy`

Centralizes protected path and action policy. Policy is reapplied during planning and execution. A more restrictive classification wins.

### `insights`

Builds local derived intelligence such as Storage Time Machine snapshots, compatible scan deltas, ownership attribution, Reclaim Passports and recovery context. Insight generation cannot create executable actions.

### `cloud`

Implements the fixed Rust transport to:

```text
https://winreclaim.vercel.app/api/assistant
```

It accepts only HTTPS endpoints, applies a bounded timeout, identifies the desktop client and parses the typed `{ model, result }` response. `WINRECLAIM_ASSISTANT_URL` may redirect development builds to a preview endpoint.

### `assistant` and `assistant_commands`

Build aggregate storage metadata and request an advisory summary from the cloud proxy.

The assistant payload includes drive totals and category/risk/action counts. It excludes paths, drive labels, usernames, folder names, project names, directory trees and file contents.

Rust validates summary length, observation count and cleanup-claim language. The report is always marked `advisoryOnly: true`.

### `intent`

Implements optional reclaim-by-intent through the same proxy. Rust sends the user sentence plus opaque executable-candidate IDs, category, size, deterministic risk and consequence.

The routed model can propose only target size, allowed safety classes, exclusions and a short explanation. A deterministic selector validates the output against the current scan. The feature cannot provide paths, commands, action kinds or execution.

### `planner`

Resolves user-selected finding IDs against backend-owned scan state. It rejects unknown or non-actionable findings, resolves actions in Rust, creates a simulation and immutable plan, computes a plan hash and stores the plan in backend state.

Execution requires both plan ID and hash. The frontend cannot modify plan content after confirmation.

### `actions`

Contains compiled cleanup adapters. Adapters revalidate filesystem state, reject links and reparse points, enforce allowed roots and return structured results.

### `vault`

Implements reversible cleanup for eligible files with manifest-backed entries, original relative paths, bounded retention, NTFS compression where available and refusal to overwrite restore destinations.

### `receipts`

Persists measured execution results under `%LOCALAPPDATA%\WinReclaim\receipts`. Receipts are historical records and cannot be replayed as cleanup authority.

### `app_data`

Owns `%LOCALAPPDATA%\WinReclaim`, data-generation compatibility, snapshot/receipt clearing and vault preservation. Version 1.2.1 also removes the retired `%LOCALAPPDATA%\WinReclaim\models\storage-assistant` directory during startup.

See [data-layout.md](data-layout.md).

### `platform`

Contains Windows-specific filesystem and storage APIs, including drive enumeration and native Shell interactions. Platform code exposes narrow operations to higher layers.

## Vercel proxy boundary

`landing-page/api/assistant.js` is the only component that receives `OPENROUTER_API_KEY`.

The proxy:

- accepts only fixed `storage_summary` and `intent_constraints` tasks;
- rejects malformed and oversized payloads;
- fixes the model to `openrouter/free`;
- requires JSON Schema output;
- asks OpenRouter to route only to providers supporting required parameters;
- caps candidates, categories and output tokens;
- validates all returned fields;
- applies a basic per-IP demo limit;
- does not accept arbitrary prompts, models, tools or provider settings from the client.

The provider credential is a Vercel environment secret, not an application secret. Distributed clients cannot securely contain a reusable provider API key.

## End-to-end scan flow

```text
User selects drives/profile
  → frontend sends ScanOptions
  → Rust normalizes options
  → platform layer enumerates allowed drives
  → scanner discovers and sizes candidates
  → rules classify candidates
  → policy applies protected precedence
  → scan report is stored in backend state
  → snapshot is persisted
  → frontend receives report and progress events
  → insights generate passports/timeline
```

## End-to-end assistant flow

```text
User explicitly requests summary or intent help
  → Rust reads current backend-owned scan state
  → Rust constructs bounded anonymized metadata
  → desktop posts to fixed WinReclaim proxy
  → proxy validates request and calls openrouter/free
  → proxy validates structured response
  → Rust validates response again
  → UI displays advisory output only
```

No remote output becomes an action, path or plan.

## End-to-end cleanup flow

```text
User selects actionable finding IDs
  → planner resolves IDs against current scan
  → planner builds simulation and immutable plan
  → plan is hashed and stored
  → user confirms
  → frontend submits plan ID + hash
  → backend verifies stored plan
  → each adapter revalidates current path/evidence
  → action executes or refuses/skips
  → free space is measured
  → receipt is persisted
  → vault state is refreshed
```

At no point does the frontend or cloud model submit a raw deletion path.

## Persisted state

Persisted local data includes:

- versioned scan snapshots;
- cleanup receipts;
- vault manifests and payloads;
- internal data-generation marker.

The cloud assistant does not persist a local model, runtime, prompt file or provider key.

## Network boundaries

Core scanning and cleanup are offline.

Intended network activity:

- GitHub Releases for signed updater metadata and installers;
- the WinReclaim Vercel proxy and OpenRouter only after an explicit assistant action;
- GitHub public release metadata from the landing page.

See [privacy.md](privacy.md).

## Safety invariants

Architecture changes must preserve:

- protected classification overrides action availability;
- scan discovery does not imply execution authority;
- unknown discoveries remain inspection-only by default;
- no frontend- or model-provided arbitrary paths;
- no shell command interpolation;
- plan ID/hash verification;
- execution-time path and fingerprint validation;
- reparse-point refusal;
- no vault overwrite;
- signed updater verification;
- advisory-only model output;
- measured receipt results.

## Related documentation

- [Safety model](safety.md)
- [Threat model](threat-model.md)
- [Storage Assistant](storage-assistant.md)
- [Command API](command-api.md)
- [Rule authoring](rule-authoring.md)
- [Testing](testing.md)
- [Data layout](data-layout.md)
