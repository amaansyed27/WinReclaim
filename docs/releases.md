# Releasing WinReclaim for Windows

WinReclaim releases are built on `windows-latest` for Windows 11 x64 and published through the **Release Windows** GitHub Actions workflow.

The workflow:

1. validates the requested semantic version
2. updates `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`
3. runs frontend and Rust validation
4. commits the version change to `main` when required
5. creates the `vX.Y.Z` tag
6. builds MSI and NSIS installers
7. signs updater artifacts
8. publishes the GitHub Release, signatures, and `latest.json`

The in-app updater reads:

```text
https://github.com/amaansyed27/WinReclaim/releases/latest/download/latest.json
```

## One-time signing setup

Tauri updater signatures are mandatory. Never publish an unsigned updater build and never regenerate the key after users install a release, because existing installations trust the public key embedded in the application.

Add these repository secrets under **Settings → Secrets and variables → Actions**:

- `TAURI_SIGNING_PRIVATE_KEY` — complete contents of the WinReclaim updater private key
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` — leave unset or empty for the initial unencrypted key

The matching public key is committed in `src-tauri/tauri.conf.json`. The private key must never be committed, attached to a GitHub Release, or shared publicly.

Keep an offline backup of the private key. Losing it prevents future versions from updating existing WinReclaim installations.

## Publish v1.0.0

1. Open **Actions → Release Windows → Run workflow**.
2. Keep version `1.0.0`.
3. Edit release notes if required.
4. Leave prerelease disabled.
5. Run the workflow.

The workflow refuses to continue if the signing secret is missing. Once complete, verify the release contains:

- an NSIS setup executable
- an MSI installer
- corresponding `.sig` files
- `latest.json`

Install v1.0.0, then use a later version such as `1.0.1` to test the complete in-app update flow.

## Future releases

Run the same workflow with the next semantic version. Do not manually create the tag first; the Tauri action creates the tag and release from the version committed by the workflow.

For normal stable releases, keep **prerelease** disabled. Enable it only for versions intended for test users.
