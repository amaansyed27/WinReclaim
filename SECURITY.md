# Security Policy

WinReclaim scans storage and can execute narrowly scoped cleanup actions. Vulnerabilities that bypass path validation, plan integrity, updater verification or protected-data boundaries are treated as high priority.

## Supported versions

Security fixes are applied to:

- the latest published stable release;
- the current `main` branch when the issue affects unreleased code.

Older releases may not receive backports. Users should keep WinReclaim updated through the signed updater or the latest GitHub Release.

## Reporting a vulnerability

Do not open a public issue containing exploit details, private paths, signing material or proof-of-concept code.

Preferred reporting method:

1. Open the repository **Security** tab.
2. Select **Report a vulnerability** to use GitHub private vulnerability reporting.
3. Include the affected version or commit, reproduction steps, expected impact and any suggested mitigation.

If private vulnerability reporting is unavailable, open a minimal public issue asking the maintainer to establish a private channel. Do not include technical details in that issue.

Never send or commit:

- `TAURI_SIGNING_PRIVATE_KEY`;
- updater private-key backups;
- OpenAI API keys;
- personal filesystem paths or scan exports containing sensitive names;
- access tokens or credentials discovered in logs.

## Useful report contents

A useful report includes:

- affected WinReclaim version or commit SHA;
- Windows version and architecture;
- whether the issue occurs during scan, planning, cleanup, restore, model installation or update;
- the smallest reproducible test case;
- whether reparse points, junctions, locked files or path races are involved;
- whether user interaction is required;
- potential impact on confidentiality, integrity or availability;
- sanitized logs with usernames and paths removed.

## Security-sensitive areas

Reports are especially valuable for:

- arbitrary path deletion or traversal;
- reparse-point, symlink or junction bypasses;
- time-of-check/time-of-use path substitution;
- protected findings becoming executable;
- cleanup-plan hash or state bypasses;
- shell or command injection;
- unsafe external command resolution;
- vault restore overwrites or path escapes;
- malicious snapshot, receipt or manifest parsing;
- model or runtime download verification bypasses;
- updater signature or endpoint bypasses;
- leakage of paths, filenames or API keys to remote services;
- prompt-injection paths that influence executable cleanup behaviour;
- privilege-boundary mistakes involving Windows system directories.

## Response process

The project uses a best-effort process:

1. acknowledge receipt;
2. reproduce and assess severity;
3. prepare a fix and regression tests;
4. coordinate disclosure when appropriate;
5. publish a signed release;
6. document impact and remediation without exposing users unnecessarily.

Complex reports may take longer to validate, particularly when Windows filesystem behaviour differs across versions.

## Security design principles

WinReclaim is designed around the following controls:

- deterministic scanning and classification;
- typed Tauri commands;
- stable IDs instead of frontend-supplied deletion paths;
- immutable hashed cleanup plans;
- compiled allowlisted action adapters;
- canonical path and root validation before mutation;
- refusal to follow reparse points;
- measured receipts;
- local-first data storage;
- signed updater artifacts;
- SHA-256 verification for optional model/runtime downloads;
- advisory-only AI components with no execution authority.

See [docs/safety.md](docs/safety.md) and [docs/threat-model.md](docs/threat-model.md) for the detailed model.

## Out of scope

The following are generally not security vulnerabilities unless they cross a trust boundary:

- inaccurate reclaim estimates that are clearly labelled estimates;
- files skipped because they are locked or inaccessible;
- SmartScreen warnings on unsigned Authenticode installers;
- denial of service requiring manual modification of WinReclaim's own local data;
- issues in unsupported Windows versions;
- social engineering unrelated to repository code or official release assets.

This policy does not promise a bug bounty or monetary reward.
