# Privacy and Network Access

WinReclaim is local-first. Scanning, classification, snapshots, planning, cleanup, receipts and vault restoration run on the user's Windows device.

## Data that remains local

WinReclaim does not intentionally upload:

- filesystem paths;
- usernames;
- project names;
- directory trees;
- file contents;
- scan snapshots;
- cleanup plans;
- receipts;
- vault payloads;
- Storage Assistant prompts or outputs.

No product telemetry or analytics service is included in the desktop application.

## Intended network activity

Network access occurs only for specific optional or maintenance features.

### Signed application updates

WinReclaim checks the configured GitHub Releases endpoint for `latest.json` and downloads an update only after user interaction or the application's update flow. Tauri verifies the updater signature against the public key embedded in the application.

The update request does not include scan results or application-data contents.

### Optional local Storage Assistant installation

When the user chooses to install the Storage Assistant, WinReclaim downloads:

- a pinned GGUF model from a fixed Hugging Face revision;
- a pinned Windows CPU runtime from an upstream `llama.cpp` GitHub Release.

The files are verified before use. Local inference does not contact the model or runtime provider.

See [model-sources.md](model-sources.md).

### Optional OpenAI reclaim-by-intent

When `OPENAI_API_KEY` is configured and the user invokes reclaim-by-intent, the Rust backend sends a constrained request to the OpenAI Responses API.

The request may contain:

- backend-generated candidate IDs;
- product/category labels;
- estimated sizes;
- safety classes;
- recovery consequences;
- the user's intent sentence.

The request must not contain:

- absolute or relative filesystem paths;
- usernames;
- project names;
- directory trees;
- file contents;
- arbitrary cleanup commands;
- API keys in model input.

Response storage is disabled by the application request. The API key remains in the Rust process environment and is never exposed to the webview.

The feature is optional. Manual scanning, review, planning, cleanup, timeline, receipts and vault operation remain available without an API key.

## Storage Assistant privacy boundary

The optional local model receives structured information from the completed scan report and runs through a local verified sidecar process. It does not independently traverse the filesystem or read file contents.

Paths and folder labels supplied to the local prompt are treated as untrusted data. Generated output remains advisory and cannot enable cleanup actions.

## Landing page

The static landing page may request public release metadata from the GitHub API to resolve the newest installer links. It does not receive desktop application data.

Vercel or another site host may process ordinary web request metadata according to the host's own policy. This is separate from the WinReclaim desktop application's local data.

## Logs and issue reports

Logs and screenshots can reveal private folder names even when the application itself does not upload them. Before sharing diagnostics:

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
- optional model/runtime files remain until uninstalled;
- environment-provided API keys are not persisted by WinReclaim.

See [data-layout.md](data-layout.md) for exact locations and reset behaviour.

## Third parties

Optional network features are subject to the policies and terms of their providers:

- GitHub for releases and runtime artifacts;
- Hugging Face for the optional model artifact;
- OpenAI for optional reclaim-by-intent requests;
- the landing-page host for normal web access.

WinReclaim is not affiliated with or endorsed by those providers.

## Privacy-impacting changes

A contribution that introduces a new network request, telemetry, crash reporting, remote storage or new model provider must:

1. be opt-in unless required for an explicit user action;
2. document the exact transmitted fields;
3. avoid filesystem paths and file contents by default;
4. expose failure without blocking local core functionality;
5. update this document, the threat model and the changelog;
6. receive explicit maintainer review.
