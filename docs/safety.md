# Safety Model

WinReclaim explains storage broadly and executes cleanup narrowly. Safety is enforced in Rust and never depends on UI wording or remote model judgment.

## Safety classes

Every finding has a safety classification independent from action availability.

- **Safe now** — narrowly scoped disposable data with a known owner and consequence.
- **Rebuild or redownload** — reproducible data whose removal can cost bandwidth, compute or setup time.
- **Review first** — data that may disrupt environments, containers, emulators, launch performance or workflows.
- **Protected** — data WinReclaim refuses to add to an executable cleanup plan.

Protected always overrides a less restrictive classification or accidentally attached action.

## Non-negotiable protections

WinReclaim does not automatically remove:

- registry data;
- browser profiles or extensions;
- local model stores, including Ollama models;
- Docker volumes;
- Android virtual devices or SDK packages by raw deletion;
- project source or repositories;
- credentials and configuration;
- Program Files or arbitrary Windows directories;
- WinReclaim snapshots, receipts or vault payloads;
- unknown folders discovered only by size or generic naming.

Prefetch is available only as an explicitly selected **Review first** action restricted to `.pf` files under the active Windows Prefetch root.

## Detection is not deletion authority

A finding becomes executable only when:

1. deterministic evidence identifies it;
2. protected policy permits it;
3. a compiled adapter exists;
4. the current drive policy permits mutation;
5. the user selects it;
6. Rust creates and hashes an immutable plan;
7. the user confirms the plan;
8. execution-time validation still succeeds.

Unknown dynamic discoveries remain inspection-only.

## Drive and frontend policy

Fixed drives may support reviewed executable actions. Removable and network drives are inspection-only under current policy.

The frontend submits typed scan options and IDs. It never supplies arbitrary deletion paths, executable names or shell commands. Rust resolves mutation targets from backend-owned state.

## Plan integrity

The planner resolves selected finding IDs against the active scan and stores a complete immutable plan plus hash. Execution fails when the ID/hash is unknown or mismatched, the action is no longer valid, or current filesystem evidence differs from the plan.

Receipts are historical records and cannot be replayed as authority.

## Filesystem and command validation

Cleanup must:

- validate the target against an exact root or reviewed fingerprint;
- canonicalize where appropriate;
- reject symlinks, junctions and reparse points;
- refuse protected overlaps;
- recheck project/tool evidence immediately before mutation;
- skip locked or inaccessible entries;
- keep exact cleanup roots intact when cleaning entries;
- report partial results and skips.

Tool-native adapters use `std::process::Command` with fixed executables and explicit argument arrays. WinReclaim does not build shell command strings from user, filesystem or model data.

## Undo Vault

Eligible user-temp and crash-dump cleanup can use the local manifest-backed vault.

- Original relative paths are preserved.
- Restore remains within validated roots.
- Existing destination files are never overwritten.
- Entries expire after the documented retention period.
- Corrupt or missing payloads fail safely.
- NTFS compression is used where available.
- Receipts report measured net reclaim, not the original moved size.

Vault cleanup is reversible only while the valid payload remains.

## Estimates and measured results

Before execution, projected values are estimates. After execution, receipts record attempted actions, skips/failures, target measurements where available and free space before/after.

WinReclaim must not manufacture values that look measured.

## Optional cloud intelligence

The Storage Assistant and reclaim-by-intent use OpenRouter's `openrouter/free` router through a fixed WinReclaim Vercel proxy. They are optional and advisory.

The desktop contains no OpenRouter credential. The proxy key is stored only as `OPENROUTER_API_KEY` in Vercel.

Storage summaries send aggregate drive totals and category/risk/action counts. Intent requests send the user's sentence plus opaque candidate IDs, category, size, deterministic risk and generic consequence.

Requests exclude:

- paths and drive labels;
- usernames;
- folder and file names;
- project names;
- directory trees and file contents;
- commands and action definitions;
- provider credentials.

Remote output cannot:

- add a cleanup target;
- change measured values, risk or action availability;
- create or modify a plan;
- run a command;
- delete or restore data;
- override protected policy.

The proxy fixes the model, accepts only two known tasks, validates request sizes/shapes, requires JSON Schema output and validates responses. Rust validates the result again. Unknown IDs, unsupported classes and cleanup claims are rejected.

A cloud or free-router failure cannot disable manual scanning, review, planning, cleanup, history, receipts or restore.

## Updates and credentials

Official updater artifacts are signed and verified using the public key embedded in `tauri.conf.json`.

The updater private key and `OPENROUTER_API_KEY` must never be committed, embedded in installers, attached to releases, exposed to the webview or printed in logs.

Version 1.2.1 removes the retired local Qwen/`llama.cpp` assistant directory during startup. Current releases do not bundle or download model files.

## Reset safety

- Clear scan history removes snapshots only.
- Clear cleanup records removes receipts only.
- Normal reset preserves vault restore payloads unless explicitly included.
- Deleting receipts does not undo cleanup.

## Required safety review

Explicit review is required for changes that:

- add or broaden a cleanup adapter or allowed root;
- change protected precedence, plan hashing or execution state;
- follow links or alter vault restoration;
- introduce elevation;
- add network fields, endpoints, telemetry or providers;
- change proxy credential handling or transmitted metadata;
- change updater keys, signatures or endpoints;
- add plugin/community execution capability;
- change persisted recovery schemas.

Such changes require success and refusal-path tests plus updates to [threat-model.md](threat-model.md).

## Related documentation

- [Threat model](threat-model.md)
- [Architecture](architecture.md)
- [Privacy](privacy.md)
- [Storage Assistant](storage-assistant.md)
- [Testing](testing.md)
- [Security policy](../SECURITY.md)