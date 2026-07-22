# Security Policy

WinReclaim scans storage and can execute narrowly scoped cleanup actions. Vulnerabilities that bypass path validation, plan integrity, updater verification, privacy boundaries or protected-data policy are high priority.

## Supported versions

Security fixes target the latest stable release and current `main` when relevant. Older releases may not receive backports. Update through the signed updater or official GitHub Release.

## Reporting a vulnerability

Do not open a public issue containing exploit details, private paths, credentials, signing material or proof-of-concept code.

Preferred method:

1. Open the repository **Security** tab.
2. Select **Report a vulnerability**.
3. Include the affected version/commit, reproduction steps, impact and suggested mitigation.

When private reporting is unavailable, open only a minimal public request for a private channel.

Never send or commit:

- `TAURI_SIGNING_PRIVATE_KEY` or backups;
- `OPENROUTER_API_KEY`;
- Vercel access tokens or environment exports;
- personal paths, raw scan exports or vault data;
- access tokens or credentials from logs.

A suspected OpenRouter-key exposure should be reported privately and the key rotated immediately.

## Useful report contents

Include:

- affected version or commit;
- Windows version/architecture;
- whether the issue affects scan, planning, cleanup, restore, cloud proxy or update;
- smallest reproducible case;
- involvement of links, junctions, locked files or races;
- whether user interaction is required;
- confidentiality/integrity/availability impact;
- sanitized logs with names and paths removed.

## Security-sensitive areas

Reports are especially valuable for:

- arbitrary deletion, traversal or reparse bypass;
- time-of-check/time-of-use substitution;
- protected findings becoming executable;
- plan-hash/state bypass;
- shell/command injection;
- unsafe external command resolution;
- vault overwrite or path escape;
- malicious persisted-data parsing;
- updater signature/endpoint bypass;
- OpenRouter or updater credential exposure;
- cloud payloads containing paths, drive labels, usernames, folder/file names, project names, directory trees or contents;
- proxy bypass allowing arbitrary models, prompts, tools or provider options;
- unknown candidate IDs/classes accepted from remote output;
- cloud output influencing executable cleanup;
- rate-limit or cost-exhaustion vulnerabilities;
- privilege mistakes involving Windows system directories.

## Response process

Best effort:

1. acknowledge;
2. reproduce and assess severity;
3. fix with regression tests;
4. coordinate disclosure when appropriate;
5. rotate affected credentials;
6. publish a signed release;
7. document impact/remediation safely.

## Security design principles

- deterministic scanning and classification;
- typed Tauri commands and backend-owned state;
- IDs instead of frontend/model-supplied deletion paths;
- immutable hashed plans;
- compiled allowlisted adapters;
- canonical/root validation before mutation;
- reparse-point refusal;
- measured receipts and no-overwrite restore;
- local-first persisted data;
- optional privacy-minimized cloud requests;
- provider credential stored only in Vercel;
- fixed proxy tasks/model router and structured-output validation;
- advisory-only remote output;
- signed updater artifacts.

Current releases do not bundle or download a local AI model.

See [docs/safety.md](docs/safety.md), [docs/privacy.md](docs/privacy.md) and [docs/threat-model.md](docs/threat-model.md).

## Out of scope

Generally not vulnerabilities unless they cross a trust boundary:

- clearly labelled estimate inaccuracies;
- locked/inaccessible files being skipped;
- temporary free-router unavailability;
- SmartScreen warnings without Authenticode reputation;
- denial of service requiring manual corruption of local WinReclaim data;
- unsupported Windows configurations;
- unrelated social engineering.

No bug bounty or monetary reward is promised.