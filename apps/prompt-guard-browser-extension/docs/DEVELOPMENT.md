# Prompt Guard Connector Development

The connector is intentionally layered so new browser surfaces and capture methods can be added without rewriting the whole extension.

## Layers

- `src/shared/constants.js`: endpoint, message names, and shared defaults.
- `src/shared/providerProfiles.js`: provider registry for ChatGPT, Claude, DeepSeek, Gemini, Manus, and future AI web apps.
- `src/content/editorDetector.js`: DOM/editor heuristics shared by provider profiles.
- `src/content/content.js`: capture orchestration and user-visible warnings.
- `src/background/service_worker.js`: Local Control Plane API client and auth handling.
- `src/options/*`: connector settings UI.
- `targets/*/manifest.json`: browser-specific manifests.
- `scripts/build-extension.mjs`: package builder for Chrome/Edge/Safari-compatible outputs.

## Adding an AI Web App

Add a provider profile in `src/shared/providerProfiles.js`:

```js
{
  id: "new-agent-browser",
  label: "New Agent",
  hosts: ["new-agent.example"],
  editorSelectors: ["textarea", "[contenteditable='true']"],
  sendButtonSelectors: ["button[type='submit']"]
}
```

Then add the host to each target manifest's `host_permissions` and `content_scripts.matches`.

## Future Capability Hooks

The current connector captures prompt submissions. Future modules can add:

- response observation for supported DOM structures,
- provider-specific usage/cost extraction where visible locally,
- optional high-risk blocking mode,
- per-site consent prompts,
- native messaging for local auth token bootstrap,
- enterprise policy templates for managed deployments,
- Enterprise Cloud NER enrichment with approved third-party providers.

Local mode must remain local-only. Cloud or third-party NER enrichment needs explicit enterprise configuration, user/admin disclosure, provider metadata, and audit events.
