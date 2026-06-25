
const fs = require("fs");
const path = require("path");

const localEnvDir = "crates/acceptance-tests/tests/fixtures/local_env";
const uiModesDir = "crates/acceptance-tests/tests/fixtures/ui_modes";

const fixtures = {
  [`${localEnvDir}/windows_mcp_stdio_no_wfp.json`]: {
    os: { type: "windows", version: "11", arch: "x86_64" },
    agents: [{ id: "agent-1", surfaces: ["McpStdio"] }],
    peps: []
  },
  [`${localEnvDir}/windows_no_agents.json`]: {
    os: { type: "windows", version: "11", arch: "x86_64" },
    agents: [],
    peps: ["Wfp"]
  },
  [`${localEnvDir}/macos_nefilter_installed_not_approved.json`]: {
    os: { type: "macos", version: "14", arch: "aarch64" },
    agents: [],
    peps: [{ kind: "NetworkExtension", approved: false }]
  },
  [`${localEnvDir}/macos_mcp_http_ready.json`]: {
    os: { type: "macos", version: "14", arch: "aarch64" },
    agents: [{ id: "agent-2", surfaces: ["McpHttp"] }],
    peps: []
  },
  [`${localEnvDir}/linux_no_ebpf_permission.json`]: {
    os: { type: "linux", version: "Ubuntu", arch: "x86_64" },
    agents: [],
    peps: [{ kind: "eBPF", ready: false, reason: "Permission denied" }]
  },
  [`${localEnvDir}/linux_ebpf_ready.json`]: {
    os: { type: "linux", version: "Ubuntu", arch: "x86_64" },
    agents: [],
    peps: [{ kind: "eBPF", ready: true }]
  },
  [`${localEnvDir}/local_model_ollama_ready.json`]: {
    os: { type: "windows", version: "11", arch: "x86_64" },
    agents: [{ id: "ollama", surfaces: ["LocalModelServer"] }],
    peps: []
  },
  [`${localEnvDir}/browser_ai_extension_missing.json`]: {
    os: { type: "windows", version: "11", arch: "x86_64" },
    agents: [{ id: "browser", surfaces: ["BrowserActivity"] }],
    peps: []
  },
  [`${localEnvDir}/cloud_offline_spool_ready.json`]: {
    cloud_sync_status: "offline",
    local_spool_status: "ready"
  },
  [`${uiModesDir}/desktop_simple.json`]: { mode: "desktop_simple" },
  [`${uiModesDir}/desktop_advanced.json`]: { mode: "desktop_advanced" },
  [`${uiModesDir}/enterprise_server.json`]: { mode: "enterprise_server" },
  [`${uiModesDir}/sovereign_airgap.json`]: { mode: "sovereign_airgap" }
};

for (const [filepath, content] of Object.entries(fixtures)) {
  fs.writeFileSync(filepath, JSON.stringify(content, null, 2));
}

console.log("Fixtures generated.");

