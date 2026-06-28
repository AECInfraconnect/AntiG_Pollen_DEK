(function initPollekPromptGuardContent(global) {
  const provider = global.PollekPromptGuardProviders.providerForHost(
    location.hostname,
  );
  if (!provider) return;

  const { MESSAGE_CHECK } = global.PollekPromptGuardConstants;
  const { sendMessage } = global.PollekWebExt;
  const { visibleText, nearestEditor, likelySendButton } =
    global.PollekEditorDetector;
  let lastCaptureKey = "";
  let lastCaptureAt = 0;

  function sessionId() {
    const existing = sessionStorage.getItem("pollek.promptGuard.sessionId");
    if (existing) return existing;
    const next =
      crypto.randomUUID?.() ??
      `browser-session-${Date.now()}-${Math.random().toString(36).slice(2)}`;
    sessionStorage.setItem("pollek.promptGuard.sessionId", next);
    return next;
  }

  function captureKey(text) {
    return `${location.host}:${text.slice(0, 256)}:${text.length}`;
  }

  function shouldSkipDuplicate(text) {
    const key = captureKey(text);
    const now = Date.now();
    if (key === lastCaptureKey && now - lastCaptureAt < 1500) {
      return true;
    }
    lastCaptureKey = key;
    lastCaptureAt = now;
    return false;
  }

  function notify(message, tone = "info") {
    const existing = document.getElementById("pollek-prompt-guard-toast");
    existing?.remove();
    const toast = document.createElement("div");
    toast.id = "pollek-prompt-guard-toast";
    toast.textContent = message;
    toast.style.cssText = [
      "position:fixed",
      "z-index:2147483647",
      "right:16px",
      "bottom:16px",
      "max-width:360px",
      "padding:12px 14px",
      "border-radius:8px",
      "font:13px/1.45 system-ui,-apple-system,BlinkMacSystemFont,Segoe UI,sans-serif",
      "box-shadow:0 12px 32px rgba(0,0,0,.28)",
      "background:#111827",
      `border:1px solid ${
        tone === "warn" ? "#f59e0b" : tone === "error" ? "#ef4444" : "#8b5cf6"
      }`,
      "color:white",
    ].join(";");
    document.documentElement.appendChild(toast);
    window.setTimeout(() => toast.remove(), 5200);
  }

  async function sendPromptToGuard(text) {
    if (!text.trim() || shouldSkipDuplicate(text)) return;
    const response = await sendMessage({
      type: MESSAGE_CHECK,
      payload: {
        text,
        url: location.href,
        surface: location.hostname,
        agentId: provider.id,
        sessionId: sessionId(),
      },
    }).catch((error) => ({
      ok: false,
      error: error instanceof Error ? error.message : String(error),
    }));

    if (!response?.ok) {
      notify(
        `Pollek Prompt Guard could not reach the Local Control Plane: ${
          response?.error ?? "unknown error"
        }`,
        "warn",
      );
      return response;
    }

    const action = response.result?.guard_event?.action ?? response.result?.action;
    if (action === "deny") {
      notify(`Pollek recommends blocking this ${provider.label} prompt.`, "error");
    } else if (action === "redact") {
      notify(`Pollek found a Prompt Guard signal in ${provider.label}.`, "warn");
    }
    return response;
  }

  document.addEventListener(
    "click",
    (event) => {
      if (!likelySendButton(event.target, provider)) return;
      const editor = nearestEditor(event.target, provider);
      const text = visibleText(editor).trim();
      void sendPromptToGuard(text);
    },
    true,
  );

  document.addEventListener(
    "keydown",
    (event) => {
      if (
        event.key !== "Enter" ||
        event.shiftKey ||
        event.altKey ||
        event.ctrlKey
      ) {
        return;
      }
      const editor = nearestEditor(event.target, provider);
      if (!editor) return;
      const text = visibleText(editor).trim();
      void sendPromptToGuard(text);
    },
    true,
  );
})(globalThis);
