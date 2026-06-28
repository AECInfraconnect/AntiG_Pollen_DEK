# Pollek Prompt Guard Browser Connector

This is the real browser connector for Prompt Guard. It observes prompt submissions from supported AI web apps and sends them to the Local Control Plane endpoint:

`http://127.0.0.1:43891/v1/tenants/local/prompt-guard/check`

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

The connector stores no raw prompt text in browser storage. The Local Control Plane check endpoint stores guard metadata only: categories, rule IDs, action, severity, counts, source, and text length.

Local mode uses the local deterministic Prompt Guard engine only. Enterprise Cloud mode can add an approved NER enrichment provider later, but that must be explicit enterprise configuration with disclosure and audit metadata.

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
