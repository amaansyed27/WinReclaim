# WinReclaim Storage Assistant

The Storage Assistant is an optional cloud explanation layer for a completed deterministic scan. It uses OpenRouter's Free Models Router through the WinReclaim server-side proxy.

It does not scan the filesystem, calculate authoritative sizes, classify cleanup safety, select findings, create plans or execute cleanup. Core WinReclaim behaviour remains local and deterministic.

## Architecture

```text
WinReclaim desktop app
  └─ Rust builds anonymized aggregate metadata
      └─ POST https://winreclaim.vercel.app/api/assistant
          └─ Vercel serverless proxy
              └─ OpenRouter model: openrouter/free
```

The OpenRouter API key exists only in the Vercel production environment as `OPENROUTER_API_KEY`. It is never committed to the repository, included in the installer, stored in `%LOCALAPPDATA%`, exposed to the React frontend or returned to the client.

A desktop binary cannot securely contain a provider API key. Any key embedded in source code, Tauri configuration, frontend JavaScript, a Rust string, an installer resource or an environment file shipped with the app can be extracted. The application therefore embeds only the public proxy URL.

## Free model routing

The proxy fixes the requested model to:

```text
openrouter/free
```

OpenRouter selects an available free model that supports the request parameters, including structured output. The routed model name is returned with the report and shown in the interface.

Free model capacity can vary. Requests may be slower or temporarily rate-limited. Failure never blocks access to the deterministic scan report.

## Storage summary input

Before any network request, Rust converts the current scan into bounded aggregate metadata:

- used, free and total bytes;
- number of selected drives;
- scanned and skipped entry counts;
- category names;
- reported bytes per category;
- location count per category;
- actionable location count;
- counts by deterministic risk class;
- an explicit warning that category rows may overlap.

The summary request does **not** include:

- filesystem paths;
- drive roots or labels;
- usernames;
- folder or file names;
- project names;
- file contents;
- directory trees;
- cleanup commands;
- arbitrary executable input.

## Reclaim-by-intent input

The optional “Need help choosing?” control sends a bounded user request plus anonymized executable candidates containing:

- opaque candidate UUID;
- category;
- measured size;
- deterministic risk class;
- deterministic recovery consequence.

It does not send candidate paths, folder names, usernames, project names or file contents.

The model may return only conservative constraints:

- target reclaim bytes;
- allowed risk classes;
- candidate IDs to exclude;
- a short explanation.

Rust rejects unknown IDs and unsupported risk classes, then applies the result only as an editable selection suggestion.

## Proxy restrictions

The serverless function at `landing-page/api/assistant.js`:

- accepts only `POST` requests from the WinReclaim client contract;
- rejects oversized or malformed payloads;
- supports only `storage_summary` and `intent_constraints` tasks;
- fixes the model to `openrouter/free`;
- requires structured JSON Schema output;
- asks OpenRouter to route only to providers supporting required parameters;
- caps input arrays and output tokens;
- applies a basic per-IP demo rate limit;
- validates all returned fields before responding;
- returns bounded error messages;
- never forwards arbitrary model IDs, prompts, tools or provider options from the client.

The OpenRouter key should additionally have a low spending limit or guardrail and should be rotated after public judging.

## Authority boundary

The Storage Assistant may:

- summarize the measured drive picture;
- identify which aggregate categories are largest;
- explain deterministic risk/action counts;
- state uncertainty;
- help translate a user's cleanup preference into conservative constraints.

It cannot:

- mark a location safe;
- change `riskClass`;
- change `actionAvailable`;
- attach an action kind;
- add a cleanup target;
- access a path;
- select protected findings;
- create or modify a cleanup plan;
- run commands;
- delete or restore files.

Storage reports remain `advisoryOnly: true`.

## Output validation

The Vercel proxy requires strict JSON Schema output and validates the response before returning it. Rust then applies an independent boundary:

- summary must meet minimum and maximum lengths;
- observations are length- and count-bounded;
- cleanup claims such as “safe to delete” or “should remove” are discarded;
- model and provider errors do not alter the scan;
- intent IDs must already exist in the current executable candidate set;
- deterministic safety classes remain authoritative.

## Retired local model migration

Versions before 1.2.1 could download a Qwen GGUF model and `llama.cpp` runtime to:

```text
%LOCALAPPDATA%\WinReclaim\models\storage-assistant
```

Version 1.2.1 and later remove that retired directory during application startup. No model, runtime, manifest or prompt file is used by the cloud implementation.

Manual removal is also safe after WinReclaim is closed:

```powershell
Remove-Item -LiteralPath "$env:LOCALAPPDATA\WinReclaim\models\storage-assistant" -Recurse -Force -ErrorAction SilentlyContinue

$models = "$env:LOCALAPPDATA\WinReclaim\models"
if ((Test-Path $models) -and -not (Get-ChildItem -LiteralPath $models -Force | Select-Object -First 1)) {
    Remove-Item -LiteralPath $models -Force
}
```

This does not remove scan history, receipts or Undo Vault data.

## Deployment configuration

Create a dedicated OpenRouter key for the demo. Do not reuse a personal or broad production key.

From the `landing-page` directory:

```powershell
vercel link --project winreclaim
vercel env add OPENROUTER_API_KEY production
vercel env add OPENROUTER_API_KEY preview
vercel --prod
```

Paste the key only into the interactive Vercel prompt. Do not place it in a command, `.env` file committed to Git, GitHub Actions log, issue, screenshot or application source.

The released desktop app already targets:

```text
https://winreclaim.vercel.app/api/assistant
```

For local proxy development, the Rust client can be redirected at runtime:

```powershell
$env:WINRECLAIM_ASSISTANT_URL="https://your-preview-domain.vercel.app/api/assistant"
npm run tauri dev
```

## Related documentation

- [Privacy and network access](privacy.md)
- [Threat model](threat-model.md)
- [Safety model](safety.md)
- [Deployment and releases](releases.md)
