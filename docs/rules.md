# Rule System

WinReclaim rules turn filesystem evidence into semantic findings. Detection never grants arbitrary deletion authority by itself.

A rule provides a stable ID, product/owner name, category, recognized location or fingerprint, safety classification, explanation, consequence, confidence and optional reference to a compiled action adapter.

## Rule pipeline

```text
scanner candidate
  → deterministic rule match
  → protected-policy precedence
  → semantic finding
  → optional action availability
  → user review
  → backend plan resolution
  → execution-time revalidation
```

A finding can remain visible when it is inspection-only. This is expected for uncertain, removable, network, protected and unsupported data.

## Rule identity

Rule IDs are persisted into findings and can influence snapshots, timeline comparisons and receipts. Treat them as data-schema identifiers rather than display strings.

A stable rule ID should survive wording and implementation changes. Renaming or reusing an ID for different semantics requires compatibility review.

## Generic product shell

The WinReclaim interface must remain useful to any Windows user. It must not advertise one developer's username, projects, installed tools or folder layout.

Specific product names are shown only when:

- a deterministic rule found the product or location on the current PC;
- the name is part of that finding, evidence or consequence;
- the finding disappears when the software/location is absent.

Default scans focus on common cleanup locations. Project-output, broad AppData and unknown large-folder analysis require deeper profiles or explicit options.

## Evidence hierarchy

Rules should use the strongest available evidence:

1. exact documented location;
2. expected parent plus owner-specific marker;
3. project manifest plus generated-directory relationship;
4. bounded portable cache fingerprint within a user-owned root;
5. generic name alone.

The last level is normally insufficient for executable cleanup.

## Windows temporary locations

`%TEMP%`, the active Windows `Temp` directory and Windows Prefetch are measured as complete known roots. WinReclaim does not hide recent files behind an arbitrary display-only age threshold.

Cleanup is performed entry by entry:

- eligible files and empty subdirectories are processed;
- active, locked, inaccessible, reparse-point and administrator-protected entries are skipped;
- the root directory itself is never removed;
- receipts report measured size and skipped entries.

User Temp and Windows Temp are recognized cleanup locations. Prefetch is optional, never selected automatically and warns that Windows rebuilds launch traces and launches can temporarily be slower.

## Rebuildable and redownloadable data

Known package caches and generated outputs may use a compiled generic-directory adapter when evidence is strong enough.

Supported patterns can include portable locations such as:

- Bun cache;
- Gradle cache and wrapper distributions;
- Cargo cache;
- pip and uv caches;
- Playwright browser binaries;
- verified application caches;
- generated project outputs.

Project outputs are actionable only with nearby project evidence:

- `node_modules` requires a JavaScript manifest or lockfile;
- Rust `target` requires `Cargo.toml`;
- Python `.venv` or `venv` requires `pyvenv.cfg`;
- common build-output directories require a recognized project manifest.

Project roots and source are never removed as a substitute for generated output cleanup.

## Dynamic cache fingerprints

Deep discovery can classify bounded user-profile directories using portable cache fingerprints such as:

```text
Cache
caches
GPUCache
ComputeCache
shader-cache
tmp
node-compile-cache
```

These findings are optional and off in shallower profiles. A cache-like name is only one part of policy.

Generic cleanup is refused when a target:

- is outside the current user profile or allowed root;
- is a reparse point, junction or symbolic link;
- overlaps a protected or review-only known location;
- contains protected components associated with models, profiles, extensions, repositories, credentials or WinReclaim state;
- no longer matches the scan-time fingerprint;
- is on a drive type that is inspection-only;
- cannot be canonicalized/validated safely.

The executor revalidates immediately before deletion.

## Tool-native adapters

Some targets are safer to manage through their owning tool than raw filesystem deletion.

Examples include:

- Hugging Face cache prune;
- npm cache clean;
- conservative Docker prune.

Tool-native actions use fixed executables and explicit argument arrays. Rules do not contain command text. Consequences must identify data that is irreversible or requires redownload.

Docker volumes are excluded.

## Protected precedence

`protected` always overrides every less restrictive classification. A programming mistake that attaches an action kind to protected data must not make the finding executable.

Protected examples include:

- source repositories and user projects;
- browser profiles and extensions;
- model stores and model files;
- Docker volumes;
- Android virtual devices and SDK packages;
- credentials/configuration;
- arbitrary system/application roots;
- WinReclaim's own snapshots, receipts, vault and models.

## Runtime-data integrity

The application shell must not manufacture values that appear measured.

In particular:

- history never substitutes zero when no comparable baseline exists;
- scans compare only when roots, mode, categories, thresholds, schema and rule-set version match;
- confidence is qualitative unless backed by a real measured score;
- recovery time is not estimated without a measurement model;
- restore availability is read from the persisted vault entry and expiry;
- receipts contain actual action results and measured free-space change;
- a static list of protected products is not presented as runtime detection.

Persisted snapshot schemas are versioned. Older or incompatible snapshots can remain visible without being used as a delta baseline.

## Selection policy

Safe, rebuildable and review-first actions remain distinct.

- Protected and inspection-only findings cannot be selected.
- Rebuildable actions are not treated as “free.”
- Prefetch is never selected by default.
- Unknown discoveries do not become actionable through the optional AI features.
- Remote intent output is validated against currently executable finding IDs.
- Local Storage Assistant output cannot alter selection or action fields.

## Community rules

A future community rule format may add new detection metadata and consequence text, but it must never include:

- PowerShell or shell scripts;
- executable paths and arbitrary arguments;
- arbitrary deletion globs;
- frontend-provided paths;
- code loaded without review/signing.

New executable adapters require source review, tests, threat-model updates and a signed application release.

## Adding a rule

Follow [rule-authoring.md](rule-authoring.md). A rule contribution should include:

- deterministic evidence;
- positive and negative tests;
- protected-overlap tests;
- consequence and recovery wording;
- execution-time revalidation when actionable;
- documentation/changelog updates.

When evidence is uncertain, prefer an informative inspection-only finding.

## Related documentation

- [Rule authoring](rule-authoring.md)
- [Safety model](safety.md)
- [Threat model](threat-model.md)
- [Architecture](architecture.md)
- [Testing](testing.md)
