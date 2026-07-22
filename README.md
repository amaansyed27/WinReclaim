# WinReclaim

**Local-first Windows storage intelligence with deterministic cleanup, recovery context and signed updates.**

[![CI](https://github.com/amaansyed27/WinReclaim/actions/workflows/ci.yml/badge.svg)](https://github.com/amaansyed27/WinReclaim/actions/workflows/ci.yml)
[![Windows Release](https://github.com/amaansyed27/WinReclaim/actions/workflows/release-windows.yml/badge.svg)](https://github.com/amaansyed27/WinReclaim/actions/workflows/release-windows.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Platform: Windows 11 x64](https://img.shields.io/badge/Platform-Windows%2011%20x64-0078D4.svg)](docs/development.md)

> **Built for OpenAI Build Week — July Edition with GPT-5.6 Sol.** WinReclaim is an independent open-source project and is not endorsed by or affiliated with OpenAI.

[Download the latest Windows release](https://github.com/amaansyed27/WinReclaim/releases/latest) · [Read the documentation](docs/README.md) · [Report a bug](https://github.com/amaansyed27/WinReclaim/issues/new?template=bug_report.yml)

---

## What WinReclaim does

WinReclaim answers six practical questions:

1. What is taking my disk space?
2. What changed since my previous scan?
3. Which tool or workload most likely owns it?
4. What can be reclaimed under deterministic policy?
5. What will removal cost to restore, rebuild or redownload?
6. Can the cleanup be reversed?

It is not a registry cleaner, generic PC optimizer, debloater or AI-controlled deletion agent. Scanning and cleanup authority live in inspectable Rust code. Optional model output remains advisory.

## Why it exists

WinReclaim began with a real Windows storage investigation. A commonly suspected application used only a small fraction of the missing space. The larger contributors were local model stores, Android development data, package caches, container data and generated project output.

A targeted cleanup reclaimed substantial space without deleting projects or protected model stores. WinReclaim turns that investigation into a repeatable product with evidence, consequence labels, confirmation and receipts.

Read the full [Build Week origin](docs/build-week.md).

## Key capabilities

### Multi-drive storage scanning

Select one or more Windows drives and choose a scan profile:

- **Quick** — common temporary locations;
- **Balanced** — recommended known cleanup targets and Windows caches;
- **Deep** — project outputs, AppData and bounded unknown discovery;
- **Ultra** — widest supported coverage with lower size thresholds and larger result limits.

Fixed drives can expose reviewed cleanup actions. Removable and network drives remain inspection-only under the current policy.

### Storage Time Machine

Every completed scan creates a bounded local snapshot under:

```text
%LOCALAPPDATA%\WinReclaim\snapshots
```

Compatible later scans can show:

- storage growth or reduction;
- largest classified changes;
- likely owning tool or workload;
- recovery class and consequence;
- whether a change has a compiled cleanup adapter.

WinReclaim compares scans only when roots, profiles, enabled categories, thresholds, schema and rule-set versions are compatible. The first scan establishes a baseline.

### Reclaim Passports

Each finding can include a locally generated passport describing:

- likely owner;
- evidence used by the rule;
- last-change context;
- safety and recovery class;
- recovery method;
- consequence;
- evidence-based confidence.

Unknown dynamic discoveries remain inspection-only even when their names look cache-like.

### Reclaim Simulation

Before execution, WinReclaim creates an immutable hashed plan and shows:

- current and projected free space;
- estimated reclaim;
- reversible, redownloadable, rebuildable and irreversible portions;
- affected action count;
- protected items touched;
- consequences for each action.

Projected values are estimates. The final receipt reports measured free-space change and per-action results.

### Safe Undo Vault

Eligible user-temp and recognized crash-dump cleanup is moved into:

```text
%LOCALAPPDATA%\WinReclaim\vault
```

The vault provides:

- manifest-backed restore;
- original relative-path preservation;
- seven-day retention;
- native NTFS compression where available;
- refusal to overwrite existing destination files;
- explicit expiry and restore status.

Moving files on the same drive does not itself reclaim space, so receipts report measured net disk-space change rather than the original moved size.

### Signed updates

WinReclaim checks the official GitHub Release endpoint and verifies updater artifacts using Tauri's embedded public key. It never accepts an unsigned update.

Updater signing verifies release integrity. It is separate from Windows Authenticode publisher reputation, so an installer can still show an unknown-publisher warning when no Authenticode certificate is present.

## What WinReclaim detects

Current detection includes, when present and enabled by the selected profile:

- user temporary files;
- user-level crash dumps;
- Windows Temp;
- Windows Prefetch `.pf` files;
- Recycle Bin;
- Hugging Face cache;
- npm cache;
- Gradle caches and wrapper distributions;
- Cargo, pip, uv and Bun caches;
- Playwright browser binaries;
- Docker local data;
- Android SDK and virtual-device storage;
- Ollama models;
- Chrome, Edge and Cursor data;
- large `node_modules` directories;
- Rust `target` directories;
- Python virtual environments;
- verified build-output folders;
- bounded large-directory discovery;
- verified and dynamically discovered Windows-drive cache candidates.

Detection does not imply deletion authority. Project source, models, profiles, volumes and unknown folders remain protected or inspection-only unless a narrow compiled adapter exists.

## Executable cleanup adapters

Examples of current executable behaviour:

| Target | Method | Recovery class |
| --- | --- | --- |
| Eligible user temp | Compressed Undo Vault | Reversible during retention |
| Recognized user crash dumps | Compressed Undo Vault | Reversible during retention |
| Windows Temp entries | Exact-root entry cleanup | Irreversible; locked/protected entries skipped |
| Windows Prefetch `.pf` | Exact-root manual action | Rebuildable, review first |
| Recycle Bin | Native Windows Shell API | Irreversible |
| Hugging Face cache | Tool-native prune | Redownloadable |
| npm cache | Tool-native clean | Redownloadable |
| Conservative Docker data | Tool-native prune | Irreversible; volumes excluded |
| Verified caches/project outputs | Fingerprint-validated Rust adapter | Rebuildable/redownloadable |

Docker volumes are never included. Android virtual devices and SDK packages are not deleted through raw folder removal. Ollama models and browser profiles are protected.

## Safety model

WinReclaim uses four safety classes:

- **Safe now** — narrowly scoped disposable data;
- **Rebuild or redownload** — reproducible data with time/bandwidth cost;
- **Review first** — potentially disruptive environments or generated state;
- **Protected** — cannot enter an executable cleanup plan.

Non-negotiable controls:

- the frontend sends IDs, not arbitrary deletion paths;
- Rust resolves and validates all mutation targets;
- cleanup plans are immutable and hashed;
- paths and fingerprints are revalidated immediately before execution;
- reparse points, junctions and symbolic links are rejected;
- external tools receive fixed explicit argument arrays, not shell strings;
- protected classification overrides action availability;
- unknown discoveries remain inspection-only;
- restore never overwrites an existing destination;
- receipts distinguish estimates from measured results.

Read [Safety model](docs/safety.md) and [Threat model](docs/threat-model.md).

## Optional cloud intelligence

WinReclaim uses OpenRouter's `openrouter/free` router for two explicitly requested advisory features:

- **Storage Assistant** — summarizes aggregate storage totals and deterministic category/risk counts;
- **Reclaim by intent** — translates a natural-language preference into conservative constraints over existing executable candidates.

The desktop application does not contain or request an API key. It calls the WinReclaim server-side proxy at:

```text
https://winreclaim.vercel.app/api/assistant
```

The OpenRouter credential is stored only as a Vercel environment secret. Judges and normal users can test the feature without entering a key.

Storage summaries send aggregate totals and category/risk/action counts. Reclaim-by-intent sends the user's sentence plus opaque candidate IDs, category, size, risk class and recovery consequence. Neither request sends paths, drive labels, usernames, folder names, project names, directory trees or file contents.

The proxy fixes the model to `openrouter/free`, requires structured JSON output, validates request and response shapes and applies a demo rate limit. Rust independently validates returned IDs and safety classes. Remote output cannot create cleanup targets, change risk, create plans, run commands or execute deletion.

Free model availability can vary. The proxy retries incompatible output once, accepts bounded JSON-wrapped responses, and returns a conservative deterministic fallback when no compatible model is available. Scanning, review, planning, cleanup, history, receipts and restore remain local and unaffected.

Read [Storage Assistant](docs/storage-assistant.md) and [Privacy](docs/privacy.md).

## Privacy

WinReclaim includes no desktop telemetry.

Local operations:

- drive scanning and sizing;
- rule classification;
- snapshots and timeline;
- Reclaim Passports;
- planning and simulation;
- cleanup and receipts;
- Undo Vault and restore.

Intended network access is limited to:

- GitHub Releases for signed application updates;
- explicit advisory requests to the WinReclaim Vercel proxy and OpenRouter;
- public GitHub release metadata used by the landing page.

The previous local Qwen/`llama.cpp` assistant was removed. Version 1.2.1 automatically deletes its retired directory at startup.

Read [Privacy and network access](docs/privacy.md) and [Local data layout](docs/data-layout.md).

## Download and install

Open the [latest GitHub Release](https://github.com/amaansyed27/WinReclaim/releases/latest).

Recommended:

- `*-setup.exe` — NSIS installer for most users;
- `*.msi` — MSI package for managed deployment.

Official releases should also include `.sig` files and `latest.json` for the in-app updater.

Primary target: **Windows 11 x64**.

## Development

Requirements:

- Windows 11 x64;
- Node.js 22+;
- Rust 1.88+ with MSVC toolchain;
- Visual Studio Build Tools with Desktop development with C++;
- Windows SDK;
- WebView2 Runtime.

Start locally:

```powershell
git clone https://github.com/amaansyed27/WinReclaim.git
cd WinReclaim
npm ci
npm run tauri dev
```

Required validation:

```powershell
npm run check
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
cargo test --manifest-path src-tauri/Cargo.toml --all-targets
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

Local installer build:

```powershell
npm run tauri build
```

The desktop app targets the production proxy by default. To test a preview deployment:

```powershell
$env:WINRECLAIM_ASSISTANT_URL="https://your-preview-domain.vercel.app/api/assistant"
npm run tauri dev
```

Read the [Development guide](docs/development.md) and [Testing guide](docs/testing.md).

## Architecture

```text
React desktop UI
  └─ typed Tauri commands
      ├─ Windows drive/platform APIs
      ├─ bounded scanner
      ├─ deterministic rules and protected policy
      ├─ timeline and Reclaim Passports
      ├─ optional privacy-bounded OpenRouter constraints and summaries
      ├─ immutable planner and simulation
      ├─ compiled cleanup adapters
      ├─ compressed Undo Vault
      ├─ measured receipts
      └─ signed updater
```

Rules identify and explain storage. They cannot execute arbitrary commands. Cleanup behaviour lives in reviewed Rust adapters.

Read [Architecture](docs/architecture.md) and [Command API](docs/command-api.md).

## Documentation

- [Documentation index](docs/README.md)
- [Architecture](docs/architecture.md)
- [Safety model](docs/safety.md)
- [Threat model](docs/threat-model.md)
- [Rule system](docs/rules.md)
- [Rule authoring](docs/rule-authoring.md)
- [Development](docs/development.md)
- [Testing](docs/testing.md)
- [Release engineering](docs/releases.md)
- [Troubleshooting](docs/troubleshooting.md)
- [FAQ](docs/faq.md)
- [Privacy](docs/privacy.md)
- [Data layout](docs/data-layout.md)
- [Licensing](docs/licensing.md)
- [Build Week origin](docs/build-week.md)

## Contributing and support

- Read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request.
- Use [SUPPORT.md](SUPPORT.md) for bug-report requirements.
- Report vulnerabilities privately according to [SECURITY.md](SECURITY.md).
- Participation is governed by [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
- Project decisions and release authority are described in [GOVERNANCE.md](GOVERNANCE.md).
- Track user-facing changes in [CHANGELOG.md](CHANGELOG.md).

## Licence and notices

WinReclaim is released under the [MIT License](LICENSE).

Third-party dependencies and cloud providers retain their upstream licences and terms. See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) and [docs/licensing.md](docs/licensing.md).

OpenAI, GPT, OpenRouter, GitHub, Vercel, Windows and other product names are trademarks of their respective owners. Their mention does not imply endorsement.
