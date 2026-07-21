# OpenAI Build Week — July Edition

WinReclaim was built for the **OpenAI Build Week — July Edition** with **GPT-5.6 Sol** used as a development collaborator for research, architecture review, implementation support, debugging and documentation.

## Origin

The project began with a practical Windows storage investigation. A commonly suspected application was not the main source of disk pressure. The larger contributors were developer and local-AI workloads such as:

- local model stores;
- Android emulators and SDK data;
- Hugging Face cache;
- Docker data;
- Gradle and npm caches;
- generated project outputs.

A targeted manual cleanup reclaimed significant space without deleting projects or Ollama models. WinReclaim was created to turn that investigation into a repeatable, inspectable and safer product.

## Build Week goals

The project focused on six user questions:

1. What is taking my disk space?
2. What changed since the previous scan?
3. Which tool or workload most likely owns it?
4. What can be reclaimed under deterministic policy?
5. What will removal cost to recover, rebuild or redownload?
6. Can the action be reversed?

The Build Week implementation prioritizes safety architecture over broad deletion coverage.

## How GPT-5.6 Sol was used

GPT-5.6 Sol assisted the development process with tasks such as:

- translating the original investigation into product requirements;
- reviewing module boundaries and cleanup authority;
- reasoning about Windows path and reparse-point hazards;
- generating and reviewing implementation changes;
- debugging Rust, Tauri, packaging and GitHub Actions issues;
- improving user-facing consequence language;
- drafting test plans and developer documentation;
- checking that optional AI features remain advisory.

Human review and repository tests remain responsible for accepted code and releases. Model assistance does not replace security review, filesystem validation or release signing.

## AI inside the product

Development with GPT-5.6 Sol is separate from the application's optional AI features.

WinReclaim can optionally provide:

- **Reclaim by intent:** a constrained OpenAI API request that translates a user goal into conservative selection constraints. It receives anonymized candidate metadata and has no execution authority.
- **Storage Assistant:** an optional local Qwen3.5-2B model that summarizes a completed deterministic scan through a verified local `llama.cpp` sidecar. It cannot change safety or action fields.

The scanner, rules, planner, cleanup adapters, vault, receipts and signed updates do not depend on AI-generated deletion decisions.

## Independent project notice

WinReclaim is an independent open-source project. Participation in or development for Build Week does not imply that OpenAI sponsors, certifies, endorses or maintains the application.

OpenAI, GPT and related names are trademarks of their respective owners.

## Continuing after Build Week

Build Week established the initial product and safety model. Future work should continue to follow the same principles:

- deterministic evidence before action;
- protected data over convenience;
- local-first operation;
- transparent recovery consequences;
- measured receipts rather than marketing claims;
- signed, reproducible release processes;
- explicit boundaries for every AI component.
