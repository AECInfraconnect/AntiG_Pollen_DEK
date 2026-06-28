(function initPollekPromptGuardConstants(global) {
  const DEFAULT_ENDPOINT =
    "http://127.0.0.1:43891/v1/tenants/local/prompt-guard/check";

  global.PollekPromptGuardConstants = {
    DEFAULT_ENDPOINT,
    DEFAULT_SETTINGS: {
      enabled: true,
      endpoint: DEFAULT_ENDPOINT,
      apiToken: "",
      captureMode: "observe",
    },
    MESSAGE_CHECK: "POLLEK_PROMPT_GUARD_CHECK",
    SOURCE_BROWSER_EXTENSION: "browser_extension",
  };
})(globalThis);
