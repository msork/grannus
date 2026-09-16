# Starter manifest

This bootstrap contains 49 source, governance, documentation, configuration,
CI, packaging, and automation files. Key entry points:

- `AGENTS.md`: repository-wide Codex contract;
- `CODEX_START.md`: first-session prompt and GitHub publication commands;
- `PROJECT_SETTINGS.md`: product/platform/quality defaults;
- `ARCHITECTURE.md`: component boundaries and data paths;
- `ROADMAP.md`: evidence-gated milestones;
- `crates/`: initial Rust workspace and deterministic fake vertical slice;
- `docs/protocol.md`: current v1 input datagram authority;
- `docs/threat-model.md`: assets, boundaries, threats, and invariants;
- `.github/`: CI, dependency policy, PR, ownership, and issue templates;
- `scripts/publish-github.sh`: guarded publication helper for `msork/grannus`.

The archive intentionally excludes a `.git` directory, credentials, generated
build output, hardware captures, and proprietary material.
