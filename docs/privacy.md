# Privacy and Network Access

WinReclaim is local-first. Scanning, classification, Storage Brief generation, reclaim-by-intent parsing, snapshots, planning, cleanup, receipts, and vault restoration run on the user's Windows device.

## Data that remains local

WinReclaim does not intentionally upload:

- filesystem paths;
- drive roots or labels;
- usernames;
- folder or file names;
- project names;
- directory trees;
- file contents;
- scan metadata or snapshots;
- intent text;
- cleanup plans;
- receipts;
- vault payloads.

No product telemetry, analytics, crash-reporting service, remote model provider, or cloud assistant is included in the desktop application.

## Intended desktop network activity

### Signed application updates

WinReclaim checks the configured GitHub Releases endpoint for `latest.json`. Update downloads occur through the application update flow and are verified by Tauri against the public key embedded in the application.

Update requests do not include scan results or application-data contents.

### No assistant network access

Storage Brief and reclaim-by-intent run through deterministic local Rust rules. They do not require:

- an API key;
- a model download;
- a provider SDK;
- a hosted function;
- an account;
- a network connection.

The retired OpenRouter/Vercel proxy is not used by version 1.2.1 or later.

## Landing page

The static landing page may request public GitHub Release metadata to resolve the newest installer links. It does not receive desktop application data.

A site host may process ordinary web request metadata according to the host's own policy. This is separate from the WinReclaim desktop application.

## Logs and issue reports

Logs and screenshots can reveal private folder names even when the application does not upload them. Before sharing diagnostics:

- replace the Windows username;
- redact drive labels and project names;
- remove tokens and signing material;
- do not attach snapshots, receipts, or vault files unless sanitized;
- inspect screenshot backgrounds and title bars;
- never share the updater private key.

## Data retention

- snapshots use bounded local retention;
- vault entries have a limited restore window;
- receipts remain until removed through Settings or manual deletion;
- retired local-model files are removed by version 1.2.1 during startup;
- no assistant provider credentials are stored because the production application has no provider integration.

See [data-layout.md](data-layout.md) for exact locations and reset behaviour.

## Third parties

The desktop application's only intended online service is GitHub Releases for update metadata and signed installer downloads.

WinReclaim is not affiliated with or endorsed by GitHub, Microsoft, OpenAI, or other named technology providers.

## Privacy-impacting changes

A contribution that introduces a new network request, telemetry, crash reporting, remote storage, model provider, or credential must:

1. be opt-in unless required for a clearly requested operation;
2. document the exact transmitted fields;
3. avoid filesystem paths, names, and file contents by default;
4. expose failure without blocking local core functionality;
5. keep credentials outside distributed clients;
6. update this document, the threat model, and the changelog;
7. receive explicit maintainer review.
