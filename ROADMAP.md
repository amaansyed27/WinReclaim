# Roadmap

This roadmap describes direction, not guaranteed delivery dates. Safety and release quality take priority over feature count.

## Current product foundation

WinReclaim currently focuses on:

- Windows 11 x64;
- deterministic multi-drive scanning;
- explainable findings and recovery classes;
- Storage Time Machine and Reclaim Passports;
- immutable cleanup planning and simulation;
- allowlisted cleanup adapters;
- compressed Undo Vault and measured receipts;
- signed Windows releases;
- optional advisory-only remote and local intelligence.

## Near-term priorities

### Release reliability

- harden the consolidated Windows release workflow;
- validate clean NSIS and MSI installation paths;
- test signed in-app updates across consecutive versions;
- improve release artifact and manifest verification;
- document reproducible release inputs where practical.

### Scanner accuracy and performance

- improve error and skip visibility;
- reduce repeated sizing work;
- improve selected-drive progress reporting;
- expand deterministic tool-specific evidence;
- benchmark Deep and Ultra profiles on large developer machines.

### Safety and recovery

- expand refusal-path tests;
- improve vault expiry and partial-restore reporting;
- strengthen time-of-check/time-of-use resistance;
- add clearer per-action recovery guidance;
- continue auditing protected-root precedence.

### User experience

- improve explanations for inspection-only findings;
- make scan compatibility and timeline baselines clearer;
- improve keyboard and screen-reader navigation;
- refine installer/update failure messages;
- keep advanced controls available without overwhelming normal users.

### Storage Assistant

- maintain pinned model/runtime provenance;
- build the fixed anonymized evaluation suite;
- measure CPU and memory performance on lower-end systems;
- consider a WinReclaim-specific adapter only after passing the evaluation gate;
- preserve advisory-only authority.

## Medium-term exploration

### Faster scanner backends

Potential backends include:

- NTFS Master File Table enumeration for initial inventory;
- USN Change Journal indexing for incremental refresh;
- tool-native metadata for exact size and last-use context.

Any new backend must preserve the existing rule, policy, planner and execution boundaries.

### Expanded deterministic coverage

Possible additions:

- more package-manager caches;
- more build systems;
- additional verified browser/runtime caches;
- more Windows diagnostics and servicing locations;
- clearer application ownership attribution.

New detections can remain inspection-only until a safe adapter exists.

### Distribution and trust

Possible work:

- Windows Authenticode signing when sustainable;
- stronger release provenance/attestations;
- improved update rollback guidance;
- package-manager distribution after installer stability is proven.

## Explicit non-goals

The project does not plan to become:

- a registry cleaner;
- a Windows debloater;
- a generic performance booster;
- an arbitrary-path deletion tool;
- a background telemetry agent;
- an AI agent with deletion authority;
- a secure-erasure product;
- a replacement for tool-native project or environment managers.

## How roadmap decisions are made

Proposals are evaluated against:

1. risk of unintended data loss;
2. deterministic evidence quality;
3. recovery consequence;
4. privacy/network impact;
5. maintainability and testability;
6. value to Windows users;
7. release and support cost.

See [GOVERNANCE.md](GOVERNANCE.md) and [CONTRIBUTING.md](CONTRIBUTING.md).
