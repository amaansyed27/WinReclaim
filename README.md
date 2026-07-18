# WinReclaim

**Local-first Windows storage intelligence for developers and AI power users.**

WinReclaim answers four questions:

1. What is taking my disk space?
2. What can I safely reclaim?
3. What will removing it cost me later?
4. What actually changed after cleanup?

It is not a registry cleaner, generic PC optimiser, debloater or AI-controlled deletion agent. The scanner and cleanup decisions are deterministic, inspectable and local.

## Build Week origin

WinReclaim started from a real Windows storage investigation. WhatsApp was suspected, but it used only 766 MB. The actual storage pressure came from local AI models, Android emulators, Hugging Face cache, Docker data, Gradle, npm and project build output. A targeted cleanup reclaimed 27.55 GB without touching projects or Ollama models.

## Alpha capabilities

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

### Reclaim by intent

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

Without an API key, scanning, manual planning, cleanup and receipts remain fully available offline.

### Executable adapters

- User temp files older than seven days
- Recognised user-level crash dumps
- `hf cache prune --yes`
- `npm cache clean --force`
- Conservative `docker system prune --force --filter until=168h`

Docker volumes are never included.

### Safety classes

- **Safe now** — narrowly scoped disposable data
- **Rebuild or redownload** — caches that a tool can restore later
- **Review first** — environments, containers and generated project data
- **Protected** — data WinReclaim refuses to add to an automatic plan

## Explicit exclusions

WinReclaim does not:

- clean Prefetch
- modify the registry
- promise fake speed improvements
- delete browser profiles
- remove Ollama models automatically
- prune Docker volumes
- delete Android emulators or SDK packages by raw folder removal
- follow filesystem reparse points during cleanup
- accept arbitrary deletion paths from the frontend
- let GPT execute or broaden a cleanup plan

## Architecture

```text
React UI
  └─ typed Tauri commands
      ├─ scanner
      ├─ deterministic rules
      ├─ optional GPT intent constraints
      ├─ immutable cleanup planner
      ├─ allowlisted cleanup adapters
      ├─ verifier
      └─ receipt persistence
```

Rules identify and explain storage. They cannot execute arbitrary commands. Cleanup behaviour lives in compiled Rust adapters.

See:

- [`docs/architecture.md`](docs/architecture.md)
- [`docs/rules.md`](docs/rules.md)
- [`docs/safety.md`](docs/safety.md)

## Development

Requirements:

- Windows 10 or 11
- Node.js 22+
- Rust stable with the MSVC toolchain
- Visual Studio Build Tools with Desktop development with C++
- WebView2 Runtime

```powershell
npm install
npm run tauri dev
```

Frontend check:

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
cargo clippy --all-targets
```

Build installers:

```powershell
npm run tauri build
```

Artifacts are generated under `src-tauri\target\release\bundle`. The manual **Build Windows installers** GitHub Actions workflow also uploads MSI and NSIS artifacts.

## Privacy

The scan, rules engine, plans, cleanup and receipts run locally. WinReclaim does not upload file paths, project names, directory trees or receipts. No telemetry is included in this alpha.

When reclaim-by-intent is enabled, only the labels, categories, sizes, safety classes and consequences of currently executable findings are sent to the OpenAI Responses API with response storage disabled. The API key remains in the Rust process environment and is never sent to the webview.

## Status

This is an early public alpha built for OpenAI Build Week. Review every cleanup plan before execution and keep backups of important data.

## License

MIT
