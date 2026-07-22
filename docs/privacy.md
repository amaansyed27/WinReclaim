# Privacy and Network Access

WinReclaim is local-first. Scanning, classification, snapshots, planning, cleanup, receipts and vault restoration run on the user's Windows device.

## Data that remains local

WinReclaim does not intentionally upload:

- filesystem paths;
- drive roots or labels;
- usernames;
- folder or file names;
- project names;
- directory trees;
- file contents;
- scan snapshots;
- cleanup plans;
- receipts;
- vault payloads.

No product telemetry or analytics service is included in the desktop application.

## Intended network activity

Network access occurs only for specific optional or maintenance features.

### Signed application updates

WinReclaim checks the configured GitHub Releases endpoint for `latest.json` and downloads an update only after user interaction or the application's update flow. Tauri verifies the updater signature against the public key embedded in the application.

The update request does not include scan results or application-data contents.

### Optional OpenRouter cloud assistance

When the user explicitly requests a Storage Assistant summary or reclaim-by-intent suggestion, the Rust backend sends bounded metadata to the WinReclaim server-side proxy:

```text
https://winreclaim.vercel.app/api/assistant
```

The proxy stores the OpenRouter credential as a server-side Vercel environment secret and requests the `openrouter/free` router. The provider key is not included in the desktop binary, source code, webview or request payload.

A storage-summary request may contain:

- aggregate used, free and total bytes;
- drive count;
- scanned and skipped entry counts;
- category names;
- reported bytes and location counts per category;
- actionable location counts;
- counts by deterministic risk class;
- an overlap warning for category rows.

A reclaim-by-intent request may additionally contain:

- the user's intent sentence;
- backend-generated opaque candidate IDs;
- candidate category;
- measured size;
- deterministic risk class;
- deterministic recovery consequence.

These requests must not contain:

- filesystem paths;
- drive roots or labels;
- usernames;
- folder or file names;
- project names;
- directory trees;
- file contents;
- arbitrary cleanup commands;
- provider API keys.

The proxy accepts only fixed assistant tasks, validates request size and shape, constrains output with JSON Schema and validates returned fields. The routed model name is returned for transparency. OpenRouter free-model availability and limits can vary.

The feature is optional. Manual scanning, review, planning, cleanup, timeline, receipts and vault operation remain available when the proxy or free-model capacity is unavailable.

## Storage Assistant authority boundary

Remote output is advisory. It cannot independently traverse the filesystem, access local paths, add cleanup targets, change safety classes, create a plan, execute cleanup or restore data.

Rust remains authoritative for IDs, measured values, risk classes, action availability and cleanup execution. Unsafe cleanup claims are rejected before display.

See [storage-assistant.md](storage-assistant.md).

## Landing page and proxy hosting

The landing page may request public release metadata from the GitHub API to resolve the newest installer links. It does not receive desktop application data.

The `/api/assistant` route is a serverless proxy for explicit assistant requests. Vercel and OpenRouter may process ordinary request metadata and the bounded fields described above according to their respective policies. No desktop telemetry is sent through this route.

## Logs and issue reports

Logs and screenshots can reveal private folder names even when the application does not upload them. Before sharing diagnostics:

- replace the Windows username;
- redact drive labels and project names;
- remove API keys and tokens;
- do not attach snapshot, receipt or vault files unless specifically sanitized;
- inspect screenshot backgrounds and title bars;
- avoid sharing the updater private key under any circumstances.

## Data retention

- snapshots use bounded local retention;
- vault entries have a limited restore window;
- receipts remain until removed through Settings or manual deletion;
- retired local-model files are removed by version 1.2.1 during startup;
- provider API keys are not persisted by the WinReclaim desktop application.

See [data-layout.md](data-layout.md) for exact locations and reset behaviour.

## Third parties

Optional network features are subject to the policies and terms of their providers:

- GitHub for releases and updater artifacts;
- Vercel for landing-page and proxy hosting;
- OpenRouter and its routed providers for explicit assistant requests.

WinReclaim is not affiliated with or endorsed by those providers.

## Privacy-impacting changes

A contribution that introduces a new network request, telemetry, crash reporting, remote storage or model provider must:

1. be opt-in unless required for an explicit user action;
2. document the exact transmitted fields;
3. avoid filesystem paths, names and file contents by default;
4. expose failure without blocking local core functionality;
5. keep provider credentials outside distributed clients;
6. update this document, the threat model and the changelog;
7. receive explicit maintainer review.
