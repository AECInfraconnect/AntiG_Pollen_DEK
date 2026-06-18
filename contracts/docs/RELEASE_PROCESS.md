# Release Process

Pollen Contracts use GitHub actions for release management.

Upon a tag matching `contracts-v*`:
1. Artifacts are bundled (`.yaml`, `.json`, code).
2. Checksums are generated (`SHA256SUMS`) and signed via `cosign`.
3. A GitHub Release is drafted and published.
