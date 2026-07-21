# Contributing to WinReclaim

WinReclaim accepts bug fixes, documentation improvements, tests, detection rules and carefully reviewed product changes. Because the application can remove files, safety requirements are stricter than in a typical desktop application.

## Before contributing

Read these documents first:

- [Developer documentation](docs/README.md)
- [Architecture](docs/architecture.md)
- [Safety model](docs/safety.md)
- [Threat model](docs/threat-model.md)
- [Rule authoring](docs/rule-authoring.md)
- [Testing](docs/testing.md)

Use GitHub Issues for bugs and feature proposals. Do not publish vulnerability details in a public issue; follow [SECURITY.md](SECURITY.md).

## Development environment

WinReclaim is developed and tested primarily on Windows 11 x64.

Required software:

- Node.js 22 or newer
- npm
- Rust 1.88 or newer with the MSVC toolchain
- Visual Studio Build Tools with **Desktop development with C++**
- WebView2 Runtime
- Git

Clone and start the desktop application:

```powershell
git clone https://github.com/amaansyed27/WinReclaim.git
cd WinReclaim
npm ci
npm run tauri dev
```

See [docs/development.md](docs/development.md) for full setup and troubleshooting guidance.

## Branch and pull-request workflow

1. Create a focused branch from the latest `main`.
2. Keep unrelated refactors out of the change.
3. Add or update tests for behaviour changes.
4. Update documentation when a user-facing contract changes.
5. Run all required checks locally.
6. Open a pull request using the repository template.

Recommended branch names:

```text
fix/locked-file-receipt
feature/new-cache-rule
docs/release-guide
refactor/scanner-boundary
```

## Required checks

Run from the repository root:

```powershell
npm ci
npm run check
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
cargo test --manifest-path src-tauri/Cargo.toml --all-targets
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

For installer-affecting changes, also run:

```powershell
npm run tauri build
```

## Safety rules for code changes

A contribution must not weaken these boundaries:

- The frontend submits stable IDs and typed requests, not arbitrary cleanup paths.
- Cleanup actions are compiled Rust adapters with explicit validation.
- User-controlled input must not be interpolated into shell command strings.
- Reparse points and symbolic links must not be followed during cleanup.
- Protected findings must never become actionable through rule precedence mistakes.
- Unknown discoveries remain inspection-only unless a reviewed deterministic adapter exists.
- Estimates must not be presented as measured values.
- The optional GPT feature and local Storage Assistant remain advisory only.
- Existing destination files must never be overwritten during vault restoration.

Any change to deletion logic, path validation, updater signing, model verification or plan hashing requires tests covering both success and refusal paths.

## Adding or changing a detection rule

Rule changes should include:

- a stable rule ID;
- the recognised path or fingerprint;
- the owner/product label;
- an accurate consequence description;
- a safety class;
- confidence grounded in deterministic evidence;
- tests for expected matches and protected exclusions;
- documentation when the rule introduces a new category.

Detection alone does not justify deletion. New executable cleanup behaviour requires a compiled adapter and separate safety review. See [docs/rule-authoring.md](docs/rule-authoring.md).

## Frontend conventions

- Keep Tauri calls inside `src/lib/tauri.ts` or a feature-specific typed API module.
- Use shared domain types rather than duplicating response shapes.
- Preserve plain-language explanations for destructive or rebuildable actions.
- Avoid displaying fabricated percentages, recovery times or ownership certainty.
- Keep keyboard navigation and visible focus states intact.
- Do not hide safety consequences behind hover-only UI.

## Rust conventions

- Prefer small modules with explicit ownership boundaries.
- Return structured errors rather than panicking in command handlers.
- Use `std::process::Command` with fixed executables and explicit argument arrays.
- Canonicalize and revalidate paths immediately before filesystem mutation.
- Treat filesystem metadata as stale between scan and execution.
- Use saturating arithmetic for storage totals.
- Add unit tests next to modules and integration tests where cross-module behaviour matters.

## Documentation conventions

- Use relative repository links.
- Write commands for PowerShell unless the instruction is platform-independent.
- Distinguish current behaviour from planned work.
- Do not describe an unreleased feature as available in the latest installer.
- Update `CHANGELOG.md` under **Unreleased** for meaningful changes.

## Commit messages

Use concise imperative messages. Conventional prefixes are encouraged:

```text
fix: reject stale generic cache fingerprints
feat: add verified cache rule for a new tool
docs: explain updater signing recovery
refactor: isolate receipt persistence
ci: tighten Windows release validation
```

## Licensing of contributions

WinReclaim is distributed under the MIT License. By submitting a contribution, you confirm that you have the right to provide it and agree that it may be distributed under the repository license. No contributor licence agreement is currently required.

## Review expectations

Maintainers may request changes when a patch:

- expands cleanup scope without adequate evidence;
- relies on folder names alone for destructive behaviour;
- changes persisted formats without migration or compatibility handling;
- introduces silent network activity or telemetry;
- weakens local-first defaults;
- lacks refusal-path tests;
- mixes unrelated product and formatting changes.

Safety takes precedence over convenience and release speed.
