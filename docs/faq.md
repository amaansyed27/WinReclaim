# Frequently Asked Questions

## What is WinReclaim?

WinReclaim is a local-first Windows storage intelligence and cleanup application. It scans selected drives, explains large storage areas, classifies recovery consequences and executes only reviewed, allowlisted cleanup actions.

## Is it a registry cleaner or PC optimizer?

No. WinReclaim does not modify the registry, debloat Windows or promise generic speed improvements. Its purpose is storage visibility and evidence-based reclaim.

## Does WinReclaim delete files automatically?

No cleanup runs without user review and confirmation. Findings can also be inspection-only, review-first or protected. The backend creates an immutable hashed plan before execution.

## Can it delete any folder I choose?

No. WinReclaim intentionally does not expose arbitrary-path deletion. Executable actions are compiled Rust adapters with validated roots and consequences.

## What does “local-first” mean?

Scanning, classification, snapshots, planning, cleanup, receipts and vault restoration run locally. No desktop telemetry is included.

Optional network access is limited to signed updates and explicit requests for OpenRouter-backed Storage Assistant or reclaim-by-intent help. Those requests use bounded anonymized metadata and never control cleanup execution.

## Does it send my file paths to a cloud model?

No. The optional cloud features exclude filesystem paths, drive labels, usernames, folder names, project names, directory trees and file contents.

Storage summaries send aggregate drive totals and category/risk/action counts. Reclaim-by-intent sends the user's sentence plus opaque candidate IDs, category, size, deterministic risk and a generic consequence class.

## Do I need an OpenRouter API key?

No. The desktop application calls the WinReclaim server-side proxy. The provider credential is stored only as a Vercel environment secret and is never included in the installer or shown to the frontend.

## What is the Storage Assistant?

It is an optional cloud explanation layer using OpenRouter's `openrouter/free` router. It summarizes aggregate metadata from a completed deterministic scan.

It cannot change risk, enable cleanup, select items, create a plan or delete data.

## Is a model bundled or downloaded?

No. Version 1.2.1 does not bundle or download model weights or a local inference runtime. The previous Qwen/`llama.cpp` integration was removed.

Version 1.2.1 automatically deletes the retired `%LOCALAPPDATA%\WinReclaim\models\storage-assistant` directory during startup.

## What happens if free model capacity is unavailable?

The assistant shows a retryable error. Scanning, findings, planning, cleanup, history, receipts and restore continue to work normally because they do not depend on the cloud model.

## What is Storage Time Machine?

Every completed scan creates a bounded local snapshot. Compatible later scans can show growth and reduction by finding/category. The first scan establishes a baseline.

## Why are two scans not comparable?

Comparison requires compatible roots, scan profile, enabled categories, thresholds, schema and rule-set version. WinReclaim avoids presenting an invalid delta when the scan scope changed.

## What is a Reclaim Passport?

A passport explains a finding's likely owner, evidence, recovery class, consequence and confidence. It is generated locally from deterministic rules and scan metadata.

## What is Reclaim Simulation?

Before execution, the plan shows projected free space, estimated reclaim, action counts and the portions that are reversible, redownloadable, rebuildable or irreversible. The final receipt reports measured results.

## What is the Undo Vault?

Eligible user-temp and crash-dump data can be moved into a local manifest-backed vault instead of being immediately destroyed. Entries preserve original relative paths, use a limited retention period and never overwrite existing files during restore.

## Why did moving files to the vault reclaim less space than their size?

Moving files on the same drive does not free space. Net reclaim comes from compression and filesystem allocation changes. WinReclaim reports the measured result.

## Are all cleanup actions reversible?

No. Consequences differ:

- vault-backed actions can be restored during retention;
- caches can usually be redownloaded or rebuilt;
- tool-native prune commands may be irreversible;
- Recycle Bin emptying is irreversible;
- protected items are not executable.

Review the consequence shown for each action.

## Does WinReclaim delete Ollama models?

No. Ollama models can be detected and explained but are protected from automatic cleanup.

## Does it delete browser profiles?

No. Browser profiles and extensions are protected. Some verified disposable browser cache areas may be shown separately when policy supports them.

## Does it delete Docker volumes?

No. Docker volumes are explicitly excluded. A conservative tool-native prune action may remove eligible non-volume data and is labelled irreversible.

## Does it delete Android emulators or SDK packages?

No raw folder deletion is provided for Android virtual devices or SDK packages. They are review-first/protected and should be managed through supported Android tooling.

## Why is a detected folder not selectable?

Detection is not deletion authority. The finding may be unknown, on removable/network storage, protected, review-only or unsupported by an executable adapter.

## Why were files skipped?

They may be locked, inaccessible, protected, behind a reparse point or no longer match the scanned rule. WinReclaim skips uncertain files instead of forcing removal.

## Why does estimated reclaim differ from actual reclaim?

Files can change between scan and execution, be locked, compress differently or already be removed. The receipt measures free space after execution and is the authoritative result.

## Does WinReclaim require administrator access?

Normal user-level scanning and cleanup should run in the current-user context. Some Windows locations can be inaccessible without elevation and will be skipped. WinReclaim should not use elevation to bypass its safety policy.

## Which Windows versions are supported?

The project is developed and released primarily for Windows 11 x64. Other Windows configurations may work but are not the primary test target.

## Why does Windows show an unknown publisher warning?

Tauri updater signatures are different from Authenticode publisher signing. A release can have valid updater integrity signatures while still lacking a commercial Authenticode certificate and reputation.

## How do updates work?

The application retrieves `latest.json` from the official GitHub Release, downloads the referenced installer and verifies its Tauri signature using the embedded public key.

## Can I reset the app?

Settings provides separate controls for scan history, receipt records and broader application reset. Vault restore files are preserved unless explicitly included in the reset.

## Where is local data stored?

```text
%LOCALAPPDATA%\WinReclaim
```

See [data-layout.md](data-layout.md).

## How do I report a bug?

Read [SUPPORT.md](../SUPPORT.md), search existing issues and use the bug-report template. Redact personal paths and credentials.

## How do I report a security vulnerability?

Follow [SECURITY.md](../SECURITY.md) and use GitHub private vulnerability reporting. Do not publish exploit details in a public issue.

## Is WinReclaim affiliated with OpenAI or OpenRouter?

No. WinReclaim was built for the OpenAI Build Week July Edition with GPT-5.6 Sol as a development collaborator, but it is an independent open-source project and is not endorsed by or affiliated with OpenAI, OpenRouter or routed model providers.
