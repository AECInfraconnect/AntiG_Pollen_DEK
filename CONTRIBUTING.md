# Contributing to Pollek DEK

We welcome contributions from the community!

## Submitting Pull Requests

1. Fork the repository and create your branch from `main`.
2. Make your focused changes.
3. If you've added code that should be tested, add tests.
4. Ensure the test suite passes (`cargo test --workspace`).
5. Run formatting and linting:
   - `cargo fmt --all -- --check`
   - `cargo clippy --workspace -- -D warnings`
6. Make sure your commits follow the Conventional Commits specification.
7. Open a Pull Request and describe the changes.

## Developer Certificate of Origin (DCO)

All contributions must include a DCO sign-off in the commit message.
You can use `git commit -s` to append it automatically.

## Branch Protection and CI Gates

To prevent CI breakages and maintain code quality, the `main` branch is strictly protected:

1. **Pre-push Hooks**: All contributors MUST run `setup-hooks.sh` (or `setup-hooks.ps1` on Windows) to install local git hooks. This enforces formatting, linting, and contract validation before pushing.
2. **Branch Protection Rules (GitHub)**:
   - Require a pull request before merging.
   - Require status checks to pass before merging (Scorecard, Lint, Test, Security).
   - Require linear history.
   - Do not allow bypassing the above settings.
