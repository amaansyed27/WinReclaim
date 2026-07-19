# WinReclaim

**Local-first Windows storage intelligence for developers and AI power users.**

WinReclaim answers six questions:

1. What is taking my disk space?
2. What changed since my previous scan?
3. Which tool or workload most likely owns it?
4. What can I safely reclaim?
5. What will removing it cost me later?
6. Can I reverse the cleanup?

It is not a registry cleaner, generic PC optimiser, debloater or AI-controlled deletion agent. The scanner and cleanup decisions are deterministic, inspectable and local.

## Build Week origin

WinReclaim started from a real Windows storage investigation. WhatsApp was suspected, but it used only 766 MB. The actual storage pressure came from local AI models, Android emulators, Hugging Face cache, Docker data, Gradle, npm and project build output. A targeted cleanup reclaimed 27.55 GB without touching projects or Ollama models.

## Storage intelligence

### Storage Time Machine

Every completed scan creates a local snapshot under `%LOCALAPPDATA%\WinReclaim\snapshots`. WinReclaim retains the latest 40 snapshots and compares the newest two to show:

- profile storage growth or reduction
- the largest classified directory changes
- likely owning tool or workload
- attribution confidence
- recovery class
- growth already backed by an executable cleanup adapter

The Time Machine page includes a storage-growth graph and a ranked change timeline. The first scan establishes a baseline; the second and later scans produce deltas.

Current attribution is deterministic and evidence-based. It combines WinReclaim rule identity, directory location, category and filesystem modification time. Kernel-level process attribution through ETW and USN Journal monitoring is not part of the current implementation.

### Reclaim Passports

Every finding receives a locally generated passport containing:

- likely owner
- directory-level last-change time
- recovery class
- recovery method
- estimated recovery time
- confidence score
- evidence used by the classifier

Unknown dynamic discoveries remain inspection-only even when their names look cache-like.

### Reclaim Simulation

Before execution, the immutable cleanup plan includes a simulation showing:

- current and projected free space
- estimated reclaim
- reversible, redownloadable, rebuildable and irreversible portions
- affected action count
- estimated recovery time
- protected items touched

Projected values remain estimates. The receipt reports the actual free-space change measured by Windows after execution.

### Safe Undo Vault

Eligible user-temp and crash-dump cleanup is moved into `%LOCALAPPDATA%\WinReclaim\vault` instead of being immediately destroyed.

- seven-day local retention
- native NTFS compression through `compact.exe`
- manifest-backed restore
- original relative paths preserved
- existing destination files are never overwritten
- expired payloads are removed automatically

Moving data on the same drive does not itself reclaim space, so WinReclaim compresses the vault payload and reports only the measured net disk-space change. npm, Hugging Face and Docker command-based cleanup cannot be fully reversed and are labelled accordingly.

## Scanning

### Detection

- User temporary files eligible by age
- User-level crash dumps
- Hugging Face cache
- npm cache
- Gradle cache and wrapper distributions
- Cargo, pip, uv and Bun caches
- Playwright browser binaries
- Docker local data
- Android SDK and AVDs
- Ollama models
- Chrome, Edge and Cursor data
- Large `node_modules`, Rust `target`, Python virtual environments and build folders in common project locations
- Dynamically discovered large directories that do not match the built-in catalogue

Scan profiles include Quick, Balanced, Deep and Ultra. Ultra enables every source, AppData discovery, the 64 MB minimum threshold and up to 100 dynamic findings while retaining protected-root and reparse-point exclusions.

WinReclaim excludes its own snapshot, receipt and vault data from dynamic findings.

## Reclaim by intent

The optional GPT-5.6 feature translates a request such as:

> Free around 20 GB, but do not touch Ollama, browser profiles or Android emulators.

into conservative constraints:

- target reclaim size
- allowed safety classes
- explicit exclusions
- a short explanation

GPT never receives filesystem paths, usernames, project names or directory trees. It cannot return commands or deletion targets. Rust applies those constraints only to the allowlisted findings from the current scan, and the user must still review the selection before WinReclaim creates a hashed cleanup plan.

