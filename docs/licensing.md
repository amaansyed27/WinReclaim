# Licensing

## WinReclaim source and documentation

WinReclaim is distributed under the [MIT License](../LICENSE).

The licence permits use, copying, modification, merging, publication, distribution, sublicensing and sale of copies, provided the copyright and licence notice are preserved.

The software is provided without warranty. A cleanup tool can have significant consequences; distributors and users are responsible for validating their builds and deployment environment.

## Contributions

Unless a contribution states otherwise before acceptance, submitted code and documentation are licensed under the repository's MIT License. Contributors must have the right to provide the material.

Do not submit:

- copied proprietary code;
- model weights without distribution permission;
- reverse-engineered assets with unclear rights;
- third-party logos or screenshots presented as project-owned artwork;
- generated content that reproduces restricted source material;
- dependencies with terms incompatible with the intended distribution.

No contributor licence agreement is currently required.

## Dependency licences

JavaScript and Rust dependencies retain their upstream licences. The lockfiles record the resolved dependency graph:

```text
package-lock.json
src-tauri/Cargo.lock
```

Before adding a dependency:

1. identify the exact upstream project;
2. review its licence and notice requirements;
3. check transitive dependencies when relevant;
4. prefer source dependencies over opaque binaries;
5. document user-visible or runtime-downloaded components;
6. update [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md).

## Optional model and runtime

The optional Storage Assistant downloads components that are not relicensed by WinReclaim:

- Qwen3.5-2B GGUF model — documented as Apache-2.0 by the pinned source;
- `llama.cpp` Windows CPU runtime — MIT.

The source, revision/tag, filenames and verification details are listed in [model-sources.md](model-sources.md).

Anyone redistributing a bundle that includes these optional artifacts must comply with their licences and include required notices. The standard WinReclaim installer does not need to embed them because they are downloaded only after user confirmation.

## OpenAI service and trademarks

The optional reclaim-by-intent feature calls an external OpenAI service using a user-provided API key. The service and model are governed by the user's agreement with OpenAI; they are not licensed under WinReclaim's MIT License.

OpenAI, GPT and related marks belong to their respective owners. “Built for OpenAI Build Week — July Edition with GPT-5.6 Sol” describes the project's development context and does not imply sponsorship, certification, endorsement or affiliation.

## WinReclaim name and branding

The MIT License covers repository code and project-created assets, but it does not automatically grant trademark rights. Forks should avoid presenting themselves as official WinReclaim releases when they change safety policy, updater keys or cleanup behaviour.

Recommended fork practice:

- use a distinct application identifier;
- use a distinct updater endpoint and signing key;
- state that the build is unofficial;
- avoid uploading artifacts to official WinReclaim release channels;
- document changes to safety and privacy behaviour.

## Binary distribution checklist

A distributor should:

- include the MIT licence;
- preserve required third-party notices;
- review locked dependency licences;
- use its own protected updater signing key for a fork;
- verify the installer and update metadata;
- avoid bundling optional artifacts without complying with their terms;
- document any telemetry, network or policy changes;
- avoid implying OpenAI or upstream endorsement.

## Questions

Licensing questions can be raised through a GitHub issue when they do not contain confidential information. This document is general project guidance, not legal advice.
