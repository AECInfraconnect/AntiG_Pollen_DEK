# AI Agent Contribution Instructions

AI agents must not push directly to `main` unless a human maintainer explicitly requests it for that task.

Default workflow:

1. Create a feature branch.
2. Make focused changes.
3. Run required checks.
4. Commit with Conventional Commit format.
5. Open a pull request.
6. Include test evidence and risk notes.

Required checks before PR:

- cargo fmt --all -- --check
- cargo clippy --workspace -- -D warnings
- cargo test --workspace
- cargo deny check
- cargo audit
