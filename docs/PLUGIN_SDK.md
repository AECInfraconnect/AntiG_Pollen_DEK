# Pollek Plugin SDK

This document describes the local Pollek plugin model. The goal is to let
third-party extensions improve discovery, observe coverage, definitions,
telemetry, and selected control paths without weakening the local privacy and
capability model.

## Extension Points

Pollek plugins are identified by `kind` in `pollek-plugin.v1` manifests.

- `discovery.scanner`: adds local metadata scanners for AI apps, model servers, tools, and agent surfaces.
- `discovery.signature`: contributes signatures that help match well-known AI apps and surfaces.
- `observe.collector`: contributes activity metadata collectors.
- `enforce.method`: adds a host-dependent control method. These must prove real readiness before the UI may call them enforceable.
- `policy.evaluator`: evaluates policy decisions.
- `policy.preset`: contributes user-facing rule presets.
- `telemetry.exporter`: exports selected telemetry to an external destination after consent.
- `telemetry.transform`: transforms local telemetry, such as redaction or normalization.
- `resource.classifier`: classifies observed resources.
- `risk.scorer`: adds risk scoring for activity, agents, tools, or resources.
- `definition.feed`: updates well-known AI app definitions and friendly explanations.
- `notify.channel`: sends local or external notifications.
- `compliance.profile`: adds enterprise compliance mappings for Advanced or enterprise modes.

## Manifest

Plugin manifests use `contracts/schemas/pollek-plugin.v1.schema.json` and are also represented in `dek-plugin-sdk::PluginManifest`.

Required fields:

- `schema_version`: currently `pollek.plugin.v1`
- `id`: reverse-DNS plugin id, for example `com.example.splunk-exporter`
- `name`
- `version`
- `kind`
- `entry`

Recommended fields:

- `wit_world`: the WIT world implemented by the plugin.
- `abi`: `component` or `core`.
- `min_engine_version` and `max_engine_version`.
- `os`: supported operating systems.
- `capabilities`: requested host, HTTP, key-value, native, and data-scope capabilities.
- `config_schema`: JSON schema for plugin settings.
- `author`, `homepage`, `license`.
- `signature`, `sbom`, and `checksum`.
- `registry`: source, OCI/private registry reference, update channel, rollback versions, and revocation id.
- `governance`: review requirements, marketplace eligibility, and trust labels.
- `limits`: memory, fuel, timeout, and maximum output size.

## Capability Consent

Pollek grants no sensitive capability by default.

Basic host capabilities such as logging and clock access can be granted automatically. Sensitive capabilities require explicit user or admin consent before host functions are linked or activated.

Sensitive examples:

- `http_out`: sends data to another host.
- `native`: uses OS-specific privileged capability providers.
- `kv`: reads or writes plugin state.
- `data_scope`: reads or writes Pollek data such as telemetry, candidates, definitions, or policy suggestions.

The local dashboard must explain these capabilities in human language before install. For example, `http_out:splunk.example.com:443` should appear as "Sends selected telemetry to splunk.example.com".

## Trust, Signing, and Review

Plugin loading is fail-closed for signature state. A plugin with
`signature.status` other than `valid` cannot be loaded by the host without an
explicit developer-preview or sideload risk decision.

Marketplace and local registry records should expose these trust labels to the
dashboard:

- `verified`: publisher and artifact are trusted by the configured root.
- `local_only`: the plugin does not request outbound HTTP capability.
- `sends_data_out`: the plugin can send approved telemetry or requests off device.
- `reviewed_native`: native capability use has passed review.
- `developer_preview`: local test or private development artifact.
- `private_registry`: installed from a configured enterprise/private registry.
- `unverified`: no configured trust proof.

Checksums use the `sha256:<hex>` form. Signature metadata can include issuer,
subject, certificate reference, and transparency log reference so future
marketplace tooling can verify Sigstore or enterprise signing flows without
changing the manifest shape.

## Local Testing

During local development:

1. Create a manifest that validates against `contracts/schemas/pollek-plugin.v1.schema.json`.
2. Implement one of the WIT worlds in `contracts/wit/`.
3. Use only capabilities listed in the manifest.
4. Validate, pack, and optionally publish the plugin to the local registry.
5. Confirm the Installed Plugins view shows the granted capabilities and health.
6. Verify plugin activity appears in user-facing Activity or History when the plugin uses a capability.

Helper commands:

```bash
node scripts/pollek-plugin.mjs new tmp/my-plugin com.example.my-plugin
node scripts/pollek-plugin.mjs test-manifest tmp/my-plugin
node scripts/pollek-plugin.mjs checksum tmp/my-plugin/plugin.wasm
node scripts/pollek-plugin.mjs pack tmp/my-plugin
node scripts/pollek-plugin.mjs publish-local tmp/my-plugin
```

`publish-local` writes to `pollek-local-data/plugin-registry` by default. The
Local Control Plane reads that index at runtime. Set
`POLLEK_PLUGIN_REGISTRY_DIR` if a developer or enterprise install keeps the
local plugin registry somewhere else.

Rust helpers are available in the `pollek-pdk` crate:

- `ManifestDraft` creates a manifest with conservative local defaults.
- `parse_manifest` and `validate_manifest_basics` validate manifest shape.
- `sha256_checksum` produces the canonical checksum string.
- `capability_description` turns requested capabilities into user-facing copy.

Example plugin packages live under `examples/plugins/`.

## Lifecycle Operations

Installed plugins are persisted in the Local Control Plane registry and emit
audit events into Activity/History. The local API and dashboard support:

- install with explicit consent
- enable and disable
- health probe
- update
- staged canary rollout
- rollback to the previous local version
- revoke and remove granted capabilities
- uninstall and clear the local plugin namespace

The dashboard must not call a plugin "enforcing" unless the host capability has
passed the relevant OS readiness probe. Plugins can increase observe coverage
even when enforce is unavailable.

## Marketplace Submission

Marketplace distribution is optional for the local product. Local, private registry, and sideload paths remain valid.

A marketplace item should include:

- verified publisher state
- signature or checksum state
- OS support
- plugin kind
- human-readable capabilities
- privacy note
- health behavior
- uninstall and rollback expectations

Native or enforcement plugins require stricter review. They must not claim blocking support unless the host setup and probe evidence prove the method works on the current OS.

## Current Status

The repository now includes WIT worlds, a manifest schema, SDK manifest fields,
a PDK helper crate, a deny-by-default capability broker, fail-closed signature
state checks, generated marketplace contract models, local persistent
installed-plugin registry storage, lifecycle APIs, example plugins, developer
CLI scaffolding, and user-facing plugin audit events in Activity and History.

Still external to the open-source local repo: production public marketplace
operations, commercial cloud approval queues, store billing, and enterprise
trust-root distribution. The local interfaces are intentionally present so
those services can be added without changing the Local Dashboard dependency
boundary.
