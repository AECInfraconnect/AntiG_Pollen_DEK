# POLLEK Fingerprint Definition File

The active agent discovery catalog is `crates/dek-fingerprint-defs/data/baseline.v3.json`.
Runtime updates can also be loaded from the signed definition bundle path used by
`dek-fingerprint-defs::DefinitionStore`.

## Top-Level Sections

- `signatures`: process, CLI, config, port, package, and install markers for AI agents.
- `installed_app_signatures`: desktop and CLI applications found by install paths or app markers.
- `web_ai_signatures`: browser-hosted AI surfaces such as ChatGPT, Claude, Gemini, DeepSeek, Copilot, and Perplexity.
- `browser_processes`: browser executables that should not be emitted as agents by themselves.
- `ai_process_hints`: allowlist and denylist tokens for unknown AI-like process candidates.

## Browser AI Signatures

Each `web_ai_signatures` entry supports:

```json
{
  "id": "chatgpt_web",
  "domain": "chatgpt.com",
  "alias_domains": ["chat.openai.com"],
  "name": "ChatGPT (Web)",
  "vendor": "OpenAI",
  "title_patterns": ["ChatGPT"],
  "app_cmdline_patterns": ["--app=*chatgpt.com*"],
  "capability_tags": ["llm.chat", "web.chat"],
  "risk_weight": 0.5
}
```

Discovery uses these fields across multiple sources:

- Browser session files and SNI match `domain` and `alias_domains`.
- Browser windows match `title_patterns`.
- PWA/app windows match `app_cmdline_patterns`, especially Chromium `--app=https://...`.
- `id` is the stable merge key. Keep it unchanged across definition updates.

## Browser Process Denylist

`browser_processes` describes processes such as `chrome.exe`, `msedge.exe`, `firefox.exe`,
and `safari`. Process scanning skips these entries, because a browser is only a container
for web AI surfaces. Browser-specific sources then emit named candidates such as
`ChatGPT (Web)` or `Claude (Web)`.

## Unknown Process Hints

`ai_process_hints` controls whether unknown process evidence can become an unconfirmed
candidate.

- `require_match: true` means an unknown process must match `name_tokens` or `cmd_tokens`.
- `deny_tokens` removes common vendor helpers, updaters, drivers, and telemetry processes.
- Known signatures still win over hints. Hints only affect unknown candidates.

This protects users from false positives such as generic Dell, NVIDIA, Intel, Realtek,
update, helper, or telemetry processes.

## Adding A New AI

1. Prefer a precise `signatures` or `installed_app_signatures` entry for local apps and CLIs.
2. Add a `web_ai_signatures` entry for browser-hosted AI.
3. Add process names to `browser_processes` only when the process is a browser container.
4. Add broad unknown-process tokens to `ai_process_hints` sparingly.
5. Run:

```powershell
cargo test -p dek-fingerprint-defs -p dek-agent-discovery --locked
cargo clippy -p dek-fingerprint-defs -p dek-agent-discovery -p local-control-plane --all-targets --locked -- -D warnings
```
