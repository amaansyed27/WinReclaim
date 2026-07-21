# WinReclaim Documentation

This directory documents the product, developer workflow, safety boundaries and release process. The source code remains authoritative when documentation and implementation differ; please report inconsistencies.

## Start here

| Document | Audience | Purpose |
| --- | --- | --- |
| [Project README](../README.md) | Everyone | Product overview, downloads and quick start |
| [Development guide](development.md) | Contributors | Windows toolchain, local setup and common workflows |
| [Architecture](architecture.md) | Developers | Module boundaries and end-to-end data flow |
| [Safety model](safety.md) | Users and developers | Non-negotiable cleanup protections |
| [Threat model](threat-model.md) | Security reviewers | Assets, trust boundaries, threats and mitigations |
| [Testing](testing.md) | Contributors | Required checks and safety-focused test strategy |

## Product and behaviour

- [FAQ](faq.md) — common product, privacy and recovery questions.
- [Data layout and lifecycle](data-layout.md) — snapshots, receipts, vault entries, model files and reset behaviour.
- [Privacy and network access](privacy.md) — local data and every intended outbound connection.
- [Rule system](rules.md) — current detection and classification guarantees.
- [Rule authoring](rule-authoring.md) — how to add detections without creating unsafe deletion paths.
- [Storage Assistant](storage-assistant.md) — optional local model design and authority boundary.
- [Pinned model sources](model-sources.md) — immutable model and runtime provenance.

## Developer reference

- [Command API](command-api.md) — typed frontend-to-Rust Tauri command contract.
- [Development guide](development.md) — setup, project layout and coding workflow.
- [Testing](testing.md) — frontend, Rust, installer and manual validation.
- [Troubleshooting](troubleshooting.md) — build, runtime, installer and updater failures.
- [Architecture](architecture.md) — component responsibilities and state transitions.
- [Threat model](threat-model.md) — security assumptions and required controls.

## Project operations

- [Release engineering](releases.md) — versioning, signing, artifacts and updater verification.
- [Licensing](licensing.md) — MIT project licence and third-party obligations.
- [Build Week origin](build-week.md) — project origin and development attribution.
- [Contributing](../CONTRIBUTING.md) — contribution workflow and review expectations.
- [Security policy](../SECURITY.md) — private vulnerability reporting.
- [Support policy](../SUPPORT.md) — issue-reporting guidance.
- [Governance](../GOVERNANCE.md) — decision and release authority.
- [Third-party notices](../THIRD_PARTY_NOTICES.md) — major bundled and optional components.
- [Changelog](../CHANGELOG.md) — release history and unreleased changes.

## Documentation standards

Documentation changes should:

- distinguish shipped behaviour from planned work;
- avoid claiming estimates are measured values;
- use PowerShell examples for Windows-specific commands;
- use relative links within the repository;
- identify any network access explicitly;
- describe recovery consequences for cleanup actions;
- avoid publishing private paths, credentials or signing material.

The documentation is part of the product safety boundary. Changes to execution behaviour are incomplete until the relevant documentation is updated.
