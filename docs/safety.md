# Safety model

## Non-negotiable protections

WinReclaim never automatically removes Prefetch, registry data, browser profiles, Ollama models, Docker volumes, Android virtual devices, Android SDK packages, Windows directories, Program Files or project source.

## Plan integrity

The frontend submits finding IDs, not paths. The Rust planner resolves those IDs against the current scan, creates an immutable plan and hashes its complete serialised form. Execution fails when the supplied hash does not match the stored plan.

## Filesystem actions

Filesystem cleanup validates canonical target paths against compiled allowed roots. Reparse points and links are rejected. Locked files are skipped instead of forcefully removed.

## External commands

External adapters use `std::process::Command` with explicit argument arrays. No command is passed through a shell.

## Verification

Free space is measured before and after execution. WinReclaim reports measured reclaim rather than claiming that the original estimate was exact.
