# Prompt Guard Connector Development

The connector is intentionally layered so new browser surfaces and capture methods can be added without rewriting the whole extension.

## Layers

- `src/shared/constants.js`: endpoint, message names, and shared defaults.
- `src/shared/providerProfiles.js`: provider registry for ChatGPT, Claude, DeepSeek, Gemini, Manus, Copilot, Perplexity, and future AI web apps.
- `src/content/editorDetector.js`: DOM/editor heuristics shared by provider profiles.
- `src/content/content.js`: capture orchestration, Prompt Guard checks, metadata-only observe events, and user-visible warnings.
- `src/background/service_worker.js`: Local Control Plane API client, browser observe API client, and auth handling.
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
  sendButtonSelectors: ["button[type='submit']"],
  responseSelectors: ["[data-message-author-role='assistant']"],
  attachmentSelectors: ["input[type='file']"]
}
```

Then add the host to each target manifest's `host_permissions` and `content_scripts.matches`.

## Adding Observe Capabilities

Observe modules should send metadata through `MESSAGE_OBSERVE`. Do not add raw
prompt, response, completion, or content fields to observe payloads. If a new
module needs local raw-text safety analysis, send that text only through
`MESSAGE_CHECK` and rely on the Local Control Plane to persist guard metadata.

Recommended payload fields:

- `event_type`
- `provider_id`
- `provider_label`
- `session_id`
- `text_length` plus `text_hash` for prompt metadata
- `response_length` plus `text_hash` for visible response metadata
- `attachment_count`, `attachment_extensions`, and aggregate byte counts
- small `metadata` values that do not contain raw user content

## Future Capability Hooks

The current connector captures prompt submissions, tab lifecycle, attachment
metadata, and visible response metadata. Future modules can add:

- provider-specific usage/cost extraction where visible locally,
- optional high-risk blocking mode,
- per-site consent prompts,
- native messaging for local auth token bootstrap,
- enterprise policy templates for managed deployments,
- Enterprise Cloud NER enrichment with approved third-party providers.

Local mode must remain local-only. Cloud or third-party NER enrichment needs explicit enterprise configuration, user/admin disclosure, provider metadata, and audit events.
