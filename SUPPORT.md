# Support

WinReclaim is an open-source project maintained on a best-effort basis.

## Where to ask

Use GitHub Issues for reproducible application bugs, installer/updater failures, incorrect classifications, cleanup/restore failures, cloud-assistant failures, documentation errors and focused feature requests.

Before opening an issue:

1. install the latest release;
2. review [docs/troubleshooting.md](docs/troubleshooting.md);
3. search existing issues;
4. reproduce with the smallest safe scan profile;
5. remove personal information and credentials.

## Information to include

- WinReclaim version;
- Windows edition/version/architecture;
- installer type or development build;
- scan profile and selected drive types;
- exact visible error;
- reproduction steps;
- sanitized logs;
- whether the optional OpenRouter-backed Storage Assistant or reclaim-by-intent feature was invoked;
- for cloud errors, the HTTP status and routed model name when shown, but never the request body if it may contain private user text.

Do not paste paths, folder/file names, usernames, project names, credentials, OpenRouter keys, Vercel tokens or updater signing keys.

## Security issues

Do not report vulnerabilities publicly. Follow [SECURITY.md](SECURITY.md).

## Unsupported requests

Support is not guaranteed for:

- modified/unofficial installers;
- unsigned update manifests;
- unsupported Windows versions;
- arbitrary-path deletion;
- registry cleaning, debloating or performance claims;
- recovery of data removed outside the Undo Vault;
- third-party tools whose own cleanup failed;
- custom builds that weaken safety/privacy validation;
- guaranteed availability or quality from OpenRouter's free-model router.

## Data recovery warning

If data appears to have been removed unexpectedly, stop writing to the drive and do not rerun cleanup. Check the Undo Vault first. For irreversible actions, seek appropriate professional recovery guidance; WinReclaim cannot guarantee recovery.

## Response expectations

Response times vary. Complete sanitized reports are easier to address. Duplicates, incomplete reports and out-of-scope requests may be closed with documentation links.