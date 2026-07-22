# Contributing to WinReclaim

WinReclaim accepts bug fixes, documentation, tests, detection rules and carefully reviewed product changes. Because it can remove files and optionally sends bounded metadata to a cloud proxy, safety and privacy requirements are stricter than in a typical desktop application.

## Read first

- [Developer documentation](docs/README.md)
- [Architecture](docs/architecture.md)
- [Safety model](docs/safety.md)
- [Privacy](docs/privacy.md)
- [Threat model](docs/threat-model.md)
- [Rule authoring](docs/rule-authoring.md)
- [Testing](docs/testing.md)

Use GitHub Issues for bugs/features. Report vulnerabilities privately through [SECURITY.md](SECURITY.md).

## Development setup

WinReclaim is developed primarily on Windows 11 x64 with Node.js 22+, npm, Rust/MSVC, Visual Studio C++ Build Tools, WebView2 and Git.

```powershell
git clone https://github.com/amaansyed27/WinReclaim.git
cd WinReclaim
npm ci
npm run tauri dev
```

See [docs/development.md](docs/development.md).

## Pull-request workflow

1. Branch from current `main`.
2. Keep the change focused.
3. Add success and refusal-path tests.
4. Update user/developer docs for contract changes.
5. Run all required checks.
6. Open a PR with the template.

## Required checks

```powershell
npm ci
npm run check
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
cargo test --manifest-path src-tauri/Cargo.toml --all-targets
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

Installer changes also require `npm run tauri build`.

## Non-negotiable boundaries

- Frontend and remote models submit typed IDs/data, never arbitrary cleanup paths.
- Cleanup actions are compiled Rust adapters with explicit validation.
- User/filesystem/model input is never interpolated into shell strings.
- Reparse points and links are refused during mutation.
- Protected precedence cannot be weakened.
- Unknown discovery remains inspection-only without a reviewed adapter.
- Estimates are not presented as measured results.
- Cloud assistance remains optional and advisory.
- Provider credentials never enter desktop source, frontend code, binaries or logs.
- Cloud payloads exclude paths, drive labels, usernames, folder/file names, project names, directory trees and contents.
- Remote output cannot change risk/action fields, create plans or execute cleanup.
- Vault restore never overwrites existing destinations.

Changes to deletion, path policy, proxy payloads/schemas, credential handling, updater signing, persistence or plan hashing require explicit security review and refusal tests.

## Rules and adapters

Rule changes need a stable ID, deterministic evidence, owner/category, consequence, safety class, protected exclusions and tests. Detection alone does not justify deletion. New executable behaviour requires a compiled adapter and separate review. See [docs/rule-authoring.md](docs/rule-authoring.md).

## Cloud changes

Any change to `landing-page/api/assistant.js`, `src-tauri/src/cloud.rs`, assistant/intent payloads or provider routing must:

- document exact transmitted fields;
- add negative privacy assertions;
- keep `OPENROUTER_API_KEY` server-side only;
- reject client-selected models, prompts, tools and provider options;
- validate request and response shapes;
- preserve offline deterministic functionality;
- update privacy, threat-model, testing and changelog documentation.

Do not use a personal broad-spend key for tests. Use a dedicated limited Vercel secret.

## Frontend conventions

Keep Tauri calls in typed API modules, use shared domain types, preserve visible consequences, avoid fabricated metrics, maintain keyboard/focus support and show explicit cloud loading/failure states.

## Rust conventions

Prefer small modules, structured errors, fixed `std::process::Command` arguments, execution-time path validation, saturating storage arithmetic and tests near the responsible module.

## Documentation

Use relative links and PowerShell examples. Distinguish shipped behaviour from plans. Never describe the retired Qwen/`llama.cpp` local model or direct OpenAI API integration as current behaviour. Update `CHANGELOG.md` for meaningful changes.

## Licensing

Contributions are accepted under the repository MIT License unless agreed otherwise before acceptance. Contributors must have the right to submit their material.

Safety, privacy and truthful recovery consequences take precedence over convenience and release speed.