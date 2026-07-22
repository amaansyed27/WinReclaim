# Threat Model

WinReclaim inspects local storage and can remove a limited set of files. The principal security objective is to prevent a benign scan result, compromised webview, malicious filename, remote model output or stale filesystem state from becoming arbitrary deletion authority.

## Security objectives

WinReclaim should:

- remove only data authorized by a compiled and reviewed action;
- keep protected data non-actionable;
- prevent path traversal and link-based escape;
- preserve plan integrity between review and execution;
- avoid shell injection;
- protect updater integrity;
- keep provider credentials outside distributed clients;
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
- server-side OpenRouter API key;
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

### Desktop application to WinReclaim proxy

Optional assistant requests cross the local-machine boundary only after explicit user action.

Controls:

- one fixed HTTPS endpoint by default;
- bounded timeout and request sizes;
- fixed task names;
- Rust-built aggregate/anonymized payloads;
- no paths, drive labels, usernames, folder names, project names, directory trees or file contents;
- no provider credential in the desktop app;
- local core functionality remains available offline.

### WinReclaim proxy to OpenRouter

The Vercel serverless function holds the provider key and calls `openrouter/free`.

Controls:

- `OPENROUTER_API_KEY` stored only as a Vercel environment secret;
- fixed model router and fixed task-specific prompts;
- strict JSON Schema output;
- `require_parameters: true` routing constraint;
- bounded candidates, categories and output tokens;
- input and output validation;
- client cannot select a model, tool, arbitrary prompt or provider option;
- bounded error responses;
- best-effort per-IP demo throttling;
- separate provider-side usage/spending guardrail and key rotation.

Residual risk: an internet client can spoof the public desktop header, and in-memory serverless throttling is not globally authoritative. The demo key must therefore have a strict provider-side limit and should be rotated after judging.

### Remote output to Rust selector/presenter

Remote output is untrusted advisory data.

Controls:

- proxy validates structured output before return;
- Rust validates summary length, observation count, IDs and risk classes again;
- unknown candidate IDs are rejected;
- cleanup claims are rejected;
- remote output cannot create paths, actions or plans;
- deterministic risk/action fields remain authoritative;
- failures leave the scan report usable.

## Threats and mitigations

### Arbitrary path deletion

**Threat:** A frontend request or malformed finding causes deletion outside an intended root.

**Mitigations:** The frontend sends IDs, Rust resolves paths, actions enforce roots/fingerprints, protected policy overrides action availability, and execution revalidates current paths.

### Path traversal in persisted data

**Threat:** A malicious vault manifest or snapshot contains `..`, absolute paths or alternate prefixes.

**Mitigations:** Reject absolute/escaping components, store backend-generated manifests and validate restore destinations.

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

**Mitigations:** Label projections as estimates, measure free space before/after, report skips and keep runtime-data integrity checks. Model summaries cannot replace measurements.

### Vault overwrite or escape

**Threat:** Restore overwrites a new file or writes outside the original location.

**Mitigations:** Preserve validated relative paths, reject traversal and never overwrite existing destinations.

### Malicious update

**Threat:** A compromised endpoint serves an unauthorized installer.

**Mitigations:** Tauri verifies updater signatures using the embedded public key. The private key is held outside the repository.

A compromised signing key remains a critical residual risk; offline backup and strict key custody are required.

### Embedded provider-key extraction

**Threat:** A reusable OpenRouter key is compiled into Rust, bundled in frontend JavaScript, placed in Tauri configuration or shipped in an environment file.

**Mitigations:** The desktop contains only the public proxy URL. The provider key exists only in Vercel's server-side environment. Integrity checks reject committed key patterns.

### Proxy abuse and quota exhaustion

**Threat:** Third parties call the public proxy repeatedly and exhaust free-model capacity or account limits.

**Mitigations:** Fixed low-capability tasks, bounded payloads, best-effort IP throttling, provider-side usage limits, dedicated demo key, monitoring and key rotation. No paid fallback should be enabled for the judging key unless intentionally budgeted.

### Prompt injection

**Threat:** User intent text or category labels instruct the model to alter safety policy or generate commands.

**Mitigations:** Fixed system prompts, structured schemas, no paths/names in payloads, allowlisted risk classes, allowlisted candidate IDs, rejection of cleanup language and no model authority over actions.

### Sensitive-data exfiltration

**Threat:** Paths, project names or API keys are sent to a network service or logs.

**Mitigations:** Aggregate payload construction in Rust, generic consequence classes, server-side-only provider key, no telemetry, bounded errors and redaction guidance. Production proxy diagnostics must not log full request bodies.

### Malicious or incorrect routed model

**Threat:** The free router selects a weak or adversarial model that returns unsafe or fabricated output.

**Mitigations:** The model receives no execution authority; strict schemas and dual validation bound accepted fields; routed model identity is displayed; unsafe output is rejected; deterministic UI remains available.

## Assumptions

The current model assumes:

- the Windows user account and installed WinReclaim binary are not already fully compromised;
- the user can authorize cleanup actions presented by the application;
- the underlying filesystem and Windows APIs behave according to documented semantics;
- official releases are obtained from the configured GitHub repository;
- the embedded public updater key is authentic;
- Vercel correctly protects configured environment secrets;
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
- changes proxy tasks, transmitted fields, model routing or provider credentials;
- permits plugins or community rules;
- follows links or reparse points;
- exposes a new Tauri mutation command.

See [SECURITY.md](../SECURITY.md) for reporting and [safety.md](safety.md) for user-facing protections.
