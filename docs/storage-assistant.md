# WinReclaim Storage Brief and Local Intent Rules

WinReclaim 1.2.1 removes all model and cloud-provider dependencies from the desktop product.

The former Storage Assistant is now **Storage Brief**, a deterministic local explanation layer for a completed scan. The “Need help choosing?” control also runs locally through conservative intent rules.

Neither feature scans independently, reads file contents, changes cleanup safety, creates actions, creates plans, or executes cleanup.

## Architecture

```text
Completed ScanReport
  ├─ deterministic Storage Brief aggregation
  └─ deterministic reclaim-by-intent parsing
       └─ validated editable candidate selection
```

There is no model process, sidecar, API key, HTTP client, hosted function, provider SDK, or background service.

## Storage Brief

The brief is generated from the current backend-owned `ScanReport` and can describe:

- measured used and free bytes;
- selected drive count;
- number of reported locations;
- number and reported size of verified cleanup actions;
- largest reported categories;
- safety-class distribution;
- skipped entries;
- scan warnings.

Category rows can overlap. Drive totals remain authoritative.

The brief does not receive or need:

- file contents;
- a separate filesystem traversal;
- API credentials;
- network access;
- model weights;
- executable commands.

The returned report remains `advisoryOnly: true`.

## Local reclaim-by-intent

The intent rules accept a bounded user sentence and operate only on existing executable candidates from the current scan.

Supported behaviour:

- parses size targets expressed in KB, MB, GB, or TB;
- defaults to `safe_now` candidates;
- includes `rebuild_or_redownload` only when wording accepts caches, rebuilds, redownloads, dependencies, or named developer-tool categories;
- includes `review_first` only through explicit phrases such as “include review-first actions”;
- excludes matching candidate categories for phrases such as “do not touch”, “exclude”, “keep”, “protect”, “avoid”, “leave”, or “skip”.

The selector then:

1. validates all allowed risk classes;
2. validates exclusion IDs against the current candidate set;
3. sorts by deterministic safety rank and reported size;
4. stops after the requested size target is met;
5. returns an editable suggestion with `remoteUsed: false`.

## Authority boundary

Storage Brief and local intent rules may:

- summarize scan measurements;
- explain category and safety counts;
- propose an editable selection from existing verified actions.

They cannot:

- calculate authoritative filesystem sizes independently;
- mark a location safe;
- change `riskClass`;
- change `actionAvailable`;
- attach an action kind;
- add a cleanup target;
- access an arbitrary path;
- select protected findings;
- create or modify a cleanup plan;
- run commands;
- delete or restore files.

Planning and execution continue through the deterministic Rust planner and compiled cleanup adapters.

## Retired model migration

Earlier builds could download a Qwen GGUF model and `llama.cpp` runtime to:

```text
%LOCALAPPDATA%\WinReclaim\models\storage-assistant
```

Version 1.2.1 removes that retired directory during startup. Manual removal is also safe after WinReclaim is closed:

```powershell
Remove-Item -LiteralPath "$env:LOCALAPPDATA\WinReclaim\models\storage-assistant" -Recurse -Force -ErrorAction SilentlyContinue

$models = "$env:LOCALAPPDATA\WinReclaim\models"
if ((Test-Path $models) -and -not (Get-ChildItem -LiteralPath $models -Force | Select-Object -First 1)) {
    Remove-Item -LiteralPath $models -Force
}
```

This does not remove snapshots, receipts, or Undo Vault payloads.

## Failure behaviour

A brief or intent parsing error leaves the deterministic scan unchanged. Users can continue reviewing findings and creating cleanup plans manually.

No third-party outage, model capacity limit, key expiry, rate limit, or hosted deployment can block these features.

## Related documentation

- [Privacy and network access](privacy.md)
- [Threat model](threat-model.md)
- [Safety model](safety.md)
- [Testing](testing.md)
