# Project Governance

WinReclaim currently uses a maintainer-led governance model.

## Maintainer

The project is maintained by Amaan Syed. The maintainer is responsible for repository administration, release signing, security coordination, roadmap decisions and final merge authority.

## Decision principles

Project decisions are guided by this order of priority:

1. prevent unintended data loss;
2. preserve deterministic and inspectable behaviour;
3. protect user privacy and local-first operation;
4. maintain clear recovery consequences;
5. keep the application useful and understandable;
6. improve performance and breadth without weakening the earlier constraints.

A popular feature may be declined when it expands deletion authority without sufficient evidence or recovery guarantees.

## Contributions

Anyone may propose changes through issues and pull requests. Merge decisions consider:

- safety impact;
- correctness and test coverage;
- maintainability;
- consistency with project scope;
- user-facing clarity;
- compatibility with persisted data and signed updates;
- licensing and provenance.

The project does not currently require a contributor licence agreement. Contributions are accepted under the repository's MIT License.

## Releases

Only maintainers with access to the updater signing key may publish official releases. Official artifacts are those attached to the repository's GitHub Releases and accompanied by Tauri updater signatures and `latest.json`.

The private updater key is not shared through the repository. Release access may be expanded only with a documented key-custody process.

## Security decisions

Security reports are handled privately according to [SECURITY.md](SECURITY.md). The maintainer may temporarily withhold details, disable a feature or delay a release when disclosure would place users at risk.

## Changes to governance

Governance may evolve if the project gains regular maintainers or organizational ownership. Material changes will be documented through a pull request so contributors can review the new responsibilities and decision process.
