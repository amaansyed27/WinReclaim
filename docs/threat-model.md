# Threat Model

WinReclaim inspects local storage and can remove a limited set of files. The principal security objective is to prevent a benign scan result, compromised webview, malicious filename or stale filesystem state from becoming arbitrary deletion authority.

## Security objectives

WinReclaim should:

- remove only data authorized by a compiled and reviewed action;
- keep protected data non-actionable;
- prevent path traversal and link-based escape;
- preserve plan integrity between review and execution;
- avoid shell injection;
- protect updater and optional-download integrity;
- avoid leaking sensitive filesystem metadata;
- preserve recoverability when an action is labelled reversible;
- report partial failure rather than hiding it.

## Assets

Protected assets include:

- user documents, source code and repositories;
- credentials, browser profiles and extensions;
- local AI models and model configuration;
- Android SDK packages and virtual devices;
- Docker volumes and persistent container data;
- Windows system files and installed applications;
- WinReclaim vault payloads;
- updater private signing key;
- OpenAI API key;
- integrity of snapshots, plans, receipts and update metadata.

## Trust boundaries

### Webview to Rust backend

The React frontend is less trusted than the Rust execution layer. The frontend may be buggy or manipulated and therefore cannot supply arbitrary paths or commands.

Controls:

- typed Tauri commands;
- backend-owned scan state;
- stable finding and plan IDs;
- backend path resolution;
- immutable plan hashes;
- revalidation before mutation.

### Scan time to execution time

Filesystem state can change after scanning. A directory can be replaced, moved or converted into a junction before execution.

Controls:

- canonicalization immediately before action;
- reparse-point and symlink refusal;
- exact-root or fingerprint validation at execution;
- protected-path checks applied again;
- safe failure when evidence is stale.

### WinReclaim to external commands

Some cleanup adapters invoke tool-native commands.

Controls:

- fixed executable resolution;
- explicit argument arrays;
- no shell command construction;
- no user-controlled executable or flags;
- documented irreversible effects;
- exit-code and output handling.

### Desktop application to network services

Update checks, optional artifact downloads and optional OpenAI requests cross the local-machine boundary.

Controls:

- minimal documented endpoints;
- signed Tauri updater artifacts;
- pinned model/runtime sources;
- SHA-256 verification;
- archive extraction path validation;
- no telemetry;
- anonymized constrained OpenAI request schema;
- local core functionality remains available when offline.

### Local assistant prompt to sidecar process

Folder names can contain malicious prompt text, and the sidecar is a downloaded executable.

Controls:

- verified pinned runtime artifact;
- model and runtime manifest;
- paths treated as untrusted prompt data;
- structured, bounded output validation;
- finding-ID allowlist;
- rejection of cleanup claims;
- advisory-only output;
- process timeout and constrained arguments.

## Threats and mitigations

### Arbitrary path deletion

**Threat:** A frontend request or malformed finding causes deletion outside an intended root.

**Mitigations:** The frontend sends IDs, Rust resolves paths, actions enforce roots/fingerprints, protected policy overrides action availability, and execution revalidates current paths.

### Path traversal in persisted data

**Threat:** A malicious vault manifest, archive entry or snapshot contains `..`, absolute paths or alternate prefixes.

**Mitigations:** Use enclosed archive paths, reject absolute/escaping components, store backend-generated manifests and validate restore destinations.

### Reparse-point race

**Threat:** An allowed directory is replaced with a junction after scan.

**Mitigations:** Inspect metadata and canonical paths immediately before mutation, refuse links/reparse points and fail closed when identity changes.

Residual risk remains for complex filesystem races; high-risk actions should be narrow and entry-by-entry.

### Shell and command injection

**Threat:** A path or label is interpolated into PowerShell, CMD or a shell command.

**Mitigations:** Use `std::process::Command` with fixed executables and explicit arguments. Rules cannot contain command strings.

### Plan tampering

**Threat:** The frontend changes selected actions or paths after user confirmation.

**Mitigations:** The backend stores an immutable plan and requires its ID and complete hash for execution.

### Protected-data misclassification

**Threat:** A broad rule identifies a project, model store or profile as cache.

**Mitigations:** Protected precedence, owner/project evidence requirements, inspection-only unknown discoveries, rule tests and execution-time fingerprint validation.

### Misleading estimates

**Threat:** Users authorize deletion based on fabricated or stale reclaim values.

**Mitigations:** Label projections as estimates, measure free space before/after, report skips and keep runtime-data integrity checks.

### Vault overwrite or escape

**Threat:** Restore overwrites a new file or writes outside the original location.

**Mitigations:** Preserve validated relative paths, reject traversal and never overwrite existing destinations.

### Malicious update

**Threat:** A compromised endpoint serves an unauthorized installer.

**Mitigations:** Tauri verifies updater signatures using the embedded public key. The private key is held outside the repository.

A compromised signing key remains a critical residual risk; offline backup and strict key custody are required.

### Malicious optional artifact

**Threat:** The model or runtime provider serves a replaced binary/model.

**Mitigations:** Pin immutable revisions/tags, verify expected hashes and refuse use when the manifest or file differs.

### Prompt injection

**Threat:** A filename instructs the optional model to claim deletion is safe or change a risk class.

**Mitigations:** Treat paths as untrusted data, constrain prompt/output, validate IDs and allowed fields, reject deletion language and never use model output to create or execute actions.

### Sensitive-data exfiltration

**Threat:** Paths, project names or API keys are sent to a network service or logs.

**Mitigations:** Local-first design, documented minimal OpenAI schema, API key in the Rust environment only, no telemetry and redaction guidance.

## Assumptions

The current model assumes:

- the Windows user account and installed WinReclaim binary are not already fully compromised;
- the user can authorize cleanup actions presented by the application;
- the underlying filesystem and Windows APIs behave according to documented semantics;
- official releases are obtained from the configured GitHub repository;
- the embedded public updater key is authentic;
- local administrators can always modify application files and state.

WinReclaim is not a sandbox or endpoint-security product and cannot protect against a hostile local administrator.

## Out of scope

- registry cleaning;
- malware removal;
- secure erasure guarantees;
- recovery of arbitrary data removed by other programs;
- kernel-level attribution of file growth;
- defending a fully compromised operating system;
- third-party service availability;
- Authenticode reputation and SmartScreen policy.

## Security review triggers

Update this threat model when a change:

- adds an executable cleanup adapter;
- broadens an allowed root;
- adds network access or telemetry;
- changes plan hashing or persisted schemas;
- changes vault restoration;
- introduces elevated privileges;
- changes updater signing or endpoints;
- adds a model/runtime provider;
- permits plugins or community rules;
- follows links or reparse points;
- exposes a new Tauri mutation command.

See [SECURITY.md](../SECURITY.md) for reporting and [safety.md](safety.md) for user-facing protections.
