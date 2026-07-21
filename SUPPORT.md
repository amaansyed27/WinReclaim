# Support

WinReclaim is an open-source project maintained on a best-effort basis.

## Where to ask

Use GitHub Issues for:

- reproducible application bugs;
- installer or updater failures;
- incorrect storage classification;
- cleanup or restore failures;
- documentation errors;
- focused feature requests.

Before opening an issue:

1. install the latest published release;
2. review [docs/troubleshooting.md](docs/troubleshooting.md);
3. search existing issues;
4. reproduce with the smallest safe scan profile possible;
5. remove personal information from screenshots and logs.

## Information to include

Provide:

- WinReclaim version;
- Windows edition, version and architecture;
- installation type (`.exe`/NSIS, MSI or development build);
- scan profile and selected drives;
- whether the path is fixed, removable or network storage;
- exact visible error text;
- reproduction steps;
- relevant sanitized logs;
- whether the optional Storage Assistant or OpenAI intent feature was enabled.

Do not paste private folder names, usernames, credentials, API keys or signing keys.

## Security issues

Do not report vulnerabilities publicly. Follow [SECURITY.md](SECURITY.md).

## Unsupported requests

The project cannot guarantee support for:

- modified or unofficial installers;
- unsigned update manifests;
- Windows versions outside the documented requirements;
- deleting arbitrary user-supplied paths;
- registry cleaning, debloating or performance-boost claims;
- recovery of data removed outside the Undo Vault;
- third-party tools whose own cleanup command failed;
- custom builds that weaken path validation or protected-root policy.

## Data recovery warning

Stop writing to the affected drive if data appears to have been removed unexpectedly. Do not repeatedly run cleanup. Check the WinReclaim Undo Vault first. For irreversible actions, use professional recovery guidance appropriate to the storage device; WinReclaim cannot guarantee recovery.

## Response expectations

Response times vary. Complete reports with clear reproduction steps are easier to address. Duplicate, incomplete or out-of-scope issues may be closed with a pointer to existing documentation.
