# Development Guide

WinReclaim is a Windows-first Tauri 2 desktop application with a React/TypeScript frontend and Rust backend.

## Supported development platform

The primary supported environment is Windows 11 x64 using PowerShell.

Required software:

- Git
- Node.js 22 or newer
- npm
- Rust 1.88 or newer
- Rust target `x86_64-pc-windows-msvc`
- Visual Studio Build Tools with **Desktop development with C++**
- Windows 10/11 SDK
- WebView2 Runtime

Recommended tools:

- Visual Studio Code
- rust-analyzer
- ESLint/TypeScript language support
- GitHub CLI for release diagnostics

## Clone and install

```powershell
git clone https://github.com/amaansyed27/WinReclaim.git
cd WinReclaim
npm ci
```

Confirm the toolchain:

```powershell
node --version
npm --version
rustc --version
cargo --version
rustup show active-toolchain
```

The Rust toolchain should resolve to MSVC on Windows. Install the expected target if necessary:

```powershell
rustup target add x86_64-pc-windows-msvc
rustup component add rustfmt clippy
```

## Run the application

```powershell
npm run tauri dev
```

This starts Vite on the configured local development URL and launches the Tauri application. The browser frontend alone cannot execute scans because the typed commands require the Rust process.

Frontend-only preview:

```powershell
npm run dev
```

Use frontend-only mode for layout work, but expect Tauri calls to fail unless they are guarded by development fixtures.

## Project layout

```text
WinReclaim/
├─ src/                         React and TypeScript frontend
│  ├─ components/              Shared application shell components
│  ├─ features/                Scan, findings, plan, receipt, vault, settings,
│  │                           timeline, updater and assistant features
│  ├─ lib/                     Typed Tauri client and presentation helpers
│  └─ types.ts                 Shared frontend domain types
├─ src-tauri/
│  ├─ src/
│  │  ├─ actions/              Compiled cleanup adapters
│  │  ├─ assistant/            Optional local assistant download and inference
│  │  ├─ commands/             Tauri command boundary
│  │  ├─ domain/               Rust domain models
│  │  ├─ insights/             Timeline and Reclaim Passport logic
│  │  ├─ intent/               Optional OpenAI intent interpretation
│  │  ├─ planner/              Immutable cleanup plan creation
│  │  ├─ platform/             Windows-specific APIs
│  │  ├─ policy.rs             Protected-path and safety policy
│  │  ├─ receipts/             Execution record persistence
│  │  ├─ rules/                Deterministic storage classification
│  │  ├─ scanner/              Bounded storage discovery and sizing
│  │  ├─ storage/              In-process application state
│  │  └─ vault/                Reversible cleanup and restoration
│  ├─ tauri.conf.json          Desktop, bundle and updater configuration
│  └─ Cargo.toml               Rust dependencies and release profile
├─ scripts/                    Repository integrity and version scripts
├─ docs/                       Product and developer documentation
├─ landing-page/               Static Vercel-ready product site
└─ .github/workflows/          CI and Windows release automation
```

## Frontend workflow

Type checking and integrity checks:

```powershell
npm run check
```

Production build:

```powershell
npm run build
```

The `check:integrity` script enforces product-specific constraints. Do not bypass it to make CI green; update implementation and tests together when a legitimate contract changes.

### Tauri calls

Keep normal application commands centralized in `src/lib/tauri.ts`. Feature-specific APIs may live beside a feature when they have their own domain types, as with the Storage Assistant.

Frontend code should:

- submit stable IDs and typed request objects;
- avoid constructing filesystem paths for execution;
- preserve backend error messages in a user-safe form;
- unsubscribe from Tauri events during cleanup;
- distinguish estimated and measured values;
- show consequences before destructive actions.

See [command-api.md](command-api.md).

## Rust workflow

Run commands from the repository root:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml --all
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
cargo test --manifest-path src-tauri/Cargo.toml --all-targets
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

Useful focused tests:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml scanner
cargo test --manifest-path src-tauri/Cargo.toml planner
cargo test --manifest-path src-tauri/Cargo.toml vault
cargo test --manifest-path src-tauri/Cargo.toml assistant
```

### Error handling

- Tauri command handlers should return structured failures rather than panic.
- Filesystem errors should include enough context to diagnose the target category without leaking unnecessary user data to remote services.
- Locked or inaccessible files are normally skipped and reflected in results rather than forcefully removed.
- Integer storage calculations should use checked or saturating arithmetic.

### Filesystem mutation

Before mutating a path:

1. resolve the action from backend-owned state;
2. verify the plan ID and hash when applicable;
3. confirm the current path still matches the rule/action contract;
4. reject reparse points and symbolic links;
5. canonicalize and validate the allowed root;
6. avoid crossing filesystem or trust boundaries unexpectedly;
7. measure the result for the receipt.

## Optional OpenAI intent feature

Set the key only in the process environment:

```powershell
$env:OPENAI_API_KEY="your-key"
$env:OPENAI_MODEL="gpt-5.6" # optional override
npm run tauri dev
```

The key must not be stored in frontend state, committed files or screenshots. The feature remains optional; all deterministic scan and cleanup functionality works without it.

## Optional local Storage Assistant

The local assistant is installed from the application's Settings page. Development builds download a pinned Qwen GGUF model and a pinned `llama.cpp` Windows CPU sidecar. Both artifacts are verified before use.

Developers should not replace pinned URLs or hashes without updating:

- `src-tauri/src/assistant/mod.rs`;
- download verification tests;
- [model-sources.md](model-sources.md);
- [storage-assistant.md](storage-assistant.md);
- [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md).

## Local application data

WinReclaim writes owned state below:

```text
%LOCALAPPDATA%\WinReclaim
```

Use the Settings reset controls where possible. For isolated development testing, back up any needed vault data before removing the directory manually. See [data-layout.md](data-layout.md).

## Build installers locally

```powershell
npm run tauri build
```

Expected output is below `src-tauri\target\release\bundle` for the default target, or the target-specific directory when `--target` is supplied.

Local builds may not have updater signatures unless `TAURI_SIGNING_PRIVATE_KEY` is set. Never use the production private signing key on an untrusted machine.

## Version synchronization

The application version is stored in multiple manifests. Use:

```powershell
npm run version:set -- 1.2.3
```

This updates the package and Tauri/Cargo manifests managed by the script. Official releases perform version synchronization through GitHub Actions; do not manually create the release tag first.

## Working with the landing page

```powershell
cd landing-page
python -m http.server 4173
```

Open `http://localhost:4173`. The landing page is dependency-free and retrieves the latest public GitHub Release metadata in the browser.

## Debugging tips

Enable Rust backtraces for a development session:

```powershell
$env:RUST_BACKTRACE="1"
npm run tauri dev
```

Run the frontend with full TypeScript diagnostics:

```powershell
npx tsc --noEmit --pretty false
```

Inspect the Windows release bundle tree:

```powershell
Get-ChildItem .\src-tauri\target -Recurse -File |
  Where-Object { $_.Extension -in '.exe', '.msi', '.sig' }
```

See [troubleshooting.md](troubleshooting.md) for common toolchain and runtime failures.
