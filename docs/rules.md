# Rule system

Rules describe storage and may attach a compiled cleanup action. Detection never grants arbitrary filesystem deletion by itself.

A rule provides a stable ID, product name, recognised path, safety classification, explanation, consequence, confidence and optional reference to a compiled action adapter.

## Generic product shell

The main WinReclaim interface must remain useful to any Windows user. It must not advertise one developer's username, projects, installed tools or folder layout.

Specific application names are allowed only when:

- a detection rule has found that application or storage location on the current PC
- the name is shown as part of that finding, its explanation or its cleanup consequence
- the corresponding finding disappears when the software or folder is not present

Default scans focus on common cleanup locations. Project output, broad AppData discovery and unknown large-folder analysis are enabled only by deeper scan profiles or explicit advanced options.

## Windows temporary locations

`%TEMP%`, the active Windows `Temp` directory and Windows Prefetch are measured as complete roots. WinReclaim does not hide recent files behind an age threshold.

Cleanup is performed entry by entry:

- normal files and empty subdirectories are removed
- active, locked, inaccessible, reparse-point and administrator-protected entries are skipped
- the root directory itself is never removed
- receipts report the measured size before and after cleanup and the number of skipped entries

User Temp and Windows Temp are recommended cleanup locations. Prefetch is optional, never selected automatically, and warns that Windows will rebuild launch traces and that application launches may temporarily be slower.

## Rebuildable and redownloadable data

Known package caches and generated outputs may use the compiled `generic_directory` adapter. This includes portable locations such as Bun, Gradle, Cargo, pip, uv and Playwright caches when present.

Project outputs are actionable only when nearby filesystem evidence verifies the project type:

- `node_modules` requires a JavaScript manifest or lockfile
- Rust `target` requires `Cargo.toml`
- Python `.venv` or `venv` requires `pyvenv.cfg`
- common build-output directories require a recognised project manifest

Unknown applications are not hardcoded. Deep discovery may classify a directory through a portable cache fingerprint such as `Cache`, `caches`, `GPUCache`, `ComputeCache`, `shader-cache`, `tmp` or `node-compile-cache`. These findings are optional and off by default. The executor revalidates the path and fingerprint immediately before deletion.

Generic cleanup is refused when a target:

- is outside the current user profile
- is a reparse point
- overlaps a protected or review-only known location
- contains protected path components associated with models, profiles, extensions, repositories or WinReclaim state
- no longer matches the rule or project fingerprint used during scanning

## Runtime-data integrity

The application shell must not manufacture values that look measured. In particular:

- history never substitutes zero when no comparable baseline exists
- scans are compared only when their root, mode, enabled categories, thresholds and rule-set version match
- confidence is represented by the rule's qualitative classification, not an invented percentage
- recovery time is not estimated unless a future adapter measures it
- restore availability is read from each persisted vault entry and its exact expiry timestamp
- cleanup receipts contain measured before-and-after values and executed action results, not a static list of protected products

Persisted snapshot schemas are versioned. Older or incompatible snapshots remain visible in the chart but cannot be used as a delta baseline.

## Safety precedence

`protected` always overrides every less restrictive classification. Protected findings cannot expose an executable action even if a programmer accidentally attaches an action kind.

Safe cleanup, rebuildable cleanup and destructive review remain distinct. Rebuildable and Prefetch actions are never selected as part of the default recommendation.

## Community rules

A future community rule format may add new detections and consequence text. It must never include PowerShell, executable paths, command arguments or arbitrary deletion globs. New executable adapters require code review, tests and a signed application release.
