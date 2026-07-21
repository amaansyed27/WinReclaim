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
- sufficient free space for the application and any optional Storage Assistant download.

The optional local Storage Assistant requires approximately 1.4 GB for the model plus runtime and working space. Core WinReclaim features do not require it.

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
- the optional Storage Assistant remains uninstalled;
- the optional OpenAI feature remains unavailable unless a key is supplied to the process environment.

Start with a Quick or Balanced scan of the system fixed drive.

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

This can preserve snapshots, receipts, vault entries or optional model files depending on installer behaviour. Review and remove the directory manually only after confirming that no restore payload is needed.

## Remove the optional Storage Assistant

Use Settings to uninstall the assistant. This removes its model/runtime directory without removing scan history, receipts or vault data.

## Clean reinstall

For a clean test environment:

1. restore or back up any required vault entries;
2. uninstall WinReclaim;
3. close remaining WinReclaim processes;
4. remove `%LOCALAPPDATA%\WinReclaim` only when no data is needed;
5. install the latest official release;
6. run a Quick scan and verify version/update state.

## Offline use

After installation, core scanning, findings, planning, cleanup, receipts and vault restoration can operate offline.

Network access is required only for:

- checking/downloading application updates;
- installing the optional model/runtime;
- invoking the optional OpenAI reclaim-by-intent feature.

## Troubleshooting

See [troubleshooting.md](troubleshooting.md) for installer, SmartScreen, updater and runtime failures.
