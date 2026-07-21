# Pinned Storage Assistant Sources

WinReclaim downloads the optional Storage Assistant only after explicit user confirmation. The model and runtime are pinned and verified before use.

The standard WinReclaim installer does not bundle either artifact.

## Model

| Field | Value |
| --- | --- |
| Base model | `Qwen/Qwen3.5-2B` |
| Quantization repository | `bartowski/Qwen_Qwen3.5-2B-GGUF` |
| Immutable revision | `915a52556175c333102d04f996380950d35155d9` |
| File | `Qwen3.5-2B-Q4_K_M.gguf` |
| Quantization | `Q4_K_M` |
| Expected SHA-256 | `84aeb7fe40e7b833d71303d7f1b9f9c1991b931b5dbd214e0aa48d56a0af1f85` |
| Approximate expected size | 1.4 GB |
| Licence | Apache-2.0 as documented by the source |

The application uses a URL containing the immutable Hugging Face revision rather than a moving branch name.

## Runtime

| Field | Value |
| --- | --- |
| Project | `ggml-org/llama.cpp` |
| Release tag | `b9993` |
| Asset | `llama-b9993-bin-win-cpu-x64.zip` |
| Executable | `llama-cli.exe` |
| Architecture | Windows x64 CPU |
| Approximate expected archive size | 19.5 MB |
| Licence | MIT |

WinReclaim queries the API for the exact pinned release tag, locates the exact asset name and requires GitHub release metadata to provide a `sha256:` digest. The downloaded archive is hashed while streaming and must match that digest.

The observed runtime digest and byte count are stored in the local Storage Assistant manifest. A previously installed runtime is accepted only when its manifest matches the pinned tag/asset and the executable exists.

## Download and verification sequence

### Runtime

1. Request the pinned `llama.cpp` release metadata.
2. Find the exact Windows CPU asset.
3. Require a valid SHA-256 digest from the asset metadata.
4. Download to a `.partial` file while hashing.
5. Refuse a digest mismatch.
6. Extract into a staging directory using enclosed archive paths only.
7. Confirm `llama-cli.exe` is present.
8. Replace the tag-specific runtime directory atomically where possible.
9. Record digest and byte count in `manifest.json`.

### Model

1. Check whether the expected model file already exists.
2. Hash the existing file and reuse only when it matches the pinned digest.
3. Otherwise remove the unverified file.
4. Download to a `.partial` file while hashing.
5. Refuse a digest mismatch.
6. Rename the verified file into its final location.
7. Record source, digest, byte count and licence in `manifest.json`.

Incomplete files are not reported as verified.

## Local manifest

The manifest records fields including:

- base model;
- quantization;
- model source;
- model SHA-256 and byte count;
- installation timestamp;
- runtime tag and asset;
- runtime SHA-256 and byte count;
- licence summary.

The manifest is evidence for local installation state, not a substitute for hashing a changed model file. Model status rechecks the model byte count/digest contract; runtime status requires pinned provenance and executable presence.

## Archive safety

Runtime extraction uses enclosed ZIP entry names. Absolute paths and entries that escape the destination are rejected.

Changing archive handling requires a threat-model review because a verified archive can still contain unsafe paths if extraction is not constrained.

## Updating an artifact

A model/runtime update must not use an unpinned “latest” reference.

Required review:

1. identify an immutable revision or tag;
2. identify the exact filename;
3. verify the upstream licence;
4. record expected model digest or runtime digest source;
5. test clean install, interrupted download and digest mismatch;
6. test safe archive extraction;
7. test inference and structured-output validation;
8. run the fixed Storage Assistant evaluation suite;
9. update this document, `storage-assistant.md` and `THIRD_PARTY_NOTICES.md`;
10. publish through a signed WinReclaim release.

## Provider availability

The optional assistant depends on availability of the pinned Hugging Face and GitHub artifacts during installation. If either provider is unavailable, WinReclaim's deterministic core remains functional.

## Trust statement

Integrity verification proves that downloaded bytes match the pinned expected artifact. It does not make model output authoritative. The Storage Assistant remains advisory and cannot change cleanup safety or execution fields.
