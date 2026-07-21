# Licensing

## WinReclaim source and documentation

WinReclaim is distributed under the [MIT License](../LICENSE). It permits use, copying, modification, publication and distribution provided the copyright and licence notice are preserved.

The software is provided without warranty. Cleanup software can have significant consequences; distributors and users are responsible for validating their builds and environment.

## Contributions

Unless stated otherwise before acceptance, contributions are licensed under the repository's MIT License. Contributors must have the right to submit the material.

Do not submit proprietary code, unlicensed model weights/assets, third-party branding presented as project-owned, restricted generated material or dependencies/services incompatible with distribution.

No contributor licence agreement is currently required.

## Dependencies

JavaScript and Rust dependencies retain their upstream licences. Locked inventories:

```text
package-lock.json
src-tauri/Cargo.lock
```

Before adding a dependency:

1. identify the exact upstream project and version;
2. review licence and notice requirements;
3. inspect relevant transitive dependencies;
4. prefer auditable source dependencies;
5. document bundled or user-visible components;
6. update [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md).

## Optional cloud services

Current WinReclaim releases do not bundle or download model weights or a local inference runtime.

Optional Storage Assistant and reclaim-by-intent requests use:

- **Vercel** for the landing page and server-side proxy;
- **OpenRouter** and its `openrouter/free` router;
- a model/provider selected by OpenRouter for each available free request.

These services and routed outputs are governed by their respective terms and policies, not by WinReclaim's MIT License. The OpenRouter credential is held only as a Vercel environment secret; no provider key or SDK is distributed in the application.

Version 1.2.0 could download Qwen/`llama.cpp` artifacts. Version 1.2.1 removes that retired integration and its owned local directory. Those artifacts are not part of current distribution.

## Build Week and trademarks

“Built for OpenAI Build Week — July Edition with GPT-5.6 Sol” describes the project's development context. GPT-5.6 Sol and Codex assisted development; the current application does not call the OpenAI API.

OpenAI, GPT, Codex, OpenRouter, Vercel, Windows, GitHub and routed provider/model names are marks of their respective owners. Their mention does not imply sponsorship, certification, endorsement or affiliation.

## WinReclaim name and branding

MIT covers repository code and project-created assets but does not automatically grant trademark rights. Forks should not present modified safety policy, updater keys or cleanup behaviour as an official WinReclaim release.

Recommended fork practice:

- use a distinct application identifier;
- use a distinct updater endpoint and signing key;
- use separate cloud credentials/endpoints;
- state that the build is unofficial;
- document safety, privacy and network changes.

## Binary distribution checklist

A distributor should:

- include the MIT licence;
- preserve required notices;
- review locked dependencies;
- use its own protected signing key for a fork;
- verify installer and updater metadata;
- keep provider credentials out of distributed clients;
- document cloud services, telemetry and transmitted fields;
- avoid implying upstream endorsement.

See [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md). This document is general project guidance, not legal advice.