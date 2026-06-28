importScripts(
  "../shared/webext.js",
  "../shared/constants.js",
  "../shared/providerProfiles.js",
);

const {
  DEFAULT_SETTINGS,
  MESSAGE_CHECK,
  SOURCE_BROWSER_EXTENSION,
} = globalThis.PollekPromptGuardConstants;
const { storageGet, storageSet, api } = globalThis.PollekWebExt;

async function getSettings() {
  const stored = await storageGet(DEFAULT_SETTINGS);
  return { ...DEFAULT_SETTINGS, ...stored };
}

async function checkPrompt(payload) {
  const settings = await getSettings();
  if (!settings.enabled) {
    return { skipped: true, reason: "disabled" };
  }

  const headers = {
    "Content-Type": "application/json",
  };
  if (settings.apiToken) {
    headers.Authorization = `Bearer ${settings.apiToken}`;
  }

  const response = await fetch(settings.endpoint || DEFAULT_SETTINGS.endpoint, {
    method: "POST",
    headers,
    body: JSON.stringify({
      text: payload.text,
      direction: "request",
      agent_id: payload.agentId,
      source: SOURCE_BROWSER_EXTENSION,
      surface: payload.surface,
      session_id: payload.sessionId,
      url: payload.url,
      persist: true,
    }),
  });

  if (!response.ok) {
    const detail = await response.text().catch(() => "");
    throw new Error(
      `Local Prompt Guard returned HTTP ${response.status}${
        detail ? `: ${detail.slice(0, 160)}` : ""
      }`,
    );
  }

  const result = await response.json();
  const action = result?.guard_event?.action || result?.action || "allow";
  return {
    ...result,
    captureMode: settings.captureMode,
    shouldWarn:
      settings.captureMode === "warn_high_risk" &&
      (action === "deny" || action === "redact"),
  };
}

api.runtime.onInstalled.addListener(async () => {
  const current = await storageGet(DEFAULT_SETTINGS);
  await storageSet({ ...DEFAULT_SETTINGS, ...current });
});

api.runtime.onMessage.addListener((message, _sender, sendResponse) => {
  if (message?.type !== MESSAGE_CHECK) {
    return false;
  }

  checkPrompt(message.payload)
    .then((result) => sendResponse({ ok: true, result }))
    .catch((error) =>
      sendResponse({
        ok: false,
        error: error instanceof Error ? error.message : String(error),
      }),
    );
  return true;
});