Set the API key in the process environment before starting the app:

```powershell
$env:OPENAI_API_KEY="your-key"
$env:OPENAI_MODEL="gpt-5.6" # optional override
npm run tauri dev
```

Without an API key, scanning, timeline history, passports, simulation, manual planning, cleanup, undo and receipts remain available offline.

## Executable adapters

- User temp files older than seven days → compressed Undo Vault
- Recognised user-level crash dumps → compressed Undo Vault
- `hf cache prune --yes` → redownloadable
- `npm cache clean --force` → redownloadable
- Conservative `docker system prune --force --filter until=168h` → irreversible

Docker volumes are never included.

## Signed updates

WinReclaim checks for updates shortly after startup and also provides a manual version button in the top bar. Available releases are downloaded and verified using Tauri's signed updater before installation.

The updater reads the `latest.json` generated by the GitHub release workflow. It never accepts unsigned packages.

## Safety classes

- **Safe now** — narrowly scoped disposable data
- **Rebuild or redownload** — caches that a tool can restore later
- **Review first** — environments, containers and generated project data
- **Protected** — data WinReclaim refuses to add to an automatic cleanup plan

## Explicit exclusions

WinReclaim does not:

- clean Prefetch
- modify the registry
- promise fake speed improvements
- delete browser profiles
- remove Ollama models automatically
- prune Docker volumes
- delete Android emulators or SDK packages by raw folder removal
- follow filesystem reparse points during scanning or cleanup
- accept arbitrary deletion paths from the frontend
- let GPT execute or broaden a cleanup plan
- overwrite files during vault restore
- install an unsigned update

## Architecture

```text
React desktop UI
  └─ typed Tauri commands
      ├─ bounded + dynamic scanner
      ├─ deterministic rules
      ├─ snapshot timeline and attribution
      ├─ Reclaim Passport generator
      ├─ optional GPT intent constraints
      ├─ immutable cleanup planner + simulation
      ├─ allowlisted cleanup adapters
      ├─ compressed Undo Vault + restore engine
      ├─ measured receipt persistence
      └─ signed updater
```

Rules identify and explain storage. They cannot execute arbitrary commands. Cleanup behaviour lives in compiled Rust adapters.

See:

- [`docs/architecture.md`](docs/architecture.md)
- [`docs/rules.md`](docs/rules.md)
- [`docs/safety.md`](docs/safety.md)
- [`docs/releases.md`](docs/releases.md)

## Development

Requirements:

- Windows 11
- Node.js 22+
- Rust stable with the MSVC toolchain
- Visual Studio Build Tools with Desktop development with C++
- WebView2 Runtime

```powershell
npm install
npm run tauri dev
```

Frontend checks:

```powershell
npm run check
npm run build
```

Rust checks:

```powershell
cd src-tauri
cargo fmt
cargo test
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
```

Local installer build:

```powershell
npm run tauri build
```

Official releases are created using **Actions → Release Windows → Run workflow**. The workflow synchronizes versions, validates the project, creates the tag, publishes NSIS and MSI installers, and generates updater signatures plus `latest.json`.

## Privacy

The scan, dynamic discovery, snapshots, passports, timeline, plans, cleanup, vault and receipts run locally. WinReclaim does not upload file paths, project names, directory trees or receipts. No telemetry is included.

When reclaim-by-intent is enabled, only labels, categories, sizes, safety classes and consequences of currently executable findings are sent to the OpenAI Responses API with response storage disabled. The API key remains in the Rust process environment and is never sent to the webview.

Update checks contact only the configured GitHub Releases endpoint and do not upload scan information.

## Validation

The Storage Time Machine implementation passed:

- TypeScript type checking
- production Vite build
- 11 Rust tests
- strict Clippy with warnings denied

## Status

Storage Time Machine, Reclaim Passports, Reclaim Simulation and the compressed Safe Undo Vault are implemented on `main`. They are not included in an older installed release until a new Windows release is published.

## License

MIT
