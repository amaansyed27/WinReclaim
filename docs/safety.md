# Safety model

## Non-negotiable protections

WinReclaim never automatically removes registry data, browser profiles, Ollama models, Docker volumes, Android virtual devices, Android SDK packages, unverified Windows directories, Program Files or project source. Prefetch is exposed only as a manually selected Review first action, restricted to `.pf` files under the exact active Windows Prefetch root.

## AI authority boundary

GPT-5.6 is optional and has no deletion authority. It receives only anonymised metadata for currently executable findings and may return a target size, allowed safety classes, explicit candidate exclusions and explanatory text.

The structured schema does not include paths or commands. Rust independently validates every returned risk class and candidate ID. Protected findings are filtered out before the request and cannot be reintroduced by the response. The resulting suggestion is shown as an editable selection and still requires the normal review and plan-confirmation flow.

## Plan integrity

The frontend submits finding IDs, not paths. The Rust planner resolves those IDs against the current scan, creates an immutable plan and hashes its complete serialised form. Execution fails when the supplied hash does not match the stored plan.

## Filesystem actions

Filesystem cleanup validates canonical target paths against compiled allowed roots. Reparse points and links are rejected. Locked files are skipped instead of forcefully removed. `%TEMP%` remains reversible through the compressed vault; Windows Temp and Prefetch are explicit exact-root actions and are labelled with their recovery consequences.

## External commands

External adapters use `std::process::Command` with explicit argument arrays. No user-controlled command is passed through a shell. Windows command shims such as `npm.cmd` are resolved by compiled platform code. Recycle Bin inspection and emptying use the native Windows Shell API rather than a script.

## Verification

Free space is measured before and after execution. WinReclaim reports measured reclaim rather than claiming that the original estimate was exact.
