# JavaScript Telemetry Exporter Example

This example demonstrates the package layout for a `telemetry.exporter` plugin.
The manifest intentionally requests `http_out:splunk.example.com:443`, so the
local dashboard must ask for explicit consent before installation.

The JavaScript file is a reference implementation for the payload policy. A
production package should compile it into a WebAssembly component using a JS
component toolchain such as JCO, then place the component at `plugin.wasm`.

## Local Validation

```bash
node scripts/pollek-plugin.mjs test-manifest examples/plugins/js-telemetry-exporter
node scripts/pollek-plugin.mjs pack examples/plugins/js-telemetry-exporter
```

The pack command can be run before `plugin.wasm` exists; it will preserve the
manifest and source files so reviewers can inspect the requested capabilities.
