# Changelog

All notable user-facing changes to WinReclaim are documented here. The project follows semantic versioning for published Windows releases.

## [Unreleased]

### Documentation

- Added a complete developer documentation index.
- Added contribution, security, support, governance and conduct policies.
- Added architecture, command API, data lifecycle, privacy, threat-model, testing, rule-authoring, release, troubleshooting and licensing guidance.
- Added GitHub issue and pull-request templates.

## [1.2.1] - 2026-07-22

### Fixed

- Storage Assistant now uses the model's native chat template, disables unnecessary reasoning, and constrains generation with a JSON Schema so local summaries return valid structured output reliably.
- Structured assistant output is read from a dedicated file and parsed with balanced JSON handling instead of depending on unstructured console text.
- Assistant failures now provide a retry action and collapsible technical details without interrupting the deterministic scan report.

### Changed

- Redesigned the storage review screen with clearer drive metrics, readable storage categories, stronger cleanup hierarchy, and larger finding text.
- Placed recommended cleanup and optional local analysis in a focused action area instead of stacking oversized full-width panels.
- Replaced the content-obscuring sticky footer with a normal review action bar.
- Added explicit local-analysis loading, ready, success, and failure states.

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
