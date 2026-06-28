# Pollek Plugin SDK Draft

This document describes the first stable draft of the local Pollek plugin model. The goal is to let third-party extensions improve discovery, observe coverage, definitions, telemetry, and selected control paths without weakening the local privacy and capability model.

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

## Capability Consent

Pollek grants no sensitive capability by default.

Basic host capabilities such as logging and clock access can be granted automatically. Sensitive capabilities require explicit user or admin consent before host functions are linked or activated.

Sensitive examples:

- `http_out`: sends data to another host.
- `native`: uses OS-specific privileged capability providers.
- `kv`: reads or writes plugin state.
- `data_scope`: reads or writes Pollek data such as telemetry, candidates, definitions, or policy suggestions.

The local dashboard must explain these capabilities in human language before install. For example, `http_out:splunk.example.com:443` should appear as "Sends selected telemetry to splunk.example.com".

## Local Testing

During local development:

1. Create a manifest that validates against `contracts/schemas/pollek-plugin.v1.schema.json`.
2. Implement one of the WIT worlds in `contracts/wit/`.
3. Use only capabilities listed in the manifest.
4. Install through the local marketplace or sideload path once available.
5. Confirm the Installed Plugins view shows the granted capabilities and health.
6. Verify plugin activity appears in user-facing Activity or History when the plugin uses a capability.

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

This SDK is a draft contract. The repository now includes WIT draft worlds, a manifest schema, SDK manifest fields, a deny-by-default capability broker, generated marketplace contract models, local persistent installed-plugin registry storage, and user-facing plugin audit events in Activity and History.

Future work still includes signing hardening, remote registry trust roots, update and rollback workflows, richer plugin health checks, examples, PDK/templates, and production submission tooling.
