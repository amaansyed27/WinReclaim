# Architecture

WinReclaim is organised around strict boundaries between discovery, interpretation, judgement and execution.

## Layers

### Scanner

The alpha uses bounded recursive inspection of known developer-tool locations and common project roots. It does not follow links or Windows reparse points. The scanner is hidden behind a module boundary so an NTFS MFT backend and USN Change Journal index can be introduced later without changing the UI or domain objects.

### Rules

Rules convert known paths into semantic findings. A finding contains a category, consequence, confidence and safety class. Rules do not contain shell commands and cannot request arbitrary deletion.

### Optional intent interpreter

When `OPENAI_API_KEY` is available, the Rust backend may send anonymised candidate metadata to the OpenAI Responses API. The model receives category labels, sizes, safety classes and consequence text. It never receives paths, usernames, project names or the file tree.

Strict structured output limits the response to a target size, allowed safety classes, candidate exclusions and a short explanation. A separate deterministic Rust selector validates the response, rejects unknown candidate IDs and maps the constraints onto the current scan. The model cannot construct or execute cleanup actions.

### Planner

The planner accepts finding IDs from the current scan, resolves them in Rust and stores an immutable plan. The plan is hashed. Execution receives only a plan ID and hash, then loads the original plan from backend state.

### Actions

Every action is compiled into an allowlisted Rust adapter. External tools are started directly with argument arrays; WinReclaim never builds shell command strings.

### Verification and receipts

Disk free space is measured before and after execution. Per-action target sizes are also measured when applicable. The final receipt is persisted as local JSON under `%LOCALAPPDATA%\WinReclaim\receipts`.

## Future scanner backends

- NTFS MFT enumeration for initial full-volume inventory
- USN Change Journal updates for incremental refresh
- Tool-native metadata sources for exact sizes and last-use information

These are intentionally outside the alpha until the core safety model is proven.
