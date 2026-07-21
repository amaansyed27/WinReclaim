# Installation and Updating

WinReclaim is distributed for Windows x64 through the repository's official GitHub Releases.

## Download

Open:

```text
https://github.com/amaansyed27/WinReclaim/releases/latest
```

Choose one installer:

- **NSIS setup (`*-setup.exe`)** — recommended for most users;
- **MSI (`*.msi`)** — intended for managed or administrative deployment.

Official releases should also contain `.sig` files and `latest.json`. Those files support the in-app updater and are not normally opened manually.

## System requirements

Primary supported target:

- Windows 11 x64;
- WebView2 Runtime;
- sufficient free space for the application and selected cleanup/restore operations;
- an internet connection only for updates and optional cloud assistance.

Version 1.2.1 does not download or install local model weights or an inference runtime.

## Installer verification

Download only from the official `amaansyed27/WinReclaim` GitHub Release page.

Tauri updater signatures protect in-app updates. They are separate from Windows Authenticode code signing. A release can be valid for the Tauri updater while Windows displays **Unknown publisher** or a SmartScreen warning when the installer does not have an Authenticode certificate/reputation.

Do not disable updater signature verification or use a `latest.json` from another repository.

## Install with the NSIS setup

1. Download the latest `*-setup.exe`.
2. Review the browser/Windows download source.
3. Run the installer.
4. Follow the current-user installation prompts.
5. Launch WinReclaim from Start or the installed shortcut.

The configured installer mode targets the current user unless release configuration changes.

## Install with MSI

1. Download the matching `.msi`.
2. Run it through Windows Installer or approved deployment tooling.
3. Apply organizational policies as required.
4. Launch and complete a Quick scan smoke test.

MSI and NSIS install paths may differ. Do not install both simultaneously unless testing upgrade/uninstall behaviour in a disposable environment.

## First launch

On first launch:

- WinReclaim initializes `%LOCALAPPDATA%\WinReclaim`;
- no cleanup runs automatically;
- a first completed scan creates the timeline baseline;
- no model or provider key is requested;
- optional Storage Assistant and reclaim-by-intent controls call the WinReclaim cloud proxy only after explicit user action;
- version 1.2.1 removes the retired `%LOCALAPPDATA%\WinReclaim\models\storage-assistant` directory if it exists.

Start with a Quick or Balanced scan of the system fixed drive.

## Optional cloud assistance

Judges and normal users do not need an OpenRouter key. The desktop app embeds only this public proxy endpoint:

```text
https://winreclaim.vercel.app/api/assistant
```

The provider credential is stored as a server-side Vercel environment secret. Paths, drive labels, usernames, folder names, project names, directory trees and file contents are excluded from assistant requests.

Free model availability can vary. A cloud failure does not block scanning, planning, cleanup, history, receipts or restoration.

## Updates

WinReclaim can check:

```text
https://github.com/amaansyed27/WinReclaim/releases/latest/download/latest.json
```

The update flow:

1. downloads public release metadata;
2. compares the semantic version;
3. downloads the referenced Windows installer;
4. verifies the Tauri signature against the embedded public key;
5. installs only after the update flow is accepted.

An invalid signature must stop installation.

## Updater key continuity

Existing installations trust the public key embedded at build time. Future updates must be signed by the matching private key.

If the official key changes without a signed migration, older installations cannot trust the new release. Users should not manually replace the embedded key or accept unsigned manifests.

## Uninstall

Use Windows **Installed apps** or the appropriate MSI management tool.

Uninstalling the application may not remove all local application data under:

```text
%LOCALAPPDATA%\WinReclaim
```

This can preserve snapshots, receipts or vault entries depending on installer behaviour. Review and remove the directory manually only after confirming that no restore payload is needed.

## Remove retired local assistant files manually

Close WinReclaim, then run:

```powershell
Remove-Item -LiteralPath "$env:LOCALAPPDATA\WinReclaim\models\storage-assistant" -Recurse -Force -ErrorAction SilentlyContinue

$models = "$env:LOCALAPPDATA\WinReclaim\models"
if ((Test-Path $models) -and -not (Get-ChildItem -LiteralPath $models -Force | Select-Object -First 1)) {
    Remove-Item -LiteralPath $models -Force
}
```

This does not remove scan history, receipts or Undo Vault data. Version 1.2.1 performs the same owned-directory cleanup automatically during startup.

## Clean reinstall

For a clean test environment:

1. restore or back up any required vault entries;
2. uninstall WinReclaim;
3. close remaining WinReclaim processes;
4. remove `%LOCALAPPDATA%\WinReclaim` only when no data is needed;
5. install the latest official release;
6. run a Quick scan and verify version/update state.

## Offline use

Core scanning, findings, planning, cleanup, receipts, history and vault restoration operate offline.

Network access is required only for:

- checking or downloading application updates;
- invoking optional Storage Assistant or reclaim-by-intent assistance.

## Troubleshooting

See [troubleshooting.md](troubleshooting.md) for installer, SmartScreen, updater and cloud-assistant failures.
