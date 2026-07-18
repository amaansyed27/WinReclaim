# Rule system

Rules describe storage; they do not delete it.

A rule provides a stable ID, product name, recognised path, safety classification, explanation, consequence, confidence and optional reference to a compiled action adapter.

## Safety precedence

`protected` always overrides every less restrictive classification. Protected findings cannot expose an executable action even if a programmer accidentally attaches an action kind.

## Community rules

A future community rule format may add new detections and consequence text. It must never include PowerShell, executable paths, command arguments or arbitrary deletion globs. New executable adapters require code review, tests and a signed application release.
