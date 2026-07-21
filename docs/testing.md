# Testing WinReclaim

WinReclaim testing must cover correctness and refusal behaviour. A cleanup tool is not adequately tested when it proves only that deletion succeeds.

## Required checks

```powershell
npm ci
npm run check
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
cargo test --manifest-path src-tauri/Cargo.toml --all-targets
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

Installer-affecting changes also require:

```powershell
npm run tauri build
```

## Core test categories

### Domain and persistence

Test scan options, safety classes, plan serialization/hashing, receipts, vault manifests, snapshot versions, cloud response types and reset defaults. Persisted-schema changes require compatibility or explicit safe invalidation tests.

### Rules and scanner

Every rule should cover positive and neighbouring negative paths, protected precedence, action availability, consequence text, reparse refusal, stale fingerprint refusal and owning-tool absence.

Scanner tests should cover profiles, selected-drive scope, fixed versus removable/network policy, cancellation, inaccessible directories, reparse refusal, thresholds, dynamic-result limits, WinReclaim-data exclusion and snapshot compatibility.

### Planner and actions

Verify that:

- only current-scan finding IDs resolve;
- protected and inspection-only findings cannot enter a plan;
- duplicates do not duplicate actions;
- hashes change with material plan content;
- wrong or stale hashes are refused;
- frontend and cloud output cannot submit paths;
- action tests use disposable temporary roots;
- links, protected overlaps, locked/inaccessible entries and stale fingerprints fail safely;
- receipts reflect partial work and measured results.

Never run destructive tests against a real profile, repository or system cache.

### Undo Vault

Cover manifest creation, relative-path preservation, compression handling, no-overwrite restore, missing/expired payloads, partial restore and reset with/without restore-file deletion.

## Cloud assistant and intent tests

No test should require a personal OpenRouter key.

### Rust payload privacy

Serialize both `storage_summary` and `intent_constraints` requests using synthetic markers and assert that payloads contain none of:

```text
C:\Users\RealName
SecretProject
Personal Drive Label
private-file.txt
```

Verify that summary payloads contain only aggregate drive/category/risk/action counts and intent payloads contain only the user sentence plus opaque IDs, category, size, risk and generic consequence.

### Proxy validation

Test `landing-page/api/assistant.js` with mocked upstream responses:

- methods other than `POST` are rejected;
- unsupported clients/tasks are rejected;
- malformed and oversized bodies are rejected;
- client-selected model/tool/provider fields are not accepted;
- category/candidate limits are enforced;
- `OPENROUTER_API_KEY` is read only server-side;
- 429, timeout, 5xx and malformed upstream output become bounded errors;
- structured string and content-part responses parse correctly;
- unknown candidate IDs/classes are rejected;
- response headers include `no-store` and `nosniff`.

Never log or fixture a real key. Search built frontend, Rust source/binaries and installer resources for key patterns before release.

### Rust response boundary

Verify:

- unknown candidate IDs and unsupported safety classes fail;
- cleanup claims are discarded;
- summary/observation lengths and counts are bounded;
- remote output cannot change measured sizes, risk or action availability;
- remote failure leaves the deterministic scan and manual planner usable;
- routed model identity is presentation metadata only.

### Live smoke test

With a dedicated low-limit demo key configured in Vercel:

1. deploy a preview;
2. point `WINRECLAIM_ASSISTANT_URL` to the preview;
3. generate one storage summary;
4. run one reclaim-by-intent request;
5. test a malformed request and repeated requests;
6. confirm Vercel logs do not contain credentials or full private payloads;
7. verify the production endpoint separately before release.

Free-router capacity can vary, so quality/availability failures must not fail the deterministic core test suite.

See [storage-assistant-evaluation.md](storage-assistant-evaluation.md).

## Manual desktop release matrix

| Area | Scenario |
| --- | --- |
| Install | Clean NSIS install as a standard user |
| Install | MSI installation and uninstall |
| Launch | First launch with no app data |
| Migration | Retired local assistant directory is removed safely |
| Scan | Quick system-drive scan |
| Scan | Multi-drive scan and inspection-only removable/network drive |
| Scan | Cancel Deep/Ultra scan |
| Timeline | First baseline and compatible later delta |
| Findings | Protected/review-only findings cannot be auto-selected |
| Plan | Simulation totals and consequences are clear |
| Cleanup | Vault-backed and rebuildable actions produce accurate receipts |
| Restore | Restore works and never overwrites existing destinations |
| Settings | History, receipts and reset controls |
| Assistant | Cloud ready/loading/success/retry/failure states |
| Intent | Conservative editable selection from anonymized candidates |
| Offline | Core scan/plan/cleanup/history/restore works without network |
| Updates | Signed update from an older installed version |

## Release artifacts

Expected assets:

```text
*.exe
*.exe.sig
*.msi
*.msi.sig
latest.json
```

Verify the semantic version, `windows-x86_64` entry, NSIS URL, signature and latest-release endpoint. Test an actual older-to-newer updater path.

## Privacy and test data

- Use synthetic names and disposable directories.
- Redact usernames, drive labels and projects from screenshots.
- Do not commit real snapshots, receipts or vault payloads.
- Never expose OpenRouter or updater private keys.
- Document every new network field and add a negative privacy assertion.

## CI

The Windows `CI` workflow runs frontend integrity/build checks, Rust formatting, `cargo check`, tests and strict Clippy. CI does not replace manual installer, proxy and updater testing.

Pull requests should state commands run, manual scenarios, untested areas and whether filesystem authority, network payloads, credentials, updater signing or persistence changed.