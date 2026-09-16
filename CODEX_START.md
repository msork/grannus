# Starting Grannus with Codex CLI

## One-time setup

```bash
git clone https://github.com/msork/grannus.git
cd grannus
rustup show
cargo test --workspace
```

If the remote repository has not been created yet, unpack this starter, enter
the `grannus` directory, and run:

```bash
git init -b main
git add .
git commit -m "chore: bootstrap Grannus"
gh repo create msork/grannus --public --source=. --remote=origin --push
```

Review the selected MIT license and repository visibility before publishing.
Do not run the final command unless GitHub CLI is authenticated as an account
allowed to create `msork/grannus`.

## First Codex session

Start Codex at the repository root:

```bash
codex
```

Use this prompt:

```text
Read AGENTS.md, PROJECT_SETTINGS.md, ARCHITECTURE.md, ROADMAP.md,
docs/protocol.md, docs/threat-model.md, and the entire current Rust workspace.
Then run the existing checks. Work only on ROADMAP milestone M0, beginning with
the first unchecked item whose prerequisites are met. Before editing, state its
acceptance criteria and identify any current external API or dependency facts
that need verification. Implement the smallest tested vertical slice. Do not
claim real capture, streaming, or Nintendo controller compatibility without
hardware evidence. Update the roadmap and docs only for work actually verified.
Finish by running formatting, Clippy, and all tests, and report exact results.
```

## Subsequent sessions

Use narrow prompts. Example:

```text
Follow AGENTS.md. Continue the first unchecked M0 item in ROADMAP.md. Inspect
existing work and git diff first. Preserve user changes. Add tests before or
with behavior. Run the full quality gate and leave the tree ready for review.
```

For hardware work, include the exact capture card/Bluetooth adapter/GPU, OS,
kernel, logs, and reproduction steps. Ask Codex to save sanitized evidence in
`docs/research/` and to use the truth labels from `AGENTS.md`.

## Suggested development rhythm

1. One issue per measurable vertical slice.
2. Ask Codex for an implementation plan only when the change crosses boundaries.
3. Review the diff and tests locally.
4. Benchmark on named hardware before changing performance claims.
5. Merge small PRs; update ADRs when decisions become hard to reverse.

## Useful commands

```bash
just check             # or run fmt/clippy/test commands directly
cargo run -p grannus-host -- doctor
cargo run -p grannus-host -- fake-session
cargo test -p grannus-protocol
git status --short
git diff --check
```

## What Codex must not do

- Port/copy joycontrol or any other GPL implementation into this MIT repo.
- Treat real-controller-to-PC support as console-facing impersonation proof.
- expose a network listener publicly before authentication exists;
- add real NAT port mappings by default;
- optimize away instrumentation or bounded queues;
- mark roadmap hardware milestones complete from unit tests alone.
