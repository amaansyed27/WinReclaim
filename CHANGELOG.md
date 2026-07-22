# Changelog

All notable user-facing changes to WinReclaim are documented here. The project follows semantic versioning for published Windows releases.

## [Unreleased]

### Documentation

- Added a complete developer documentation index.
- Added contribution, security, support, governance and conduct policies.
- Added architecture, command API, data lifecycle, privacy, threat-model, testing, rule-authoring, release, troubleshooting and licensing guidance.
- Added GitHub issue and pull-request templates.

## [1.2.5] - 2026-07-22

### Security

- Replaced the updater public key after the previous private signing key was lost.
- Establishes a new signed-update trust line; existing installations must install 1.2.5 manually once before automatic updates resume.

## [1.2.4] - 2026-07-22

### Changed

- Redesigned storage Review as a focused tabbed workspace with Overview, Recommended, Optional, Inspect and Assistant sections.
- Flattened nested panels into desktop-style tabs, section dividers and result rows while preserving selection, filtering and cleanup planning.

## [1.2.3] - 2026-07-22

### Fixed

- Builds the packaged Windows executable with the GUI subsystem so launching WinReclaim no longer opens a stray Windows Terminal or console window.
- Keeps development builds attached to PowerShell so Rust and Tauri diagnostics remain visible during local development.

## [1.2.2] - 2026-07-22

### Fixed

- Retries cloud assistance once with a compatibility prompt when strict structured output is unavailable.
- Accepts valid JSON wrapped in Markdown fences or short explanatory text, while retaining exact response validation.
- Returns a conservative deterministic summary or safe-now intent default when OpenRouter is unavailable, rate-limited, misconfigured or returns malformed output.
- Prevents optional assistant outages from surfacing as 503 or invalid-JSON failures for otherwise valid scan requests.

## [1.2.1] - 2026-07-22

### Added

- Added a server-side Vercel proxy for optional Storage Assistant and reclaim-by-intent requests.
- Added OpenRouter's `openrouter/free` router with strict structured-output schemas and routed-model disclosure.
- Added automatic startup migration that removes the retired local Storage Assistant model and runtime directory.

### Changed

- Replaced the Qwen GGUF and `llama.cpp` local assistant with an on-demand cloud explanation layer; the base installer no longer downloads, installs or manages model files.
- Judges and users can test cloud assistance without entering an API key because the provider credential remains a server-side Vercel environment secret.
- Restricted storage-summary requests to aggregate drive totals and category, risk and action-count metadata.
- Restricted reclaim-by-intent requests to the user's sentence plus opaque candidate IDs, category, size, deterministic risk and recovery consequence.
- Redesigned the storage review screen with clearer drive metrics, readable storage categories, stronger cleanup hierarchy and larger finding text.
- Placed recommended cleanup and optional cloud analysis in a focused action area instead of stacking oversized full-width panels.
- Replaced the content-obscuring sticky footer with a normal review action bar.
- Added explicit cloud-analysis loading, ready, success and failure states.

### Fixed

- Assistant failures now provide a retry action and collapsible technical details without interrupting the deterministic scan report.
- Cloud output is constrained and validated before display; unsafe cleanup claims are rejected.

### Security

- Provider API keys are no longer accepted or stored by the desktop application and are never embedded in source, installers or the webview.
- The proxy accepts only fixed tasks, validates request and response shapes, caps payloads and output, and prevents client-selected models, tools or provider options.
- Paths, drive labels, usernames, folder names, project names, directory trees and file contents stay local.

## [1.2.0] - 2026-07-22

### Added

- Multi-drive selection and scan reporting.
- Quick, Balanced, Deep and Ultra scan profiles.
- Storage Time Machine snapshots and compatible scan deltas.
- Reclaim Passports with ownership, evidence and recovery context.
- Reclaim Simulation before cleanup execution.
- Compressed Safe Undo Vault with manifest-backed restoration.
- App-data controls for history, receipts and reset behaviour.
- Optional local Storage Assistant using a pinned Qwen3.5-2B GGUF model and verified `llama.cpp` Windows CPU sidecar.
- Broader deterministic developer-tool, cache, project-output and Windows storage discovery.
- Static Vercel-ready product landing page with latest-release download resolution.
- Consolidated Windows release workflow for NSIS, MSI, signatures and `latest.json`.

### Changed

- Normal development builds no longer compile embedded `llama.cpp` bindings; the optional runtime is downloaded only when the Storage Assistant is installed.
- Findings and cleanup language distinguish measured results, estimates, recovery classes and action authority more clearly.
- Scan comparison requires compatible roots, profile, enabled categories, thresholds, schema and rule-set version.
- Generic cache and project-output cleanup requires deterministic fingerprints and execution-time revalidation.
- Application reset preserves optional model files and, by default, vault restore payloads.

### Security

- Rotated the Tauri updater key before publishing the new signed release line.
- Enforced signed updater artifacts and verified optional model/runtime downloads.
- Preserved protected-path precedence and inspection-only behaviour for unknown discoveries.
- Kept optional remote and local AI features advisory with no cleanup execution authority.

## [1.1.0] - 2026-07-19

### Changed

- Synchronized the application version and Windows release metadata for the previous release line.

## [1.0.0]

### Added

- Initial Windows release with local-first scanning, deterministic findings, reviewed cleanup plans, receipts and signed-update foundations.

## Changelog policy

- Add meaningful changes under **Unreleased** in pull requests.
- Move entries into a versioned section when the release is published.
- Use **Added**, **Changed**, **Deprecated**, **Removed**, **Fixed** and **Security** headings when applicable.
- Do not describe a feature as shipped until it is present in the published installer.
- GitHub Release notes may provide additional installation-specific detail.
