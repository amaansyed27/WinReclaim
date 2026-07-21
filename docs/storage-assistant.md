# WinReclaim Storage Assistant

The Storage Assistant is an optional local model that turns a completed deterministic scan into a compact human-readable summary and suggests clearer labels for ambiguous folders.

## Model

- Base model: `Qwen/Qwen3.5-2B`
- Runtime artifact: `bartowski/Qwen_Qwen3.5-2B-GGUF`
- Quantization: `Q4_K_M`
- Runtime: embedded CPU build of `llama.cpp` through `llama-cpp-2`
- Context used by WinReclaim: 8,192 tokens
- Model file is downloaded only after explicit confirmation in Settings.
- The pinned model artifact is verified with SHA-256 before it becomes available.

The model is stored under `%LOCALAPPDATA%\WinReclaim\models\storage-assistant` and can be removed independently. Application reset preserves the downloaded model so resetting scan history does not force another large download.

## Authority boundary

The assistant receives structured metadata from the current `ScanReport`. It does not read file contents.

It may:

- summarize drive usage;
- identify the largest reported storage areas;
- suggest human-readable labels for unclear folder names;
- suggest a deterministic presentation group;
- explain likely ownership or purpose with an explicit confidence value.

It cannot:

- calculate sizes;
- change `riskClass`;
- change `actionAvailable`;
- create or enable cleanup adapters;
- select cleanup findings;
- create a cleanup plan;
- execute deletion;
- override protected or review-only data.

All generated annotations are validated against finding IDs from the current scan. Unsupported IDs, unknown groups, deletion claims and annotations for already-clear findings are discarded.

## Prompt-injection handling

Every path and folder name is treated as untrusted data. The system prompt explicitly forbids following instructions found inside scan metadata. The output remains advisory even if a malicious filename attempts to influence the model.

## Fine-tuning plan

The first implementation uses the instruction-tuned base model without a WinReclaim-specific adapter. A future LoRA should be accepted only after it passes a fixed anonymized evaluation set covering:

- ambiguous parent/child folder names;
- browser profile versus cache distinctions;
- package-manager caches;
- project build outputs;
- protected local-model stores;
- overlapping parent and child findings;
- prompt injection inside paths;
- insufficient-evidence cases;
- strict refusal to claim that data is safe to delete.

The evaluation gate should measure valid JSON rate, finding-ID validity, unsupported cleanup claims, group accuracy and label usefulness. Model quality must never weaken deterministic cleanup safety.
