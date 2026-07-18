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

## Architecture

```text
React UI
  └─ typed Tauri commands
      ├─ scanner
      ├─ deterministic rules
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
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
```

Build installers:

```powershell
npm run tauri build
```

Artifacts are generated under `src-tauri\target\release\bundle`.

## Privacy

The scan runs locally. WinReclaim does not upload file paths, project names, directory trees or receipts. No telemetry is included in this alpha.

## Status

This is an early public alpha built for OpenAI Build Week. Review every cleanup plan before execution and keep backups of important data.

## License

MIT
