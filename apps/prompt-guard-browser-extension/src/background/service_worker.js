importScripts(
  "../shared/webext.js",
  "../shared/constants.js",
  "../shared/providerProfiles.js",
);

const {
  DEFAULT_SETTINGS,
  MESSAGE_CHECK,
  MESSAGE_OBSERVE,
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

function browserName() {
  const ua = navigator.userAgent;
  if (ua.includes("Edg/")) return "Microsoft Edge";
  if (ua.includes("Chrome/")) return "Chrome";
  if (ua.includes("Safari/")) return "Safari";
  return "Browser";
}

async function sendObserveEvent(payload, sender) {
  const settings = await getSettings();
  if (!settings.observeEnabled) {
    return { skipped: true, reason: "observe_disabled" };
  }
  if (
    payload.event_type === "visible_response_metadata" &&
    settings.responseMetadataEnabled === false
  ) {
    return { skipped: true, reason: "response_metadata_disabled" };
  }
  if (
    payload.event_type === "attachment_detected" &&
    settings.attachmentMetadataEnabled === false
  ) {
    return { skipped: true, reason: "attachment_metadata_disabled" };
  }

  const headers = {
    "Content-Type": "application/json",
  };
  if (settings.apiToken) {
    headers.Authorization = `Bearer ${settings.apiToken}`;
  }

  const tab = sender?.tab ?? {};
  const body = {
    schema_version: "pollek.browser_observe_event.v1",
    extension_id: api.runtime.id,
    extension_version: api.runtime.getManifest?.().version,
    browser_id: browserName().toLowerCase().split(" ").join("-"),
    browser_name: browserName(),
    tab_id: typeof tab.id === "number" ? tab.id : undefined,
    window_id: typeof tab.windowId === "number" ? tab.windowId : undefined,
    url: payload.url ?? tab.url,
    title: payload.title ?? tab.title,
    capture_mode: settings.captureMode,
    ...payload,
  };

  const response = await fetch(
    settings.observeEndpoint || DEFAULT_SETTINGS.observeEndpoint,
    {
      method: "POST",
      headers,
      body: JSON.stringify(body),
    },
  );

  if (!response.ok) {
    const detail = await response.text().catch(() => "");
    throw new Error(
      `Local Browser Observe returned HTTP ${response.status}${
        detail ? `: ${detail.slice(0, 160)}` : ""
      }`,
    );
  }
  return response.json();
}

api.runtime.onInstalled.addListener(async () => {
  const current = await storageGet(DEFAULT_SETTINGS);
  await storageSet({ ...DEFAULT_SETTINGS, ...current });
});

api.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message?.type === MESSAGE_OBSERVE) {
    sendObserveEvent(message.payload, sender)
      .then((result) => sendResponse({ ok: true, result }))
      .catch((error) =>
        sendResponse({
          ok: false,
          error: error instanceof Error ? error.message : String(error),
        }),
      );
    return true;
  }

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
