# Tauri Command API

The React frontend communicates with the Rust backend through typed Tauri commands. This boundary is a security control: the frontend requests operations using domain IDs and structured options, while the backend owns paths, policy, plans and execution.

The frontend wrappers live primarily in `src/lib/tauri.ts`. Rust commands are registered in `src-tauri/src/lib.rs` and implemented under `src-tauri/src/commands` and `assistant_commands.rs`.

## Design rules

- Do not call `invoke` throughout arbitrary components; use typed wrapper functions.
- Do not add a command that accepts an arbitrary deletion path.
- Request and response fields use camelCase across the Tauri boundary.
- The backend must revalidate all IDs and current state.
- Command failures must not leave an executable partial plan.
- Long operations should expose progress or busy state and support cancellation where practical.
- Provider credentials must never cross the Tauri boundary.

## Scan commands

### `list_storage_drives`

Returns available storage roots and metadata used by the drive picker. Fixed drives may support executable cleanup actions. Removable and network drives are inspection-only unless a future reviewed policy changes that rule.

### `start_scan`

```ts
startScan(options: ScanOptions): Promise<ScanReport>
```

The request contains selected roots, scan mode, category toggles, minimum finding size and dynamic-finding limits. Rust normalizes options and creates backend-owned scan state.

Guarantees:

- roots are bounded to selected drives;
- links and reparse points are not followed;
- WinReclaim-owned data is excluded;
- findings have stable IDs for the current scan;
- action availability is assigned by backend policy.

### `cancel_scan`

Requests cooperative cancellation of the active scan.

### `scan-progress` event

```ts
listen<ScanProgress>("scan-progress", handler)
```

Components must unregister listeners during cleanup.

## Findings and insight commands

### `get_reclaim_passports`

```ts
getReclaimPassports(scanId: string): Promise<ReclaimPassport[]>
```

Returns evidence-backed ownership, recovery and confidence information for findings in the specified backend scan.

### `get_storage_timeline`

Returns persisted scan summaries and compatible deltas. A snapshot may be visible without being eligible as a comparison baseline when schema or scan configuration differs.

### `get_ai_status`

Returns whether the optional OpenRouter-backed intent interpreter endpoint is available and provides public router/privacy metadata. It never returns a provider key.

### `interpret_cleanup_intent`

```ts
interpretCleanupIntent(scanId: string, prompt: string): Promise<IntentSuggestion>
```

Rust resolves executable candidates from the current backend-owned scan, converts them to opaque IDs plus category, size, deterministic risk and a generic consequence class, and calls the fixed WinReclaim proxy task.

The response is only an editable selection suggestion. Rust rejects unknown IDs and unsupported classes. It cannot create a target, plan or action.

## Planning and execution commands

### `create_cleanup_plan`

```ts
createCleanupPlan(
  scanId: string,
  findingIds: string[]
): Promise<CleanupPlan>
```

The backend resolves IDs against the current scan. It rejects unknown, protected and non-actionable findings. The immutable plan includes consequences, estimates and a hash. The frontend never supplies action paths.

### `execute_cleanup_plan`

```ts
executeCleanupPlan(
  planId: string,
  planHash: string
): Promise<CleanupReceipt>
```

Rust retrieves the backend-owned plan and verifies the hash before executing compiled adapters. Paths are revalidated immediately before mutation. The receipt contains action results and measured free-space changes.

### `list_receipts`

Returns locally persisted cleanup receipts. Receipts are records, not executable plans.

## Vault commands

### `list_vault_entries`

Returns current restore entries, expiry state, payload size and restore availability.

### `restore_vault_entry`

```ts
restoreVaultEntry(vaultEntryId: string): Promise<RestoreResult>
```

The backend resolves the manifest and payload. Existing destination files are not overwritten. A missing or invalid payload fails safely.

## Local-data management commands

### `get_app_data_summary`

Returns counts and sizes for owned snapshots, receipts and vault entries plus the application-data root.

### `clear_scan_history`

Removes WinReclaim-owned snapshot history only.

### `clear_cleanup_records`

Removes persisted receipt records only. It does not undo cleanup or remove vault payloads.

### `reset_app_data`

```ts
resetAppData(includeRestoreFiles: boolean): Promise<AppDataMutation>
```

By default, reset preserves vault restore data. The user must explicitly choose to remove restore files. Version 1.2.1 separately removes the retired local-assistant model directory during startup.

## Storage Assistant commands

The feature-specific wrappers live under `src/features/assistant`.

### `get_storage_assistant_status`

Returns cloud-endpoint availability, busy state, public provider/router label and the privacy note. It does not make a network request to OpenRouter and cannot expose the server-side credential.

### `analyze_storage_report`

```ts
analyzeStorageReport(scanId: string): Promise<StorageAssistantReport>
```

Rust verifies the requested scan is current, creates aggregate storage metadata and calls the fixed `storage_summary` proxy task. Paths, labels, names and file contents are excluded.

The returned report contains a summary, bounded observations, routed model identifier and `advisoryOnly: true`. It cannot change finding risk, action availability or selection.

The retired `install_storage_assistant` and `uninstall_storage_assistant` commands no longer exist.

## Cloud transport

`src-tauri/src/cloud.rs` posts to the fixed HTTPS endpoint and parses:

```ts
interface CloudEnvelope<T> {
  model: string;
  result: T;
}
```

The desktop may use `WINRECLAIM_ASSISTANT_URL` to point a development run at a preview deployment. Production builds use the WinReclaim Vercel endpoint.

## Adding a command

A new command should include:

1. a Rust request/response type;
2. serde naming compatible with the frontend;
3. a registered handler in `src-tauri/src/lib.rs`;
4. a typed frontend wrapper;
5. tests for valid and invalid input;
6. documentation of authority and side effects;
7. cancellation or timeout behaviour for long work;
8. explicit privacy/network notes when relevant.

Commands that mutate files require a threat-model update and refusal-path tests.

## Compatibility

The frontend and Rust backend ship together, so the command API is not a public external API. Persisted snapshots, receipts, vault manifests and updater metadata require stronger compatibility discipline because they survive application updates.
