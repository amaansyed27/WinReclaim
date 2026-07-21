# Roadmap

This roadmap describes direction, not guaranteed dates. Safety and release quality take priority over feature count.

## Current foundation

WinReclaim currently focuses on:

- Windows 11 x64;
- deterministic multi-drive scanning;
- explainable findings and recovery classes;
- Storage Time Machine and Reclaim Passports;
- immutable cleanup planning and simulation;
- allowlisted cleanup adapters;
- compressed Undo Vault and measured receipts;
- signed Windows releases;
- optional advisory cloud assistance through a privacy-bounded Vercel/OpenRouter proxy.

## Near-term priorities

### Release reliability

- validate clean NSIS/MSI installation and uninstall;
- test signed updates across consecutive versions;
- improve artifact/manifest verification;
- add stronger release provenance where practical.

### Scanner accuracy and performance

- improve error and skip visibility;
- reduce repeated sizing work;
- improve multi-drive progress;
- expand deterministic ownership evidence;
- benchmark Deep and Ultra profiles.

### Safety and recovery

- expand refusal-path tests;
- improve vault expiry and partial-restore reporting;
- strengthen time-of-check/time-of-use resistance;
- improve per-action recovery guidance;
- continue auditing protected precedence.

### User experience and accessibility

- improve explanations for inspection-only findings;
- clarify timeline compatibility/baselines;
- improve keyboard and screen-reader navigation;
- refine installer, updater and cloud-failure messages;
- preserve readable normal-user flows alongside advanced controls.

### Cloud assistant

- complete the fixed synthetic privacy/evaluation suite;
- verify that new payloads never include paths or names;
- strengthen durable abuse/rate controls beyond best-effort in-memory throttling;
- measure OpenRouter free-router availability and latency;
- provide transparent routed-model disclosure;
- keep deterministic core operation fully usable offline;
- preserve advisory-only authority and server-side credential separation.

Current releases will not reintroduce a bundled local model unless a future proposal demonstrates a clear product benefit, acceptable installer/runtime cost and equivalent security/release quality.

## Medium-term exploration

### Faster scanner backends

Potential work includes NTFS MFT enumeration, USN Change Journal indexing and tool-native metadata. New backends must retain existing rules, policy, planner and execution boundaries.

### Expanded deterministic coverage

Possible additions include more package-manager/build caches, verified browser/runtime caches, Windows diagnostics/servicing locations and stronger application ownership attribution. New detections can remain inspection-only until a safe adapter exists.

### Distribution and trust

- Windows Authenticode signing when sustainable;
- stronger attestations and reproducibility;
- clearer rollback guidance;
- package-manager distribution after installer stability is proven.

## Explicit non-goals

WinReclaim is not intended to become:

- a registry cleaner or Windows debloater;
- a generic performance booster;
- an arbitrary-path deletion tool;
- a background telemetry agent;
- an AI agent with deletion authority;
- a secure-erasure product;
- a replacement for tool-native project/environment managers.

## Decision criteria

Proposals are evaluated against:

1. unintended-data-loss risk;
2. deterministic evidence quality;
3. recovery consequence;
4. privacy, credential and network impact;
5. maintainability and testability;
6. value to Windows users;
7. release and support cost.

See [GOVERNANCE.md](GOVERNANCE.md) and [CONTRIBUTING.md](CONTRIBUTING.md).