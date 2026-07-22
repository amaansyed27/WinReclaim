# WinReclaim

**Local-first Windows storage intelligence with deterministic cleanup, recovery context, and signed updates.**

[![CI](https://github.com/amaansyed27/WinReclaim/actions/workflows/ci.yml/badge.svg)](https://github.com/amaansyed27/WinReclaim/actions/workflows/ci.yml)
[![Windows Release](https://github.com/amaansyed27/WinReclaim/actions/workflows/release-windows.yml/badge.svg)](https://github.com/amaansyed27/WinReclaim/actions/workflows/release-windows.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Platform: Windows 11 x64](https://img.shields.io/badge/Platform-Windows%2011%20x64-0078D4.svg)](docs/development.md)

> **Built for OpenAI Build Week — July Edition with GPT-5.6 Sol.** WinReclaim is an independent open-source project and is not endorsed by or affiliated with OpenAI.

[Download the latest Windows release](https://github.com/amaansyed27/WinReclaim/releases/latest) · [Read the documentation](docs/README.md) · [Report a bug](https://github.com/amaansyed27/WinReclaim/issues/new?template=bug_report.yml)

---

## What WinReclaim does

WinReclaim explains where Windows storage is going, separates reclaimable data from protected data, previews consequences, and executes only reviewed cleanup adapters.

It answers six practical questions:

1. What is taking my disk space?
2. What changed since the previous compatible scan?
3. Which tool or workload most likely owns it?
4. What can be reclaimed under deterministic policy?
5. What will removal cost to restore, rebuild, or redownload?
6. Can the cleanup be reversed?

It is not a registry cleaner, debloater, generic PC optimizer, or AI-controlled deletion agent. Scanning, classification, planning, and cleanup authority live in inspectable Rust code.

## Production guarantees

The installed desktop application requires:

- no account;
- no API key;
- no model download;
- no hosted assistant service;
- no telemetry connection;
- no background subscription.

Core scanning, findings, local summaries, intent suggestions, planning, cleanup, receipts, history, and Undo Vault operation remain available offline.

## Key capabilities

### Multi-drive scanning

Select one or more Windows drives and choose a scan profile:

- **Quick** — common temporary locations;
- **Balanced** — recommended known cleanup targets and Windows caches;
- **Deep** — project outputs, AppData, and bounded unknown discovery;
- **Ultra** — widest supported coverage with lower size thresholds and larger result limits.

Fixed drives can expose reviewed cleanup actions. Removable and network drives remain inspection-only under the current policy.

### Storage Time Machine

Every completed scan creates a bounded local snapshot under:

```text
%LOCALAPPDATA%\WinReclaim\snapshots
```

Compatible later scans can show storage growth or reduction, ownership context, recovery class, and whether a compiled cleanup adapter exists. The first scan establishes a baseline.

### Reclaim Passports

Findings can include locally generated context describing:

- likely owner;
- evidence used by the rule;
- last-change context;
- safety and recovery class;
- recovery method;
- consequence;
- evidence-based confidence.

Unknown dynamic discoveries remain inspection-only even when their names appear cache-like.

### Storage Brief

The Storage Brief is generated locally from the completed scan using deterministic aggregation. It summarizes:

- measured used and free space;
- largest reported categories;
- verified action counts;
- safety-class distribution;
- skipped entries and scan warnings.

It does not use a model, network request, API key, file contents, or remote service. Category rows can overlap, so drive totals remain authoritative.

### Local reclaim-by-intent

The optional intent field uses conservative local rules to interpret requests such as:

```text
Free 10 GB from safe items and rebuildable caches.
```

It supports:

- size targets expressed in KB, MB, GB, or TB;
- low-impact actions by default;
- rebuildable/redownloadable actions only when the request accepts them;
- review-first actions only through explicit opt-in wording;
- category exclusions such as “do not touch browser caches.”

The result is an editable suggestion over existing verified candidates. It cannot create a new cleanup target, change risk, create a plan, run a command, or execute deletion.

### Reclaim Simulation

Before execution, WinReclaim creates an immutable hashed plan showing:

- current and projected free space;
- estimated reclaim;
- reversible, redownloadable, rebuildable, and irreversible portions;
- affected action count;
- consequences for each action.

Projected values are estimates. The final receipt reports measured free-space change and per-action results.

### Safe Undo Vault

Eligible user-temp and recognized crash-dump cleanup can move data into:

```text
%LOCALAPPDATA%\WinReclaim\vault
```

The vault provides manifest-backed restore, original relative-path preservation, bounded retention, NTFS compression where available, overwrite refusal, and explicit restore status.

Moving files on the same drive does not itself reclaim space. Receipts report measured net disk-space change.

### Signed updates

WinReclaim checks the official GitHub Release endpoint and verifies updater artifacts using Tauri's embedded public key. Unsigned update artifacts are rejected.

Updater signing is separate from Windows Authenticode publisher reputation, so an installer can still display an unknown-publisher warning when no commercial Authenticode certificate is present.

## What WinReclaim detects

Current detection includes, when present and enabled by the selected profile:

- user temporary files and crash dumps;
- Windows Temp and Prefetch;
- Recycle Bin;
- Hugging Face, npm, Gradle, Cargo, pip, uv, and Bun caches;
- Playwright browser binaries;
- Docker local data;
- Android SDK and virtual-device storage;
- Ollama models;
- Chrome, Edge, and Cursor data;
- large `node_modules`, Rust `target`, and Python environment directories;
- verified build-output folders;
- bounded large-directory discovery;
- verified and dynamically discovered Windows-drive cache candidates.

Detection does not imply deletion authority. Project source, model stores, profiles, volumes, and unknown folders remain protected or inspection-only unless a narrow compiled adapter exists.

## Executable cleanup adapters

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

Docker volumes are excluded. Android virtual devices and SDK packages are not deleted through raw folder removal. Ollama models and browser profiles are protected.

## Safety model

WinReclaim uses four safety classes:

- **Safe now** — narrowly scoped disposable data;
- **Rebuild or redownload** — reproducible data with time or bandwidth cost;
- **Review first** — potentially disruptive environments or generated state;
- **Protected** — cannot enter an executable cleanup plan.

Non-negotiable controls:

- the frontend sends IDs, not arbitrary deletion paths;
- Rust resolves and validates mutation targets;
- cleanup plans are immutable and hashed;
- paths and fingerprints are revalidated immediately before execution;
- reparse points, junctions, and symbolic links are rejected;
- external tools receive fixed explicit argument arrays, not shell strings;
- protected classification overrides action availability;
- unknown discoveries remain inspection-only;
- restore never overwrites an existing destination;
- receipts distinguish estimates from measured results;
- local summaries and intent rules have no execution authority.

Read [Safety model](docs/safety.md) and [Threat model](docs/threat-model.md).

## Privacy and network access

WinReclaim includes no desktop telemetry. Scans, paths, snapshots, plans, receipts, and vault payloads remain on the device.

The desktop application's intended network access is limited to signed GitHub Release update checks and downloads. The static landing page may request public GitHub release metadata to resolve download links.

The retired Qwen/`llama.cpp` assistant and OpenRouter proxy are not part of the production runtime. Version 1.2.1 removes the old local assistant directory during startup migration.

Read [Privacy and network access](docs/privacy.md) and [Local data layout](docs/data-layout.md).

## Download and install

Open the [latest GitHub Release](https://github.com/amaansyed27/WinReclaim/releases/latest).

Recommended artifacts:

- `*-setup.exe` — NSIS installer for most Windows users;
- `*.msi` — MSI package for managed deployment.

Official releases should also include `.sig` files and `latest.json` for the signed in-app updater.

Primary target: **Windows 11 x64**.

## Development

Requirements:

- Windows 11 x64;
- Node.js 22+;
- Rust 1.88+ with the MSVC toolchain;
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

Build installers locally:

```powershell
npm run tauri build
```

Read the [Development guide](docs/development.md) and [Testing guide](docs/testing.md).

## Architecture

```text
React desktop UI
  └─ typed Tauri commands
      ├─ Windows drive/platform APIs
      ├─ bounded scanner
      ├─ deterministic rules and protected policy
      ├─ local Storage Brief and intent rules
      ├─ timeline and Reclaim Passports
      ├─ immutable planner and simulation
      ├─ compiled cleanup adapters
      ├─ compressed Undo Vault
      ├─ measured receipts
      └─ signed updater
```

Rules identify and explain storage. They cannot execute arbitrary commands. Cleanup behaviour lives in reviewed Rust adapters.

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

WinReclaim is released under the [MIT License](LICENSE). Third-party dependencies retain their upstream licences. See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) and [docs/licensing.md](docs/licensing.md).

OpenAI, GPT, GitHub, Windows, and other product names are trademarks of their respective owners. Their mention does not imply endorsement.
