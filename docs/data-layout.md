# Local Data Layout and Lifecycle

WinReclaim stores application-owned state under the current user's local application-data directory:

```text
%LOCALAPPDATA%\WinReclaim
```

If `LOCALAPPDATA` is unavailable in an unusual development environment, the backend falls back to the process temporary directory. Normal Windows installations use `LOCALAPPDATA`.

## Directory layout

```text
%LOCALAPPDATA%\WinReclaim\
├─ data-generation
├─ snapshots\
├─ receipts\
└─ vault\
```

Temporary files may appear during atomic writes or cleanup operations. The current cloud assistant does not store model weights, runtimes, prompts, provider keys or cloud responses under this directory.

## `data-generation`

The backend writes an internal generation identifier to distinguish compatible owned-data layouts. When the generation changes, incompatible snapshots and receipts may be removed during initialization. Vault data has separate retention rules because deleting it can remove recovery capability.

Changing the generation identifier is a migration decision, not a formatting change. Document why old data cannot be read before changing it.

## Snapshots

Location:

```text
%LOCALAPPDATA%\WinReclaim\snapshots
```

A completed scan creates a versioned JSON snapshot used by Storage Time Machine.

Snapshots contain derived metadata such as scan configuration, selected roots, finding identifiers, classifications, measured sizes, timestamps and schema/rule-set versions.

Snapshots are local and are not uploaded. The current retention policy keeps a bounded history rather than growing indefinitely.

## Receipts

Location:

```text
%LOCALAPPDATA%\WinReclaim\receipts
```

A receipt records the outcome of an executed cleanup plan, including plan identity, actions attempted, skipped or failed work, plan estimates, measured free-space change, timestamps and consequences.

Receipts do not contain executable authority. Replaying a receipt must never execute cleanup.

Clearing cleanup records deletes receipt files but does not restore removed data.

## Undo Vault

Location:

```text
%LOCALAPPDATA%\WinReclaim\vault
```

Eligible reversible cleanup moves data into a manifest-backed entry rather than immediately destroying it. Entries preserve original relative paths and have a bounded retention period.

Each entry can include:

- `manifest.json`;
- compressed payload files;
- original destination metadata;
- creation and expiry timestamps;
- restore state.

Restore rules:

- existing destination files are never overwritten;
- path traversal outside the original allowed root is rejected;
- missing or invalid payloads fail safely;
- expired entries are no longer promised as recoverable;
- partial restores are reported explicitly.

Moving a file on the same drive does not by itself reclaim disk space. WinReclaim applies NTFS compression where supported and reports measured net disk-space change.

## Retired local Storage Assistant

Version 1.2.0 could create:

```text
%LOCALAPPDATA%\WinReclaim\models\storage-assistant
```

That directory contained a Qwen GGUF file, a `llama.cpp` runtime, a manifest and temporary request files. Version 1.2.1 no longer uses any of those components and removes this owned directory during startup.

Manual removal after closing WinReclaim is safe:

```powershell
Remove-Item -LiteralPath "$env:LOCALAPPDATA\WinReclaim\models\storage-assistant" -Recurse -Force -ErrorAction SilentlyContinue

$models = "$env:LOCALAPPDATA\WinReclaim\models"
if ((Test-Path $models) -and -not (Get-ChildItem -LiteralPath $models -Force | Select-Object -First 1)) {
    Remove-Item -LiteralPath $models -Force
}
```

This does not remove snapshots, receipts or Undo Vault payloads.

## Cloud assistant state

The desktop application persists no OpenRouter credential. `OPENROUTER_API_KEY` exists only in the Vercel environment for the WinReclaim proxy.

Assistant requests are generated on demand and are not written to a local prompt directory. Returned summaries are held in current UI state and are not added to scan snapshots, receipts or the vault.

## Reset operations

WinReclaim exposes separate operations because the consequences differ:

| Operation | Removes | Preserves |
| --- | --- | --- |
| Clear scan history | Snapshot JSON | Receipts and vault |
| Clear cleanup records | Receipt JSON | Snapshots and vault |
| Reset app data | Most owned state | Vault by default |
| Reset including restore files | Most owned state including vault | Nothing in the app-data root except the generation marker |

The user must explicitly choose to include restore files in reset. UI wording must not imply that clearing records restores data.

## Manual deletion

Developers may remove the application-data directory for disposable testing, but should first:

1. close all WinReclaim processes;
2. confirm no needed vault entries remain;
3. back up any test fixture required for debugging;
4. avoid deleting the directory during cleanup.

For normal users, prefer Settings controls because they expose the correct consequences.

## Backups and migration

WinReclaim does not provide cloud synchronization. Users who back up the directory should treat it as sensitive because folder labels and storage metadata can reveal installed tools or project context.

A migration that changes snapshot, receipt or vault schemas should include explicit schema versioning, compatibility tests, safe refusal for unknown formats, an upgrade or retention policy and documentation/changelog updates.

## Privacy

Application-owned persisted data remains local unless the user manually shares it. Optional assistant requests send only the bounded fields documented in [privacy.md](privacy.md).
