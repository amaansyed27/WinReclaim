# Development Guide

WinReclaim is a Windows-first Tauri 2 desktop application with a React/TypeScript frontend, Rust backend and an optional Vercel/OpenRouter explanation layer.

## Supported development platform

Primary environment: Windows 11 x64 with PowerShell.

Required software:

- Git;
- Node.js 22 or newer and npm;
- Rust 1.88 or newer with `x86_64-pc-windows-msvc`;
- Visual Studio Build Tools with **Desktop development with C++**;
- Windows 10/11 SDK;
- WebView2 Runtime.

Recommended: Visual Studio Code, rust-analyzer and GitHub CLI.

## Clone and run

```powershell
git clone https://github.com/amaansyed27/WinReclaim.git
cd WinReclaim
npm ci
rustup target add x86_64-pc-windows-msvc
rustup component add rustfmt clippy
npm run tauri dev
```

Frontend-only layout preview:

```powershell
npm run dev
```

The browser preview cannot execute scans because Tauri commands require the Rust process.

## Project layout

```text
WinReclaim/
├─ src/                         React and TypeScript frontend
│  ├─ components/              Shared application shell
│  ├─ features/                Scan, findings, assistant, plan, receipt,
│  │                           vault, settings, timeline and updater
│  ├─ lib/                     Typed Tauri clients and helpers
│  └─ types.ts                 Frontend domain types
├─ src-tauri/
│  ├─ src/
│  │  ├─ actions/              Compiled cleanup adapters
│  │  ├─ assistant/            Aggregate cloud-summary boundary
│  │  ├─ cloud.rs              Fixed HTTPS proxy transport
│  │  ├─ commands/             Tauri command boundary
│  │  ├─ domain/               Rust domain models
│  │  ├─ insights/             Timeline and Reclaim Passports
│  │  ├─ intent/               OpenRouter-backed intent constraints
│  │  ├─ planner/              Immutable hashed cleanup plans
│  │  ├─ platform/             Windows-specific APIs
│  │  ├─ policy.rs             Protected-path policy
│  │  ├─ receipts/             Measured execution records
│  │  ├─ rules/                Deterministic classification
│  │  ├─ scanner/              Bounded discovery and sizing
│  │  ├─ storage/              In-process application state
│  │  └─ vault/                Reversible cleanup and restore
│  ├─ tauri.conf.json
│  └─ Cargo.toml
├─ landing-page/
│  ├─ api/assistant.js         Vercel proxy for OpenRouter
│  └─ ...                      Static product site
├─ scripts/                    Integrity and version scripts
├─ docs/                       Product/developer documentation
└─ .github/workflows/          CI and Windows release automation
```

## Frontend checks

```powershell
npm run check
npm run build
```

`check:integrity` enforces product safety and architecture contracts. Do not bypass it to make CI pass.

Frontend rules:

- submit typed IDs and request objects, not deletion paths;
- keep credentials out of React state and built JavaScript;
- distinguish estimated and measured values;
- preserve explicit loading/failure states for optional cloud features;
- show cleanup consequences before confirmation.

See [command-api.md](command-api.md).

## Rust checks

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml --all
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
cargo test --manifest-path src-tauri/Cargo.toml --all-targets
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

Focused tests:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml scanner
cargo test --manifest-path src-tauri/Cargo.toml planner
cargo test --manifest-path src-tauri/Cargo.toml vault
cargo test --manifest-path src-tauri/Cargo.toml assistant
cargo test --manifest-path src-tauri/Cargo.toml intent
```

Before filesystem mutation, Rust must resolve backend-owned state, verify the plan hash, revalidate current evidence, reject links/reparse points and protected overlaps, and measure results for the receipt.

## Optional cloud assistant

The desktop application does not read `OPENROUTER_API_KEY`. It calls the public WinReclaim proxy:

```text
https://winreclaim.vercel.app/api/assistant
```

For a Vercel preview deployment:

```powershell
$env:WINRECLAIM_ASSISTANT_URL="https://your-preview-domain.vercel.app/api/assistant"
npm run tauri dev
```

The override must use HTTPS.

### Run the proxy

From `landing-page`:

```powershell
vercel link --project winreclaim
vercel env add OPENROUTER_API_KEY preview
vercel env add OPENROUTER_API_KEY production
vercel dev
```

Enter the key only in Vercel's interactive secret prompt. Never put it in source, a committed `.env`, frontend code, Rust constants, screenshots or logs.

The proxy supports only fixed `storage_summary` and `intent_constraints` tasks, fixes the model to `openrouter/free`, requires structured JSON output and validates requests/responses.

### Privacy contract

Storage summaries may send aggregate drive totals and category/risk/action counts. Intent requests may send the user's sentence plus opaque candidate IDs, category, size, deterministic risk and generic consequence.

Do not add paths, drive labels, usernames, folder names, project names, directory trees or file contents to a cloud payload.

See [privacy.md](privacy.md), [storage-assistant.md](storage-assistant.md) and [threat-model.md](threat-model.md).

## Local application data

```text
%LOCALAPPDATA%\WinReclaim
```

Use Settings reset controls where possible. Version 1.2.1 removes the retired local-assistant model directory during startup. See [data-layout.md](data-layout.md).

## Build installers locally

```powershell
npm run tauri build
```

Bundles appear under `src-tauri\target\release\bundle` or the target-specific directory. Local builds may not contain updater signatures unless `TAURI_SIGNING_PRIVATE_KEY` is set. Never use the production signing key on an untrusted machine.

## Version synchronization

```powershell
npm run version:set -- 1.2.3
```

Official releases perform synchronization and tagging through GitHub Actions. Do not create the release tag manually first.

## Landing page

Static preview:

```powershell
cd landing-page
python -m http.server 4173
```

Vercel preview including the serverless route:

```powershell
cd landing-page
vercel dev
```

## Debugging

```powershell
$env:RUST_BACKTRACE="1"
npm run tauri dev
```

```powershell
npx tsc --noEmit --pretty false
```

```powershell
Get-ChildItem .\src-tauri\target -Recurse -File |
  Where-Object { $_.Extension -in '.exe', '.msi', '.sig' }
```

See [testing.md](testing.md) and [troubleshooting.md](troubleshooting.md).