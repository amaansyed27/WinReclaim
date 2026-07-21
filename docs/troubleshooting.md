# Troubleshooting

This guide covers development, scanning, cleanup, optional-model and release problems. Remove private paths and credentials before sharing logs.

## Development build problems

### `npm` cannot find a script or package

Confirm that PowerShell is in the repository root:

```powershell
Get-Location
Get-Content .\package.json
npm ci
```

Delete only generated dependency state when necessary:

```powershell
Remove-Item .\node_modules -Recurse -Force -ErrorAction SilentlyContinue
npm ci
```

Do not delete `package-lock.json` to work around a dependency error unless the dependency set is intentionally being changed.

### Rust uses the GNU toolchain

WinReclaim expects MSVC on Windows:

```powershell
rustup default stable-x86_64-pc-windows-msvc
rustup target add x86_64-pc-windows-msvc
rustup component add rustfmt clippy
```

Check:

```powershell
rustc -vV
```

The host should include `x86_64-pc-windows-msvc`.

### Linker or Windows SDK errors

Open **Visual Studio Installer** and ensure **Desktop development with C++** and a current Windows SDK are installed. Restart PowerShell after changing Build Tools.

Typical missing components include:

- MSVC x64/x86 build tools;
- Windows 10/11 SDK;
- C++ CMake tools when a dependency requires them.

The normal WinReclaim build does not require an embedded `llama.cpp` C++ build; the optional assistant uses a downloaded sidecar.

### WebView2 errors

Install or repair Microsoft Edge WebView2 Runtime. Tauri uses WebView2 for the desktop UI.

### Port `1420` is already in use

Stop the previous Vite/Tauri process:

```powershell
Get-NetTCPConnection -LocalPort 1420 -ErrorAction SilentlyContinue |
  Select-Object -ExpandProperty OwningProcess -Unique |
  ForEach-Object { Stop-Process -Id $_ -Force }
```

Then restart:

```powershell
npm run tauri dev
```

## Application launch and UI

### Two windows or a console window appear

Development builds can show additional process windows depending on how the application and sidecars are launched. Verify the packaged release before treating development console behaviour as a shipping regression.

If a second WinReclaim UI window appears, check `tauri.conf.json` and runtime window creation code for duplicate labels.

### Blank window

Run frontend validation:

```powershell
npm run check
npm run build
```

Then enable a Rust backtrace:

```powershell
$env:RUST_BACKTRACE="1"
npm run tauri dev
```

Inspect the webview developer console for JavaScript errors and the PowerShell window for Tauri command failures.

## Scan problems

### No drives are listed

- Confirm the process is running as the same user who can see the drive.
- Verify the drive is mounted and has a Windows drive letter.
- Check whether it is fixed, removable or network storage.
- Restart after attaching a new drive.

Removable and network drives may be inspection-only by design.

### A scan appears stuck

Deep and Ultra scans can inspect many directories. Check the current progress phase and discovered bytes. Try:

1. cancel the scan;
2. run Quick on one fixed drive;
3. exclude broad AppData/dynamic discovery;
4. check the drive for filesystem or access errors;
5. capture sanitized progress details if the same target always stalls.

Locked and inaccessible entries should be skipped rather than blocking indefinitely. Report repeatable hangs.

### A large folder is missing

Possible reasons:

- it is below the selected minimum size;
- the scan profile does not include that category;
- the path is outside selected drives/roots;
- the result limit was reached;
- the folder is protected or part of WinReclaim-owned state;
- it is behind a reparse point;
- a permissions error prevented sizing.

Use Deep or Ultra for broader inspection, but do not assume missing results should automatically become actionable.

### The timeline shows no change

The first scan creates a baseline. Later scans produce deltas only when their roots, profile, options, thresholds, schema and rule-set version are compatible.

A snapshot can be displayed while being ineligible for comparison.

## Planning and cleanup

### A finding cannot be selected

The backend may mark it inspection-only, review-only or protected. This is expected for unknown folders, removable/network storage and sensitive tool data.

