(function initPollekPromptGuardProviders(global) {
  const DEFAULT_SEND_BUTTON_SELECTORS = [
    'button[aria-label*="send" i]',
    'button[aria-label*="submit" i]',
    'button[title*="send" i]',
    'button[data-testid*="send" i]',
    'button[type="submit"]',
  ];

  const DEFAULT_EDITOR_SELECTORS = [
    "textarea",
    'input[type="text"]',
    '[contenteditable="true"]',
    '[role="textbox"]',
    ".ProseMirror",
  ];

  const PROVIDERS = [
    {
      id: "chatgpt-browser",
      label: "ChatGPT",
      hosts: ["chatgpt.com", "chat.openai.com"],
      editorSelectors: DEFAULT_EDITOR_SELECTORS,
      sendButtonSelectors: DEFAULT_SEND_BUTTON_SELECTORS,
    },
    {
      id: "claude-browser",
      label: "Claude",
      hosts: ["claude.ai"],
      editorSelectors: DEFAULT_EDITOR_SELECTORS,
      sendButtonSelectors: DEFAULT_SEND_BUTTON_SELECTORS,
    },
    {
      id: "deepseek-browser",
      label: "DeepSeek",
      hosts: ["chat.deepseek.com", "deepseek.com"],
      editorSelectors: DEFAULT_EDITOR_SELECTORS,
      sendButtonSelectors: DEFAULT_SEND_BUTTON_SELECTORS,
    },
    {
      id: "gemini-browser",
      label: "Gemini",
      hosts: ["gemini.google.com", "aistudio.google.com"],
      editorSelectors: DEFAULT_EDITOR_SELECTORS,
      sendButtonSelectors: DEFAULT_SEND_BUTTON_SELECTORS,
    },
    {
      id: "manus-browser",
      label: "Manus",
      hosts: ["manus.im", "*.manus.im"],
      editorSelectors: DEFAULT_EDITOR_SELECTORS,
      sendButtonSelectors: DEFAULT_SEND_BUTTON_SELECTORS,
    },
  ];

  function hostMatches(pattern, hostname) {
    if (pattern.startsWith("*.")) {
      const suffix = pattern.slice(1);
      return hostname.endsWith(suffix);
    }
    return hostname === pattern || hostname.endsWith(`.${pattern}`);
  }

  function providerForHost(hostname) {
    const normalized = hostname.toLowerCase();
    return (
      PROVIDERS.find((profile) =>
        profile.hosts.some((host) => hostMatches(host, normalized)),
      ) || null
    );
  }

  function extensionMatches() {
    return Array.from(
      new Set(
        PROVIDERS.flatMap((profile) =>
          profile.hosts.map((host) =>
            host.startsWith("*.")
              ? `https://${host}/*`
              : `https://${host}/*`,
          ),
        ),
      ),
    );
  }

  function hostPermissions() {
    return [
      "http://127.0.0.1:43891/*",
      "http://localhost:43891/*",
      ...extensionMatches(),
    ];
  }

  global.PollekPromptGuardProviders = {
    PROVIDERS,
    DEFAULT_EDITOR_SELECTORS,
    DEFAULT_SEND_BUTTON_SELECTORS,
    providerForHost,
    extensionMatches,
    hostPermissions,
  };
})(globalThis);
