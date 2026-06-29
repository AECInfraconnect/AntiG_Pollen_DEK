# Rust Discovery Signature Example

This example shows the shape of a Pollek `discovery.signature` plugin package.
It contributes local AI app matching signals without requesting network or
native OS capabilities.

## Files

- `pollek-plugin.json`: manifest with capability, registry, and governance metadata.
- `src/lib.rs`: minimal Rust implementation sketch for a WIT component.

## Build Path

Production plugins should build a `wasm32-wasip2` component that implements the
WIT world declared in the manifest:

```bash
node scripts/pollek-plugin.mjs test-manifest examples/plugins/rust-discovery-signature
cargo component build --release --target wasm32-wasip2
node scripts/pollek-plugin.mjs checksum examples/plugins/rust-discovery-signature/plugin.wasm
node scripts/pollek-plugin.mjs pack examples/plugins/rust-discovery-signature
node scripts/pollek-plugin.mjs publish-local examples/plugins/rust-discovery-signature
```

The helper warns if `plugin.wasm` is not built yet, but still validates the
manifest fields needed by the local marketplace.