WinReclaim does not provide an “ignore safety” switch. Inspect the path and use the owning tool's supported management interface when necessary.

### Estimated reclaim differs from the receipt

The plan contains estimates measured before execution. Files can change, be locked, be removed by another process or compress differently. The receipt reports measured free-space change and action results.

### Cleanup skips files

Common causes:

- active/locked files;
- insufficient permissions;
- reparse points;
- path/fingerprint changed after scan;
- destination moved outside an allowed root;
- antivirus or another process holding a file.

Skipped files are safer than forcing removal. Close the owning application and rescan rather than manually broadening permissions.

### A tool-native cleanup command fails

Run the owning tool directly only after reviewing its own documentation. Examples such as npm, Hugging Face or Docker may require the tool to be installed, on PATH and in a healthy state.

Do not replace a failed command with raw recursive deletion without understanding the consequence.

## Undo Vault

### Restore is unavailable

The entry may be expired, incomplete, missing its payload or already restored. Check the entry status and expiry timestamp.

### Restore refuses to overwrite a file

This is intentional. Move or rename the existing destination after confirming its contents, then retry. WinReclaim never overwrites a file created after cleanup.

### Vault cleanup did not free the estimated space

Moving files within the same volume does not free space. Net reclaim depends on NTFS compression and filesystem allocation. Use the measured receipt rather than the original item size.

## Optional Storage Assistant

### Installation fails

Check:

- internet access to GitHub and Hugging Face;
- available disk space;
- antivirus quarantine;
- whether a partial model/runtime directory exists;
- the exact hash or archive error.

Retry through Settings. Do not disable hash verification.

### Model verifies but inference fails

Possible causes:

- runtime executable quarantined or blocked;
- insufficient memory;
- sidecar process timeout;
- corrupted manifest;
- unsupported CPU/runtime artifact;
- model/runtime version mismatch.

Uninstall the assistant from Settings and reinstall the pinned artifacts. Core scanning remains available without it.

### Assistant output is missing annotations

The validator discards unsupported finding IDs, deletion claims, invalid groups and low-value annotations for already-clear findings. A short or empty advisory result can be correct.

## Optional OpenAI intent feature

### Feature reports no API key

Set the key in the same PowerShell process before launching:

```powershell
$env:OPENAI_API_KEY="your-key"
npm run tauri dev
```

Do not put the key in frontend source or commit it to the repository.

### API failure blocks suggestions

The feature is optional. Continue with manual finding selection. Check network access, key validity, model configuration and API response errors. Never expose the key in an issue.

## Installer and update problems

### Windows shows “Unknown publisher”

Tauri updater signatures verify update integrity but are not Windows Authenticode publisher signatures. An installer without an Authenticode certificate can trigger SmartScreen or an unknown-publisher warning.

Download only from the official GitHub Release and verify the repository/tag.

### Updater says signature is invalid

Do not bypass verification. Check that:

- the installed application contains the expected public key;
- the release was signed with the matching private key;
- `latest.json` references the correct asset and signature;
- the asset was not replaced after the manifest was generated;
- the release is not mixing keys from different builds.

A rotated updater key cannot update installations that trust the old public key unless a signed transition was planned in advance.

### Release workflow rejects the version

Use semantic version text without the `v` prefix, for example:

```text
1.2.0
1.2.1-beta.1
```

Do not reuse an existing tag. The release workflow creates the tag.

### Release workflow cannot push the version commit

A concurrent commit may have reached `main` after the workflow checked it out. Avoid merging to `main` while a release run is between checkout and version commit. Rerun after synchronizing the branch.

## Collecting diagnostics

Useful commands:

```powershell
npm run check
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
cargo test --manifest-path src-tauri/Cargo.toml --all-targets
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

When reporting a problem, include the command, error, version and sanitized environment details. Follow [SUPPORT.md](../SUPPORT.md) and [SECURITY.md](../SECURITY.md).
