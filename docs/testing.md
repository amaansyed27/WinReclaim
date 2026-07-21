# Testing WinReclaim

WinReclaim testing must cover correctness and refusal behaviour. A cleanup tool is not adequately tested when it only proves that deletion succeeds.

## Required local checks

Run from the repository root:

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

## Test categories

### 1. Domain and serialization tests

Test stable domain contracts including:

- scan options and profile defaults;
- finding safety classes;
- cleanup-plan serialization and hashing;
- receipts and vault manifests;
- snapshot schema versions;
- Storage Assistant response validation;
- reset request defaults.

Persisted formats should include compatibility tests before schema changes are merged.

### 2. Rule tests

Every rule should test:

- expected path recognition;
- non-matching neighbouring paths;
- protected-root precedence;
- reparse-point refusal where applicable;
- accurate action availability;
- consequence and safety classification;
- stale fingerprint refusal before execution;
- behaviour when the owning tool is absent.

A folder name such as `cache` alone is not sufficient evidence for destructive behaviour.

### 3. Planner tests

Planner tests should verify:

- only finding IDs from the current scan resolve;
- protected and inspection-only findings cannot enter a plan;
- duplicate IDs do not create duplicate actions;
- estimates are aggregated without overflow;
- plan hashes change when material plan content changes;
- execution refuses an incorrect or stale hash;
- frontend-supplied paths are never accepted.

### 4. Filesystem action tests

Use disposable temporary directories. Test both successful and refused operations:

- allowed exact roots;
- paths outside allowed roots;
- files becoming links after scan;
- junctions and reparse points;
- locked files;
- inaccessible entries;
- nested protected components;
- existing restore destinations;
- partial cleanup and skip reporting;
- before/after measurement.

Never run destructive tests against a real user profile, project directory or system cache.

### 5. Vault tests

Cover:

- manifest creation;
- original relative-path preservation;
- NTFS compression invocation handling;
- restore without overwrite;
- missing payloads;
- expired entries;
- partial restore failures;
- seven-day retention behaviour;
- reset with and without restore-file deletion.

### 6. Scanner tests

Cover:

- Quick, Balanced, Deep and Ultra option expansion;
- selected-drive scoping;
- fixed versus removable/network behaviour;
- cancellation;
- inaccessible directories;
- no reparse-point traversal;
- maximum dynamic finding counts;
- minimum size thresholds;
- WinReclaim-owned data exclusion;
- compatible and incompatible snapshot comparisons.

### 7. Optional AI tests

The OpenAI reclaim-by-intent feature must be testable without a live API call. Use deterministic fixtures for structured outputs and verify:

- unknown candidate IDs are rejected;
- protected candidates are absent from requests;
- invalid safety classes are rejected;
- model output cannot add commands or paths;
- the selector respects explicit exclusions;
- network or schema failures do not disable manual planning.

Do not place real API keys in test files, CI logs or recorded fixtures.

### 8. Storage Assistant tests

Test the local assistant boundary separately from model quality:

- model and runtime hash verification;
- archive extraction path safety;
- executable discovery within the verified runtime;
- failure cleanup for incomplete downloads;
- manifest validation;
- process timeout and non-zero exit handling;
- strict finding-ID validation;
- rejection of deletion claims;
- prompt-injection strings embedded in paths;
- output length and annotation limits;
- no change to risk or action fields.

Model usefulness can be evaluated with the fixed anonymized suite described in `storage-assistant-evaluation.md`, but safety must not depend on a quality score.

## Manual desktop test matrix

Before a stable release, test at minimum:

| Area | Scenario |
| --- | --- |
| Install | Clean NSIS install as a standard user |
| Install | MSI installation and uninstall |
| Launch | First launch with no existing app data |
| Scan | Quick scan of system drive |
| Scan | Multi-drive scan with a second fixed drive |
| Scan | Removable/network drive remains inspection-only |
| Scan | Cancel a long Deep or Ultra scan |
| Timeline | First scan creates baseline; later compatible scan creates delta |
| Findings | Protected and review-only items cannot be auto-selected |
| Plan | Simulation totals and consequences are understandable |
| Cleanup | Reversible user-temp action creates vault entry |
| Cleanup | Rebuildable cache action produces accurate receipt |
| Restore | Restore succeeds when destination is absent |
| Restore | Existing destination is not overwritten |
| Settings | Clear history, clear receipts and reset options |
| Assistant | Install, analyze, uninstall optional local model |
| Updates | Signed update from an older installed version |
| Offline | Core workflow operates without network or API key |

## Release artifact verification

A successful release should contain:

```text
*.exe
*.exe.sig
*.msi
*.msi.sig
latest.json
```

Verify `latest.json`:

- contains the intended semantic version;
- points to the uploaded NSIS artifact;
- uses the `windows-x86_64` platform key expected by Tauri;
- contains the exact generated signature;
- is reachable at the configured latest-release endpoint.

Install the older stable version and update to the new version to test the complete updater path. A same-version install does not validate updater behaviour.

## Test data and privacy

- Use synthetic folder names and disposable drives/directories.
- Redact usernames and project names from screenshots.
- Do not commit real scan snapshots or receipts.
- Do not attach vault payloads to issues.
- Never expose model, API or updater private keys.

## CI

The `CI` workflow validates frontend integrity, production build, Rust formatting, `cargo check`, tests and strict Clippy on Windows. CI is a gate, not a substitute for manual release testing.

## Reporting results

Pull requests should state:

- commands run;
- relevant manual scenarios tested;
- untested areas;
- fixture or environment limitations;
- whether filesystem mutation, updater signing or network behaviour changed.
