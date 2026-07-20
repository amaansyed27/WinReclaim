# Rule system

Rules describe storage; they do not delete it.

A rule provides a stable ID, product name, recognised path, safety classification, explanation, consequence, confidence and optional reference to a compiled action adapter.

## Generic product shell

The main WinReclaim interface must remain useful to any Windows user. It must not advertise one developer's username, projects, installed tools or folder layout.

Specific application names are allowed only when:

- a detection rule has found that application or storage location on the current PC
- the name is shown as part of that finding, its explanation or its cleanup consequence
- the corresponding finding disappears when the software or folder is not present

Default scans focus on common cleanup locations. Project output, broad AppData discovery and unknown large-folder analysis are enabled only by deeper scan profiles or explicit advanced options.

## Runtime-data integrity

The application shell must not manufacture values that look measured. In particular:

- history never substitutes zero when no comparable baseline exists
- scans are compared only when their root, mode, enabled categories, thresholds and rule-set version match
- confidence is represented by the rule's qualitative classification, not an invented percentage
- recovery time is not estimated unless a future adapter measures it
- restore availability is read from each persisted vault entry and its exact expiry timestamp
- cleanup receipts contain measured before-and-after values and executed action results, not a static list of protected products
- Prefetch is inspection-only and cannot expose an executable cleanup adapter

Persisted snapshot schemas are versioned. Older or incompatible snapshots remain visible in the chart but cannot be used as a delta baseline.

## Safety precedence

`protected` always overrides every less restrictive classification. Protected findings cannot expose an executable action even if a programmer accidentally attaches an action kind.

## Community rules

A future community rule format may add new detections and consequence text. It must never include PowerShell, executable paths, command arguments or arbitrary deletion globs. New executable adapters require code review, tests and a signed application release.
