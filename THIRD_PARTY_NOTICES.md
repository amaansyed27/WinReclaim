# Third-Party Notices

WinReclaim is licensed under the MIT License. It also depends on third-party software and can optionally download third-party model/runtime artifacts. Those components remain subject to their own licences.

This document is a practical summary, not a replacement for the licence text shipped by each dependency. `package-lock.json`, `src-tauri/Cargo.lock`, the relevant upstream repositories and downloaded artifact metadata are the authoritative dependency inventory.

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

## Optional Storage Assistant

The Storage Assistant is not required for WinReclaim's deterministic scan and cleanup workflow.

When installed by the user, WinReclaim downloads:

| Component | Source | Licence |
| --- | --- | --- |
| Qwen3.5-2B GGUF model | Pinned Hugging Face artifact documented in `docs/model-sources.md` | Apache-2.0 as documented by the model source |
| llama.cpp Windows CPU runtime | Pinned upstream GitHub release asset | MIT |

The model and runtime are stored under `%LOCALAPPDATA%\WinReclaim\models\storage-assistant` and can be removed from Settings.

## Optional OpenAI API integration

The reclaim-by-intent feature can use the OpenAI Responses API when the user explicitly provides `OPENAI_API_KEY`. No OpenAI SDK or model weights are bundled with WinReclaim. Use of the service is governed by the user's agreement with OpenAI.

OpenAI, GPT and related marks belong to their respective owners. WinReclaim is an independent project and is not endorsed by or affiliated with OpenAI.

## Installer tooling

Windows installers are produced by Tauri using NSIS and MSI/WiX-compatible packaging components as configured by the build environment. Their upstream licence notices apply to the generated installer components.

## Fonts, images and branding

Repository-created WinReclaim branding and documentation are distributed with the project under the MIT License unless a file states otherwise. Do not assume third-party screenshots, logos or trademarks are relicensed by inclusion in an issue or pull request.

## Adding a dependency

Contributors adding a dependency must:

1. identify its licence and source;
2. verify compatibility with MIT distribution;
3. avoid abandoned or unverifiable binary downloads;
4. pin security-sensitive external artifacts;
5. update this notice when the dependency is user-visible, bundled or downloaded at runtime;
6. include attribution or full licence text when required.

See [docs/licensing.md](docs/licensing.md) for contributor guidance.
