# OpenAI Build Week — July Edition

WinReclaim was built for **OpenAI Build Week — July Edition** with **GPT-5.6 Sol and Codex** used as development collaborators for research, architecture, implementation, debugging, testing and documentation.

## Origin

The project began when the Windows system drive was nearly full. WhatsApp looked like the obvious culprit, but it occupied only about **766 MB**. A Codex-assisted investigation showed that the real storage pressure came from developer workloads:

- local AI model stores;
- Android emulators and SDK data;
- Hugging Face cache;
- Docker data;
- Gradle and npm caches;
- Rust targets, Python environments and generated project output;
- temporary Windows files.

The investigation reclaimed approximately **27.55 GB** without deleting projects or Ollama models. It also consumed a significant amount of Codex usage because the same questions had to be asked repeatedly:

> What created this folder? Is it safe to remove? What will it cost to restore? Can the action be reversed?

WinReclaim turns that one-time investigation into a repeatable Windows product.

## Build Week goals

WinReclaim answers six questions:

1. What is taking my disk space?
2. What changed since the previous scan?
3. Which tool or workload likely owns it?
4. What can be reclaimed under deterministic policy?
5. What will removal cost to recover, rebuild or redownload?
6. Can the action be reversed?

The implementation prioritizes safety architecture over broad deletion coverage.

## How GPT-5.6 Sol and Codex were used

They assisted with:

- translating the original investigation into product requirements;
- reviewing Rust/Tauri module and authority boundaries;
- reasoning about Windows paths, reparse points and recovery risks;
- implementing and debugging the application;
- improving user-facing consequence language;
- packaging signed Windows releases;
- creating the landing page and documentation;
- checking that remote model output remains advisory.

Repository tests, human review and deterministic Rust code remain responsible for accepted behaviour and releases.

## AI inside the product

Development with GPT-5.6 Sol and Codex is separate from the application's runtime intelligence.

Current releases do **not** use the OpenAI API and do **not** bundle or download a local model. Optional assistance uses OpenRouter's `openrouter/free` router through the WinReclaim Vercel proxy.

- **Storage Assistant** receives aggregate drive totals and category/risk/action counts.
- **Reclaim by intent** receives the user's sentence plus opaque candidate IDs, size, category, deterministic risk and recovery consequence.

The desktop app contains no provider API key. Paths, drive labels, usernames, folder names, project names, directory trees and file contents remain local. Remote output cannot add cleanup targets, change risk classes, create plans, run commands or execute deletion.

The scanner, rules, planner, compiled cleanup adapters, Undo Vault, receipts and signed updater remain deterministic.

## Independent project notice

WinReclaim is an independent open-source project. Participation in Build Week does not imply that OpenAI, OpenRouter, Vercel or any routed model provider sponsors, certifies, endorses or maintains it.

All product and company names are trademarks of their respective owners.

## Continuing after Build Week

Future work should retain the same principles:

- deterministic evidence before action;
- protected data over convenience;
- local-first core operation;
- explicit privacy boundaries for optional cloud requests;
- transparent recovery consequences;
- measured receipts rather than marketing claims;
- signed releases and strict credential separation.