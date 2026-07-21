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
├─ vault\
└─ models\
   └─ storage-assistant\
      ├─ manifest.json
      ├─ Qwen3.5-2B-Q4_K_M.gguf
      └─ runtime\
         └─ <pinned-runtime-tag>\
            └─ llama-cli.exe
```

Additional temporary files may appear during an atomic write, download or archive extraction. Failed installations should clean incomplete artifacts before reporting the assistant as verified.

## `data-generation`

The backend writes an internal generation identifier to distinguish compatible owned-data layouts. When the generation changes, incompatible snapshots and receipts may be removed during initialization. Vault and model data have separate retention rules because deleting either can remove recovery capability or force a large download.

Changing the generation identifier is a migration decision, not a formatting change. Document why old data cannot be read before changing it.

## Snapshots

Location:

```text
%LOCALAPPDATA%\WinReclaim\snapshots
```

A completed scan creates a versioned JSON snapshot used by Storage Time Machine.

Snapshots contain derived metadata such as:

- scan configuration and selected roots;
- finding identifiers and classifications;
- sizes measured during the scan;
- timestamps and rule-set/schema versions;
- data needed for compatible comparisons.

Snapshots are local and are not uploaded. The current retention policy keeps a bounded history rather than growing indefinitely.

A snapshot from an incompatible schema or scan configuration may remain visible but cannot be used as a delta baseline.

## Receipts

Location:

```text
%LOCALAPPDATA%\WinReclaim\receipts
```

A receipt records the outcome of an executed cleanup plan, including:

- plan identity;
- actions attempted;
- skipped or failed work;
- estimated reclaim associated with the plan;
- measured free-space change;
- timestamps and consequences.

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

## Optional Storage Assistant

Location:

```text
%LOCALAPPDATA%\WinReclaim\models\storage-assistant
```

The directory contains the user-initiated optional model download, verified `llama.cpp` CPU sidecar and a manifest describing their expected provenance.

The model directory is preserved by normal application reset so users do not have to repeat a large download. It can be removed independently from Settings.

Do not place the updater private key, OpenAI API key or user scan exports in this directory.

## Reset operations

WinReclaim exposes separate operations because the consequences differ:

| Operation | Removes | Preserves |
| --- | --- | --- |
| Clear scan history | Snapshot JSON | Receipts, vault, models |
| Clear cleanup records | Receipt JSON | Snapshots, vault, models |
| Reset app data | Most owned non-model state | Models and, by default, vault |
| Reset including restore files | Most owned state including vault | Models |
| Uninstall Storage Assistant | Assistant model/runtime directory | Scans, receipts, vault |

The user must explicitly choose to include restore files in reset. UI wording must not imply that clearing records restores data.

## Manual deletion

Developers may remove the application-data directory for disposable testing, but should first:

1. close all WinReclaim processes;
2. confirm no needed vault entries remain;
3. back up any test fixture required for debugging;
4. avoid deleting the directory during cleanup or model download.

For normal users, prefer the Settings controls because they expose the correct consequences.

## Backups and migration

WinReclaim does not currently provide cloud synchronization. Users who back up the directory should treat it as sensitive because folder labels and storage metadata can reveal installed tools or project context.

A migration that changes snapshot, receipt or vault schemas should include:

- explicit schema versioning;
- compatibility tests;
- safe refusal for unknown formats;
- an upgrade or retention policy;
- documentation and changelog updates.

## Privacy

Application-owned data remains local unless the user manually shares it. See [privacy.md](privacy.md) for intended network activity and redaction guidance.
