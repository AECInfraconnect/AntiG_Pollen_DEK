(function initPollekPromptGuardContent(global) {
  const provider = global.PollekPromptGuardProviders.providerForHost(
    location.hostname,
  );
  if (!provider) return;

  const { MESSAGE_CHECK, MESSAGE_OBSERVE } = global.PollekPromptGuardConstants;
  const { sendMessage } = global.PollekWebExt;
  const { visibleText, nearestEditor, likelySendButton } =
    global.PollekEditorDetector;
  let lastCaptureKey = "";
  let lastCaptureAt = 0;
  let lastResponseKey = "";
  let responseTimer = 0;

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

  async function sha256Hex(text) {
    const bytes = new TextEncoder().encode(text);
    const digest = await crypto.subtle.digest("SHA-256", bytes);
    return `sha256:${Array.from(new Uint8Array(digest))
      .map((byte) => byte.toString(16).padStart(2, "0"))
      .join("")}`;
  }

  function baseObservePayload(eventType) {
    return {
      event_type: eventType,
      provider_id: provider.id,
      provider_label: provider.label,
      url: location.href,
      title: document.title,
      session_id: sessionId(),
      occurred_at: new Date().toISOString(),
      page_visibility: document.visibilityState,
      metadata: {
        host: location.hostname,
        path_depth: location.pathname.split("/").filter(Boolean).length,
      },
    };
  }

  async function sendObserveEvent(payload) {
    return sendMessage({
      type: MESSAGE_OBSERVE,
      payload,
    }).catch((error) => ({
      ok: false,
      error: error instanceof Error ? error.message : String(error),
    }));
  }

  async function observePromptSubmitted(text) {
    const trimmed = text.trim();
    if (!trimmed) return;
    await sendObserveEvent({
      ...baseObservePayload("prompt_submitted"),
      text_length: trimmed.length,
      text_hash: await sha256Hex(trimmed),
      metadata: {
        ...baseObservePayload("prompt_submitted").metadata,
        editor_kind: "web_ai_textbox",
        raw_prompt_or_response_stored: false,
      },
    });
  }

  function attachmentMetadata(files) {
    const list = Array.from(files ?? []);
    const extensions = Array.from(
      new Set(
        list.map((file) => {
          const name = file.name || "";
          const ext = name.includes(".") ? name.split(".").pop() : "unknown";
          return String(ext || "unknown").toLowerCase().slice(0, 12);
        }),
      ),
    );
    return {
      attachment_count: list.length,
      attachment_extensions: extensions,
      metadata: {
        total_bytes: list.reduce((sum, file) => sum + (file.size || 0), 0),
        raw_file_names_stored: false,
      },
    };
  }

  function observeAttachmentInput(target) {
    if (!(target instanceof HTMLInputElement) || target.type !== "file") return;
    const meta = attachmentMetadata(target.files);
    if (!meta.attachment_count) return;
    void sendObserveEvent({
      ...baseObservePayload("attachment_detected"),
      ...meta,
      metadata: {
        ...baseObservePayload("attachment_detected").metadata,
        ...meta.metadata,
      },
    });
  }

  function responseContainer() {
    const selectors =
      provider.responseSelectors ??
      global.PollekPromptGuardProviders.DEFAULT_RESPONSE_SELECTORS;
    for (const selector of selectors) {
      const nodes = Array.from(document.querySelectorAll(selector));
      const node = nodes.reverse().find((candidate) => visibleText(candidate).trim());
      if (node) return node;
    }
    return null;
  }

  function scheduleResponseMetadata() {
    window.clearTimeout(responseTimer);
    responseTimer = window.setTimeout(async () => {
      const container = responseContainer();
      const text = visibleText(container).trim();
      if (text.length < 40) return;
      const key = `${location.host}:${text.length}:${text.slice(-120)}`;
      if (key === lastResponseKey) return;
      lastResponseKey = key;
      await sendObserveEvent({
        ...baseObservePayload("visible_response_metadata"),
        response_length: text.length,
        text_hash: await sha256Hex(text),
        metadata: {
          ...baseObservePayload("visible_response_metadata").metadata,
          raw_prompt_or_response_stored: false,
          selector_source: "visible_dom_metadata",
        },
      });
    }, 1200);
  }

  async function sendPromptToGuard(text) {
    if (!text.trim() || shouldSkipDuplicate(text)) return;
    void observePromptSubmitted(text);
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

  document.addEventListener(
    "change",
    (event) => {
      observeAttachmentInput(event.target);
    },
    true,
  );

  document.addEventListener(
    "drop",
    (event) => {
      const meta = attachmentMetadata(event.dataTransfer?.files);
      if (!meta.attachment_count) return;
      void sendObserveEvent({
        ...baseObservePayload("attachment_detected"),
        ...meta,
        metadata: {
          ...baseObservePayload("attachment_detected").metadata,
          ...meta.metadata,
          source: "drop",
        },
      });
    },
    true,
  );

  document.addEventListener("visibilitychange", () => {
    void sendObserveEvent(baseObservePayload("tab_visible"));
  });

  const observer = new MutationObserver(scheduleResponseMetadata);
  observer.observe(document.documentElement, {
    childList: true,
    subtree: true,
    characterData: true,
  });

  void sendObserveEvent(baseObservePayload("tab_loaded"));
})(globalThis);
