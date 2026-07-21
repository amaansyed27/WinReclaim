# Safety Model

WinReclaim is designed to explain storage broadly while executing cleanup narrowly. The safety model is enforced in Rust and does not depend on UI wording or model judgment.

## Safety classes

Every finding has a safety classification independent from whether an executable adapter exists.

### Safe now

Narrowly scoped disposable data with a well-understood owner and consequence. Reversible handling is preferred when practical.

### Rebuild or redownload

Data that a tool can recreate, but whose removal can cost time, bandwidth or compute. Examples include package caches and verified generated project outputs.

These actions are not silently treated as consequence-free and should not be selected by default merely because they are reproducible.

### Review first

Data whose removal may disrupt environments, containers, emulators, launch performance or developer workflows. It requires explicit manual review and clear consequence text.

### Protected

Data that WinReclaim refuses to add to an executable cleanup plan. Protected classification always overrides a less restrictive rule or attached action kind.

## Non-negotiable protections

WinReclaim does not automatically remove:

- registry data;
- browser profiles or extensions;
- local AI model stores, including Ollama models;
- Docker volumes;
- Android virtual devices or SDK packages by raw folder deletion;
- project source or repositories;
- credentials and configuration;
- Program Files;
- arbitrary Windows directories;
- WinReclaim snapshots, receipts, vault payloads or model files;
- unknown folders discovered only by size or a generic name.

Windows Prefetch is exposed only as a manually selected **Review first** action, restricted to `.pf` entries under the exact active Prefetch root and accompanied by a rebuild/performance consequence.

## Detection is not deletion authority

A scanner can discover a large directory without knowing that it is safe to remove. Detection rules can explain and classify storage, but execution is available only when:

1. deterministic evidence identifies the target;
2. policy does not mark it protected;
3. a compiled cleanup adapter exists;
4. the finding is actionable on the current drive type;
5. the user selects it;
6. the backend creates and hashes a plan;
7. the user confirms the plan;
8. execution-time validation still succeeds.

Unknown dynamic discoveries remain inspection-only.

## Drive policy

Fixed drives can support reviewed executable actions. Removable and network drives are inspection-only under the current policy because identity, availability, filesystem semantics and recovery guarantees differ.

The UI must not imply that an inspection-only result can be enabled through an advanced override.

## Frontend authority boundary

The frontend submits:

- scan options;
- scan IDs;
- finding IDs;
- plan IDs and hashes;
- vault entry IDs;
- explicit settings choices.

It does not submit arbitrary deletion paths, executable names or shell commands.

Rust resolves all mutation targets from backend-owned state.

## Plan integrity

The planner resolves selected finding IDs against the current scan and creates an immutable plan containing actions, consequences and estimates. The complete plan is hashed and stored in backend state.

Execution fails when:

- the plan ID is unknown;
- the supplied hash does not match;
- the finding or action is no longer valid;
- current filesystem evidence differs from the plan's assumptions.

A receipt is not an executable plan and cannot be replayed.

## Filesystem validation

Filesystem cleanup must:

- validate the current target against an exact allowed root or reviewed fingerprint;
- use canonical paths where appropriate;
- reject symbolic links, junctions and reparse points;
- avoid following links during scanning and mutation;
- refuse protected overlaps;
- revalidate project/tool evidence immediately before mutation;
- skip locked or inaccessible entries instead of forcing deletion;
- keep the allowed root itself intact for entry-by-entry cleanup;
- report partial results and skipped entries.

A scan result is stale by definition at execution time; current validation is mandatory.

## External command safety

Tool-native adapters use `std::process::Command` with a fixed executable and explicit argument array. WinReclaim does not construct shell command strings from user or scan data.

External commands must have:

- documented scope;
- fixed or safely resolved executable identity;
- fixed reviewed flags;
- exit-code handling;
- irreversible-consequence text when applicable;
- no hidden volume or project deletion.

Docker volumes are never included in the current Docker action.

## Undo Vault

Eligible user-temp and crash-dump cleanup uses a local manifest-backed vault.

Safety properties:

- original relative paths are preserved;
- restore stays within the validated destination root;
- existing destination files are never overwritten;
- entries have an explicit expiry;
- missing or corrupt payloads fail safely;
- native NTFS compression is used where available;
- the receipt reports measured net reclaim, not the original moved size.

Vault-backed cleanup is reversible only while a valid payload remains within its retention period.

## Estimates and measured results

Before execution, WinReclaim shows projected values. They are estimates based on current scan state.

After execution, the receipt records:

- actions attempted;
- actions skipped or failed;
- target measurements where available;
- free space before and after;
- measured net change.

The application must not manufacture values that look measured. No baseline means no delta; no measured recovery time means no recovery-time estimate.

## Optional OpenAI intent feature

GPT-5.6 is optional and has no deletion authority.

The request is limited to anonymized metadata for currently executable candidates. The model may return:

- target reclaim size;
- allowed safety classes;
- explicit candidate exclusions;
- short explanation.

The schema does not include paths, commands or action kinds. Rust validates candidate IDs and classes, applies deterministic selection and presents the result as an editable suggestion. The normal plan and confirmation flow still applies.

A network or model failure cannot disable manual planning.

## Optional local Storage Assistant

The local assistant summarizes completed deterministic scan data and may propose labels or presentation groups.

It cannot:

- calculate authoritative sizes;
- change risk classes;
- change action availability;
- select findings;
- create plans;
- execute actions;
- override protected data.

The model and `llama.cpp` sidecar are pinned and verified. Generated annotations are validated against current finding IDs. Deletion claims, unknown groups and unsupported IDs are discarded.

Folder names are treated as untrusted prompt data.

## Updates and artifact integrity

Official updater artifacts are signed. The installed application verifies updates using the public key embedded in `tauri.conf.json`.

The updater private key must never be committed or attached to a release. Losing the key prevents trusted updates to existing installations; leaking it allows malicious updates to be signed.

Optional model/runtime downloads use pinned provenance and integrity verification. Verification must not be bypassed as a troubleshooting shortcut.

## Reset safety

Data controls are separated by consequence:

- clearing scan history removes snapshots only;
- clearing cleanup records removes receipts only;
- normal reset preserves optional model files;
- normal reset preserves vault restore payloads unless the user explicitly includes them;
- uninstalling the Storage Assistant removes only its model/runtime directory.

UI wording must make clear that deleting receipts does not undo cleanup.

## Required review for safety-sensitive changes

A change requires explicit safety review when it:

- adds or broadens a cleanup adapter;
- changes protected roots or safety precedence;
- changes plan hashing or execution state;
- follows links or reparse points;
- changes vault restoration;
- introduces elevation;
- adds a network endpoint or telemetry;
- changes updater keys, signatures or endpoints;
- changes model/runtime download verification;
- adds plugin or community execution capability;
- changes persisted schemas that affect recovery.

Such changes require success and refusal-path tests plus updates to [threat-model.md](threat-model.md).

## Related documentation

- [Threat model](threat-model.md)
- [Architecture](architecture.md)
- [Rule authoring](rule-authoring.md)
- [Testing](testing.md)
- [Security policy](../SECURITY.md)
