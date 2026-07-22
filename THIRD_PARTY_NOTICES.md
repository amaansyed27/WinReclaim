# Third-Party Notices

WinReclaim is licensed under the MIT License and depends on third-party software and optional cloud services. Those components and services remain subject to their own licences and terms.

This document is a practical summary, not a replacement for upstream licence text. `package-lock.json`, `src-tauri/Cargo.lock` and the relevant upstream repositories are the authoritative dependency inventory.

## Application dependencies

Major direct dependencies include:

| Component | Purpose | Licence summary |
| --- | --- | --- |
| Tauri 2 | Native desktop shell and IPC | MIT or Apache-2.0 upstream licensing |
| React | User interface | MIT |
| Vite | Frontend build tooling | MIT |
| TypeScript | Frontend language tooling | Apache-2.0 |
| Rust crates in `Cargo.lock` | Scanning, serialization, hashing, HTTP, Windows APIs and packaging | Individual upstream licences |
| WebView2 Runtime | Windows webview supplied by Microsoft | Microsoft licence terms |

Transitive dependencies may use other permissive licences. Binary distributors are responsible for reviewing the complete locked dependency graph for their distribution requirements.

## Optional cloud assistance

The Storage Assistant and reclaim-by-intent features use:

| Service | Purpose | Terms |
| --- | --- | --- |
| Vercel | Hosts the WinReclaim landing page and server-side assistant proxy | Vercel service terms |
| OpenRouter | Routes explicit advisory requests through `openrouter/free` | OpenRouter service terms and routed-provider terms |

No model weights, provider SDK or reusable OpenRouter credential are bundled with WinReclaim. The provider key is held only as a Vercel environment secret.

OpenRouter and routed model/provider names and marks belong to their respective owners. WinReclaim is independent and is not endorsed by or affiliated with those providers.

## Retired local assistant

Version 1.2.0 could download a Qwen3.5-2B GGUF model and a `llama.cpp` Windows CPU runtime. Version 1.2.1 removes that integration and deletes its owned `%LOCALAPPDATA%\WinReclaim\models\storage-assistant` directory during startup.

Those artifacts are not bundled, downloaded or used by current releases.

## Build Week attribution

WinReclaim was built for OpenAI Build Week — July Edition with GPT-5.6 Sol. OpenAI, GPT and related marks belong to their respective owners. WinReclaim is an independent project and is not endorsed by or affiliated with OpenAI.

## Installer tooling

Windows installers are produced by Tauri using NSIS and MSI/WiX-compatible packaging components as configured by the build environment. Their upstream licence notices apply to generated installer components.

## Fonts, images and branding

Repository-created WinReclaim branding and documentation are distributed with the project under the MIT License unless a file states otherwise. Do not assume third-party screenshots, logos or trademarks are relicensed by inclusion in an issue or pull request.

## Adding a dependency or service

Contributors adding a dependency or remote service must:

1. identify its licence, source and service terms;
2. verify compatibility with MIT distribution;
3. avoid abandoned or unverifiable binary downloads;
4. keep reusable credentials outside distributed clients;
5. document transmitted fields and retention assumptions;
6. update this notice when the component or service is user-visible;
7. include attribution or full licence text when required.

See [docs/licensing.md](docs/licensing.md) for contributor guidance.
