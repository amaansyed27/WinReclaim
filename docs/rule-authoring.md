# Rule Authoring Guide

WinReclaim rules convert filesystem evidence into understandable findings. A rule may describe storage without granting permission to delete it.

## Core principle

**Detection and execution are separate authorities.**

A folder can be detected and classified while remaining inspection-only. Executable cleanup requires a compiled Rust adapter, current-path validation and an allowed safety classification.

## Rule responsibilities

A rule should define:

- a stable identifier;
- owner or product label;
- category;
- recognized path shape or deterministic fingerprint;
- explanation of why the storage exists;
- cleanup consequence;
- safety class;
- confidence grounded in evidence;
- optional reference to an existing compiled action kind.

Rules must not contain:

- PowerShell or shell code;
- arbitrary executable paths;
- command-line fragments controlled by scan data;
- unbounded deletion globs;
- user-provided cleanup paths;
- claims that a folder is disposable based only on its name.

## Stable IDs

Rule IDs are persisted in findings and can influence snapshots and receipts. Treat them as data-schema identifiers.

Good IDs are specific and stable:

```text
npm.cache.user
huggingface.hub.cache
gradle.caches.user
project.rust.target
windows.user_temp
```

Avoid IDs tied to display wording or implementation file names.

## Evidence levels

Use the strongest available deterministic evidence:

1. exact documented location for a known tool;
2. expected parent plus tool-specific marker;
3. project manifest near a generated directory;
4. portable cache fingerprint within a bounded user-owned root;
5. folder name alone.

Level 5 is normally insufficient for executable cleanup. Unknown large folders should remain inspection-only.

## Project-output rules

Generated project directories require nearby project evidence.

Examples:

- `node_modules` requires a recognized JavaScript manifest or lockfile;
- Rust `target` requires `Cargo.toml`;
- `.venv` or `venv` requires `pyvenv.cfg`;
- build output requires a recognized project manifest and bounded relationship;
- a repository or source directory is never removed as a substitute for deleting its generated output.

Revalidate the evidence at execution time because the directory may change after scanning.

## Safety classification

Use the least permissive classification supported by evidence.

### Safe now

Reserved for narrowly scoped disposable data with a well-understood owner and consequence. Reversible handling is preferred.

### Rebuild or redownload

Use when the owning tool can recreate the data, but removal may cost time, bandwidth or build work. These findings should not be silently selected as the default recommendation.

### Review first

Use when removal can disrupt environments, containers, emulators, developer state or performance. Manual selection and clear consequence text are required.

### Protected

Use when WinReclaim must refuse executable cleanup. `protected` always overrides any attached action kind.

## Protected-path review

A proposed rule must be tested against protected components and roots, including:

- user projects and repositories;
- model stores and model files;
- browser profiles and extensions;
- credentials and configuration;
- Windows system roots outside exact reviewed actions;
- Program Files;
- Android virtual devices and SDK packages;
- Docker volumes;
- WinReclaim snapshots, receipts, vault and model files;
- reparse points, junctions and symlinks.

Do not weaken a global protection to make one rule work. Narrow the new rule instead.

## Action adapters

New executable behaviour belongs under `src-tauri/src/actions`, not inside rule data.

An adapter should:

- accept backend-resolved typed input;
- validate the current canonical path;
- enforce an exact root or documented fingerprint;
- reject reparse points;
- use explicit command arguments when invoking a tool;
- avoid shell interpolation;
- measure or report the result;
- describe skipped files and partial failures;
- return a consequence appropriate for receipts;
- have success and refusal-path tests.

Prefer tool-native cleanup commands when they provide safer semantics than raw folder deletion, but document irreversible effects.

## Consequence text

Consequence text should answer:

- What will be removed?
- What will stop working, slow down or redownload?
- Is recovery automatic, manual or unavailable?
- How long is the Undo Vault entry retained when applicable?
- Are locked or protected files skipped?

Avoid vague wording such as “safe junk” or “optimizes your PC.”

## Confidence

Confidence is qualitative and evidence-based. Do not invent a percentage when the rule does not measure one.

Examples:

- **High:** exact known path plus expected owner marker.
- **Medium:** project manifest plus conventional generated-directory name.
- **Low:** bounded cache-like fingerprint without owner-specific evidence.

Low-confidence discoveries should generally remain inspection-only.

## Testing checklist

A rule change should test:

- positive match;
- similar path that must not match;
- absent owner/tool;
- protected overlap;
- link/reparse-point refusal;
- fixed versus removable/network drive policy;
- stale evidence at execution;
- size threshold and scan-profile behaviour;
- action availability;
- consequence and safety class;
- snapshot compatibility when IDs or schemas change.

## Documentation checklist

Update as relevant:

- `docs/rules.md`;
- `README.md` detection list;
- `docs/safety.md`;
- `docs/threat-model.md`;
- `CHANGELOG.md`;
- `THIRD_PARTY_NOTICES.md` for new tool dependencies.

## Review questions

Before merging, reviewers should be able to answer:

1. What deterministic evidence identifies the storage?
2. What is the worst plausible false positive?
3. Why is the selected safety class appropriate?
4. Can the path change between scan and execution?
5. What prevents path escape or protected overlap?
6. Is the consequence accurate and visible?
7. Can the action be reversed or recreated?
8. Are failure and refusal paths tested?

When the evidence is uncertain, keep the finding visible but non-actionable.
