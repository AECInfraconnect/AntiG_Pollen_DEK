# POLLEK Definition File (`baseline.v3.json`)

The definition file is the core intelligence base for POLLEK's Auto Discovery system. It uses a structured schema to identify native agents, Web AIs running inside browser tabs, and heuristic filters to prevent false positives.

## Schema Structure

The file uses a self-describing JSON format designed for hot-reloading and synchronization via `dek-bundle-sync`.

### 1. `browser_processes`

Defines known browser executables. Processes listed here will be **ignored** by the generic process scanner and delegated to the `browser_window_scan`.

```json
{
  "process_names": ["chrome.exe", "chrome"],
  "engine": "chromium"
}
```

### 2. `web_ai_signatures`

Defines Web-based AIs. These signatures are matched against open browser window titles and `--app` command line arguments.

```json
{
  "id": "chatgpt_web",
  "domain": "chatgpt.com",
  "name": "ChatGPT (Web)",
  "vendor": "OpenAI",
  "title_patterns": ["ChatGPT"],
  "app_cmdline_patterns": ["--app=*chatgpt.com*"],
  "capability_tags": ["llm.chat"],
  "risk_weight": 0.5
}
```

### 3. `ai_process_hints`

Heuristics to identify potential AI processes that lack formal signatures, while aggressively dropping non-AI bloatware (e.g., `Dell.*`, `Intel.*`).

```json
{
  "require_match": true,
  "name_tokens": ["ollama", "lmstudio"],
  "cmd_tokens": ["langchain", "--model"],
  "deny_tokens": ["dell", "nvidia", "update"]
}
```

### 4. `signatures`

Standard app signatures for desktop and CLI agents (e.g., `Claude Desktop`, `Cursor`).

## How to Add a New Web AI

1. Edit `data/baseline.v3.json` or the relevant update delta.
2. Locate the `web_ai_signatures` array.
3. Append a new object:
   - Provide a unique `id`.
   - Add typical words found in the window title to `title_patterns`.
   - Add PWA command-line matches to `app_cmdline_patterns` if applicable.
4. Distribute the updated definition bundle. `dek-bundle-sync` will automatically download and apply the new definitions without requiring a binary recompilation.
