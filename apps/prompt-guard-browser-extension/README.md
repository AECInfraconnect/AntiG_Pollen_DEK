# Pollek Prompt Guard Browser Connector

This is the browser connector for Prompt Guard and browser-based AI activity
observe. It sends local Prompt Guard checks to:

`http://127.0.0.1:43891/v1/tenants/local/prompt-guard/check`

It also sends metadata-only browser observe events to:

`http://127.0.0.1:43891/v1/tenants/local/browser-extension/events`

Supported targets generated from the same source:

- Chrome / Chromium browsers on Windows, macOS, and Linux
- Microsoft Edge on Windows, macOS, and Linux
- Safari Web Extension source for macOS packaging through Xcode

Supported AI web surfaces in this first connector:

- ChatGPT: `chatgpt.com`, `chat.openai.com`
- Claude: `claude.ai`
- DeepSeek: `chat.deepseek.com`, `deepseek.com`
- Gemini / Antigravity web surfaces: `gemini.google.com`, `aistudio.google.com`
- Manus web surfaces: `manus.im`, `*.manus.im`
- Microsoft Copilot: `copilot.microsoft.com`
- Perplexity: `perplexity.ai`, `www.perplexity.ai`

The connector stores no raw prompt text in browser storage. Browser observe
events are metadata-only: tab lifecycle, provider, browser, prompt length/hash,
visible response length/hash, attachment counts/extensions, and session metadata.
The observe endpoint refuses raw prompt, response, completion, or content keys.
The Prompt Guard check path can evaluate raw text locally, but the Local Control
Plane stores guard metadata only: categories, rule IDs, action, severity,
counts, source, and text length.

Local mode uses the local deterministic Prompt Guard engine only. Enterprise Cloud mode can add an approved NER enrichment provider later, but that must be explicit enterprise configuration with disclosure and audit metadata.

## Runtime Settings

The Options page controls:

- Prompt Guard local check endpoint
- browser observe endpoint
- observe on/off
- visible response metadata capture on/off
- attachment metadata capture on/off
- warning mode

These settings are stored in browser extension storage. They do not give Pollek
permission to silently install the extension.

## Build

```bash
npm --prefix apps/prompt-guard-browser-extension run build
```

Generated folders:

- `dist/chromium`
- `dist/edge`
- `dist/safari-webextension`

Generated submission packages:

- `packages/pollek-prompt-guard-chromium.zip`
- `packages/pollek-prompt-guard-edge.zip`
- `packages/pollek-prompt-guard-safari-webextension.zip`

See `docs/INSTALL.md` for browser-specific installation and `docs/DEVELOPMENT.md` for adding new providers or future capabilities.

## Installation Limits

Chrome, Edge, and Safari do not allow silent extension installation from a local web dashboard. Users must approve installation, or an organization must deploy it with managed browser policy.
