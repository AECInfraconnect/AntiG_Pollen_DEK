# Dependency Audit

Pollek uses `cargo audit` and `cargo deny` in CI. Advisory ignores are allowed
only when they are explicit, owned, and time-boxed.

Each `deny.toml` ignored advisory must include a comment immediately above the
advisory with:

- `owner`: accountable team or person.
- `expiry`: date when the ignore must be re-reviewed.
- `issue`: tracking issue.
- `reason`: short justification and expected patch path.

The CI step `node scripts/security/check-deny-ignore-metadata.mjs` fails when
metadata is missing or an ignore has expired. This keeps ignored dependencies
visible while upstream patches are tracked.

Operational rule: when an upstream patch becomes available, update the
dependency and remove the corresponding advisory ignore in the same change.
