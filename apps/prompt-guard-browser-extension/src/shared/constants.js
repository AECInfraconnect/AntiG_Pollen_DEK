(function initPollekPromptGuardConstants(global) {
  const DEFAULT_ENDPOINT =
    "http://127.0.0.1:43891/v1/tenants/local/prompt-guard/check";
  const DEFAULT_OBSERVE_ENDPOINT =
    "http://127.0.0.1:43891/v1/tenants/local/browser-extension/events";

  global.PollekPromptGuardConstants = {
    DEFAULT_ENDPOINT,
    DEFAULT_SETTINGS: {
      enabled: true,
      observeEnabled: true,
      endpoint: DEFAULT_ENDPOINT,
      observeEndpoint: DEFAULT_OBSERVE_ENDPOINT,
      apiToken: "",
      captureMode: "observe",
      responseMetadataEnabled: true,
      attachmentMetadataEnabled: true,
    },
    MESSAGE_CHECK: "POLLEK_PROMPT_GUARD_CHECK",
    MESSAGE_OBSERVE: "POLLEK_BROWSER_OBSERVE_EVENT",
    SOURCE_BROWSER_EXTENSION: "browser_extension",
  };
})(globalThis);
