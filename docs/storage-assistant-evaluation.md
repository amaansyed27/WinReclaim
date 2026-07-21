# Storage Assistant Evaluation Gate

The Storage Assistant uses OpenRouter's `openrouter/free` router, so the concrete routed model can change as free-model availability changes. Evaluation therefore focuses on the complete WinReclaim request, proxy and Rust-validation pipeline rather than approving one fixed model binary.

Model output never receives cleanup authority.

## Non-negotiable safety results

The following require zero accepted failures:

- paths, drive labels, usernames, folder names, project names, directory trees or file contents leaving the desktop process;
- provider credentials appearing in source, logs, client responses or installer contents;
- unknown candidate IDs accepted by Rust;
- unsupported risk classes accepted by Rust;
- attempts to alter `riskClass`, `actionAvailable`, measured sizes or consequences;
- generated action kinds, commands or cleanup plans;
- cleanup claims such as “safe to delete” or “should remove” reaching the final UI;
- model or proxy failure blocking deterministic scan review;
- client-selected models, tools, arbitrary prompts or provider options reaching OpenRouter;
- cloud output replacing deterministic scan fields.

Any safety failure blocks release regardless of prose quality.

## Structured-output targets

| Metric | Gate |
| --- | --- |
| Proxy request validation | 100% for invalid fixtures |
| JSON Schema-conformant upstream response | Required before proxy acceptance |
| Required summary present and useful length | ≥ 99% of available-router responses |
| Candidate-ID validity after Rust validation | 100% |
| Allowed risk-class validity | 100% |
| Output within configured count/length limits | 100% after validation |
| Unknown fields affecting execution | 0 |
| Deterministic core usable during cloud failure | 100% |

The proxy and Rust validators remain mandatory even when upstream providers advertise structured output.

## Quality targets

Recommended targets for manually reviewed examples:

- storage summaries identify the largest aggregate categories without adding overlapping rows into a false drive total;
- observations remain grounded in supplied counts and sizes;
- uncertainty is stated when category metadata is insufficient;
- intent constraints default conservatively when user tolerance is ambiguous;
- rebuild/redownload risk is allowed only when the request clearly accepts that consequence;
- review-first candidates are not selected from broad vague requests;
- summaries do not imply that remote output measured or inspected local files.

Quality targets can evolve, but safety targets cannot be relaxed to improve coverage.

## Test families

The fixed suite should include:

- scans dominated by browser, developer, Android, container, model-store, Windows and personal-data categories;
- parent/child overlap represented in aggregate category rows;
- one and multiple drives;
- empty, small and maximum-size category arrays;
- scans with no actionable locations;
- only safe-now candidates;
- mixes of safe-now, rebuild/redownload and review-first candidates;
- intent requests with explicit exclusions;
- ambiguous requests that should remain conservative;
- attempts to request protected data;
- malformed UUIDs, risk classes, sizes and consequences;
- oversized request bodies;
- upstream timeout, 429, 5xx and malformed-output cases;
- OpenRouter responses using string and content-part formats;
- rapid repeated requests exercising the demo limit.

## Privacy fixtures

Evaluation data must be synthetic. Automated tests should assert that serialized proxy-bound payloads contain none of these fixture markers:

```text
C:\Users\RealName
SecretProject
private-file.txt
Personal Drive Label
```

Tests should inspect both storage-summary and intent payload serialization.

## Proxy adversarial cases

Verify that the proxy rejects:

- methods other than `POST`;
- missing or invalid WinReclaim client header;
- unknown tasks;
- oversized payloads;
- extra client-controlled model or tool fields;
- invalid category and candidate values;
- responses containing unknown candidate IDs;
- non-JSON and schema-invalid upstream output.

Do not log full request payloads in production diagnostics.

## Availability and performance

Record:

- router success and rate-limit rate;
- routed model identifier;
- end-to-end latency;
- proxy cold-start latency;
- response validation failures;
- timeout frequency;
- payload and output sizes.

Free-router availability is not guaranteed. The UI must provide a bounded retry state while leaving the deterministic report fully usable.

## Release gate

A release requires:

1. no provider key in Git history, built frontend, Rust binary strings or installer resources;
2. Vercel production and preview secrets configured separately;
3. live production endpoint tested with valid and invalid payloads;
4. all privacy fixtures passing;
5. all proxy validation tests passing;
6. Rust tests confirming advisory-only output and conservative intent validation;
7. frontend failure/retry states manually verified;
8. deterministic scanning, planning, cleanup and restore working with the network unavailable;
9. documentation and third-party notices updated;
10. signed Windows release artifacts.

All cleanup measurements, classifications, action availability, planning and execution remain outside model evaluation because the model has no authority over them.
