# Troubleshooting

This guide covers development, scanning, cleanup, cloud assistance and release problems. Remove private paths and credentials before sharing logs.

## Development build problems

### `npm` cannot find a script or package

Run from the repository root:

```powershell
Get-Location
Get-Content .\package.json
npm ci
```

When needed, remove only generated dependencies:

```powershell
Remove-Item .\node_modules -Recurse -Force -ErrorAction SilentlyContinue
npm ci
```

Do not delete `package-lock.json` unless intentionally changing dependencies.

### Rust uses the GNU toolchain

```powershell
rustup default stable-x86_64-pc-windows-msvc
rustup target add x86_64-pc-windows-msvc
rustup component add rustfmt clippy
rustc -vV
```

The host should include `x86_64-pc-windows-msvc`.

### Linker or Windows SDK errors

Install **Desktop development with C++**, MSVC x64/x86 tools and a current Windows SDK through Visual Studio Installer. Restart PowerShell afterward.

WinReclaim does not compile or launch a local model runtime.

### WebView2 errors

Install or repair Microsoft Edge WebView2 Runtime.

### Port `1420` is already in use

```powershell
Get-NetTCPConnection -LocalPort 1420 -ErrorAction SilentlyContinue |
  Select-Object -ExpandProperty OwningProcess -Unique |
  ForEach-Object { Stop-Process -Id $_ -Force }

npm run tauri dev
```

## Launch and UI

### Two windows or a console window appear

Development builds can show a console because they are started from PowerShell. The packaged GUI release should not open a local-model sidecar or second console. If a second WinReclaim window appears, inspect `tauri.conf.json` and duplicate window labels.

### Blank window

```powershell
npm run check
npm run build
$env:RUST_BACKTRACE="1"
npm run tauri dev
```

Inspect both the webview console and PowerShell output.

## Scan problems

### No drives are listed

Confirm that the drive is mounted, has a drive letter and is visible to the current user. Removable and network drives may be inspection-only by policy.

### A scan appears stuck

1. Cancel it.
2. Run Quick on one fixed drive.
3. Disable broad AppData/dynamic discovery.
4. Check the drive for access/filesystem errors.
5. Record the sanitized progress phase if the same target repeatedly stalls.

Locked and inaccessible entries should be skipped.

### A large folder is missing

It may be below the threshold, outside selected roots, excluded by the profile, protected, behind a reparse point, inaccessible or beyond the dynamic-result limit. Broader discovery never automatically grants cleanup authority.

### Timeline shows no change

The first scan is a baseline. Later deltas require compatible roots, profile, options, thresholds, schema and rule-set version.

## Planning and cleanup

### A finding cannot be selected

It may be inspection-only, review-first, protected, on unsupported drive storage or missing a compiled adapter. There is no “ignore safety” switch.

### Estimate differs from receipt

Plans contain pre-execution estimates. Files can change, become locked, be removed elsewhere or compress differently. The receipt's measured result is authoritative.

### Cleanup skips files

Common causes: active/locked files, permissions, reparse points, stale fingerprints, protected overlaps or antivirus locks. Close the owning program and rescan rather than broadening deletion manually.

### Tool-native command fails

Confirm that the owning tool is installed and healthy. Do not replace a failed npm, Hugging Face or Docker command with raw recursive deletion.

## Undo Vault

### Restore is unavailable

The entry may be expired, incomplete, missing its payload or already restored.

### Restore refuses to overwrite

Intentional. Review and move/rename the existing destination, then retry. WinReclaim never overwrites a post-cleanup file.

### Vault action freed less than its original size

Moving files within one volume does not itself free space. Net reclaim depends on NTFS compression and allocation. Use the measured receipt.

## Optional cloud assistant

### “Cloud assistant is not configured”

The Vercel project is missing its server-side key. From `landing-page`:

```powershell
vercel link --project winreclaim
vercel env add OPENROUTER_API_KEY production
vercel env add OPENROUTER_API_KEY preview
vercel --prod
```

Enter the key only in the interactive prompt. Do not add it to desktop source, frontend JavaScript, Tauri configuration, GitHub issues or screenshots.

### Preview endpoint is not used

Set the override in the same PowerShell process before launch:

```powershell
$env:WINRECLAIM_ASSISTANT_URL="https://your-preview-domain.vercel.app/api/assistant"
npm run tauri dev
```

Only HTTPS overrides are accepted.

### Free model capacity is busy / HTTP 429

OpenRouter free-model capacity varies. Wait briefly and retry. Do not enable an unbounded paid fallback on the judging key. Manual scanning, selection and cleanup remain available.

### The assistant returns HTTP 403

The proxy requires the WinReclaim desktop client contract. Verify that the request originates through current Rust `cloud.rs`, not a browser form or stale client build.

### The assistant returns HTTP 502

Possible causes:

- OpenRouter or a routed provider is temporarily unavailable;
- no available free model supports required structured parameters;
- upstream returned malformed structured output;
- the proxy deployment is stale.

Redeploy the current `landing-page` root, inspect bounded Vercel logs and retry. Never print the provider key or full private payload.

### Summary failed validation

The proxy and Rust reject malformed output, unsupported candidate IDs/classes and deletion claims. The deterministic scan is unaffected. This is a safe failure, not a reason to weaken validation.

### Old Qwen/`llama.cpp` files remain

Version 1.2.1 removes the retired directory at startup. After closing WinReclaim, manual removal is safe:

```powershell
Remove-Item -LiteralPath "$env:LOCALAPPDATA\WinReclaim\models\storage-assistant" -Recurse -Force -ErrorAction SilentlyContinue
```

This does not remove snapshots, receipts or Undo Vault payloads.

## Installer and updater

### Windows shows “Unknown publisher”

Tauri updater signatures verify update integrity but are not Authenticode publisher signatures. Download only from the official GitHub Release.

### Updater signature is invalid

Do not bypass verification. Confirm the embedded public key, matching private signing key, correct `latest.json`, unchanged assets and consistent key lineage.

### Release workflow rejects the version

Use a new semantic version without `v`, such as `1.2.1` or `1.2.2-beta.1`. Do not reuse an existing tag.

### Release workflow cannot push the version commit

A concurrent `main` update may have landed after checkout. Avoid merging while a release is between checkout and version commit; rerun after synchronization.

## Diagnostics

```powershell
npm run check
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
cargo test --manifest-path src-tauri/Cargo.toml --all-targets
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

When reporting a problem, include the command, application version, sanitized error and environment details. Follow [SUPPORT.md](../SUPPORT.md) and [SECURITY.md](../SECURITY.md).