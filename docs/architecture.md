# Architecture

WinReclaim is a Windows-first Tauri 2 application organized around strict boundaries between discovery, interpretation, judgment, planning and execution.

The architecture is designed so that a UI bug, model output or broad filesystem discovery cannot directly become arbitrary deletion authority.

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
  ├─ optional OpenAI intent constraints
  ├─ immutable cleanup planner and simulation
  ├─ compiled cleanup adapters
  ├─ measured receipts
  ├─ compressed Undo Vault and restore engine
  ├─ optional local Storage Assistant sidecar
  └─ signed updater
```

## Frontend

The React application is organized by feature under `src/features`.

Major areas:

- `scan` — drive selection, scan profile and progress;
- `timeline` — Storage Time Machine history and deltas;
- `findings` — grouped findings, Reclaim Passports and selection;
- `plan` — simulation and final confirmation;
- `receipt` — measured execution results;
- `vault` — restore entries and outcomes;
- `settings` — scan defaults, data reset, assistant and application state;
- `assistant` — optional local-model installation and advisory report;
- `update` — signed update checks and installation.

The frontend is responsible for presentation and user intent. It is not trusted to define filesystem authority.

Normal Tauri wrappers live in `src/lib/tauri.ts`. The frontend sends typed options, scan IDs, finding IDs, plan IDs and hashes. It does not submit arbitrary deletion paths.

## Rust module map

The desktop backend is registered in `src-tauri/src/lib.rs`.

### `commands`

The main Tauri command boundary. Commands translate frontend requests into backend operations and return serialized domain models.

Responsibilities:

- validate request shape;
- coordinate application state;
- avoid exposing internal paths unnecessarily;
- return structured errors;
- keep mutation authority in backend modules.

See [command-api.md](command-api.md).

### `domain`

Shared Rust domain types for drives, scans, findings, plans, receipts, timelines, vault entries and assistant results.

Domain objects should distinguish:

- measured versus estimated values;
- safety class versus action availability;
- recovery class versus restore availability;
- persisted identity versus display labels.

### `scanner`

Performs bounded filesystem inspection for selected drives and scan profiles.

Current responsibilities include:

- known target discovery;
- project-output discovery;
- optional AppData and dynamic large-folder discovery;
- Windows cache discovery;
- size measurement;
- cancellation and progress events;
- reparse-point refusal;
- limits for thresholds and unknown results;
- exclusion of WinReclaim-owned state.

The scanner does not execute cleanup.

### `rules`

Converts deterministic filesystem evidence into semantic findings.

A finding can contain:

- stable rule/finding identity;
- owner and category;
- explanation and consequence;
- safety class;
- evidence-based confidence;
- optional compiled action kind.

Rule data cannot contain arbitrary shell commands or deletion globs. Protected policy overrides an accidentally attached action.

See [rules.md](rules.md) and [rule-authoring.md](rule-authoring.md).

### `policy`

Centralizes protected path and action policy. This layer is deliberately independent from display wording.

Policy is consulted during discovery/planning and should be reapplied immediately before mutation. A more restrictive classification wins.

### `insights`

Builds derived local intelligence such as:

- Storage Time Machine snapshots;
- compatible scan deltas;
- likely ownership attribution;
- Reclaim Passports;
- recovery context.

Insight generation cannot create executable actions.

### `intent`

Implements the optional remote reclaim-by-intent feature.

When configured, Rust sends a constrained set of anonymized candidate metadata to the OpenAI Responses API. The model can propose target size, allowed safety classes, exclusions and explanation. A deterministic selector validates the output against the current scan.

The feature cannot provide paths, commands, action kinds or plan execution.

### `planner`

Resolves user-selected finding IDs against backend-owned scan state.

The planner:

1. rejects unknown and non-actionable findings;
2. resolves action details in Rust;
3. aggregates estimates and recovery classes;
4. creates a simulation;
5. serializes an immutable plan;
6. computes a plan hash;
7. stores the plan in backend state.

Execution requires both the plan ID and hash. The frontend cannot modify plan content after confirmation.

### `actions`

Contains compiled cleanup adapters.

Adapters may use:

- exact-root entry-by-entry filesystem cleanup;
- generic directory cleanup with deterministic fingerprints;
- compressed Undo Vault movement;
- Windows Shell APIs;
- tool-native commands started with explicit argument arrays.

Adapters must revalidate current filesystem state, reject links/reparse points, enforce allowed roots and return structured results.

### `vault`

Implements reversible cleanup for eligible files.

The vault:

- creates manifest-backed entries;
- preserves original relative paths;
- uses bounded retention;
- applies native NTFS compression where available;
- never overwrites existing restore destinations;
- reports partial restoration.

### `receipts`

Persists execution records under `%LOCALAPPDATA%\WinReclaim\receipts`.

Receipts contain measured results and action outcomes. They are historical records and cannot be replayed as cleanup authority.

### `app_data`

Owns the application-data root and reset semantics.

```text
%LOCALAPPDATA%\WinReclaim
```

It manages data-generation compatibility, snapshot/receipt clearing, vault preservation and model preservation. See [data-layout.md](data-layout.md).

### `platform`

Contains Windows-specific filesystem and storage APIs, including drive enumeration and native Shell interactions. Platform code should expose narrow safe operations to higher layers.

### `assistant` and `assistant_commands`

Implement the optional local Storage Assistant.

The installation path downloads:

- a pinned Qwen3.5-2B GGUF file;
- a pinned `llama.cpp` Windows CPU runtime archive.

Both are verified. The runtime is safely extracted and executed as a sidecar only when the user requests analysis.

Assistant output is validated against the current scan and cannot change risk, action availability, selection or execution.

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

At no point does the frontend submit a raw deletion path.

## Persisted state

Persisted local data includes:

- versioned scan snapshots;
- cleanup receipts;
- vault manifests and payloads;
- optional model/runtime manifest and files;
- internal data-generation marker.

Persisted formats are more stable than internal command shapes because they survive upgrades. Schema changes require compatibility handling or an explicit safe invalidation policy.

## Network boundaries

Core scanning and cleanup are offline.

Intended network activity:

- GitHub Releases for signed updater metadata and installers;
- Hugging Face for user-initiated model download;
- GitHub Releases for user-initiated `llama.cpp` runtime download;
- OpenAI Responses API only when the user configures and invokes reclaim-by-intent;
- GitHub public release API from the static landing page.

See [privacy.md](privacy.md).

## Safety invariants

Architecture changes must preserve:

- protected classification overrides action availability;
- scan discovery does not imply execution authority;
- unknown discoveries remain inspection-only by default;
- no frontend-provided arbitrary paths;
- no shell command interpolation;
- plan ID/hash verification;
- execution-time path and fingerprint validation;
- reparse-point refusal;
- no vault overwrite;
- signed updater verification;
- advisory-only AI components;
- measured receipt results.

## Extension points

Safe extension points include:

- new deterministic rules;
- new compiled adapters with tests;
- additional insight views derived from scan state;
- new scanner backends behind the existing domain boundary;
- persisted schema migrations;
- improved local assistant evaluation.

Potential future scanner backends include NTFS MFT enumeration and USN Change Journal updates. They must retain the same rule, policy, planner and execution boundaries.

## Related documentation

- [Safety model](safety.md)
- [Threat model](threat-model.md)
- [Command API](command-api.md)
- [Rule authoring](rule-authoring.md)
- [Testing](testing.md)
- [Data layout](data-layout.md)
