const { DEFAULT_SETTINGS } = globalThis.PollekPromptGuardConstants;
const { storageGet, storageSet } = globalThis.PollekWebExt;

const fields = {
  enabled: document.getElementById("enabled"),
  endpoint: document.getElementById("endpoint"),
  apiToken: document.getElementById("apiToken"),
  captureMode: document.getElementById("captureMode"),
  save: document.getElementById("save"),
  status: document.getElementById("status"),
};

async function load() {
  const settings = await storageGet(DEFAULT_SETTINGS);
  fields.enabled.checked = Boolean(settings.enabled);
  fields.endpoint.value = settings.endpoint || DEFAULT_SETTINGS.endpoint;
  fields.apiToken.value = settings.apiToken || "";
  fields.captureMode.value = settings.captureMode || "observe";
}

async function save() {
  await storageSet({
    enabled: fields.enabled.checked,
    endpoint: fields.endpoint.value.trim() || DEFAULT_SETTINGS.endpoint,
    apiToken: fields.apiToken.value.trim(),
    captureMode: fields.captureMode.value,
  });
  fields.status.textContent = "Saved.";
  window.setTimeout(() => {
    fields.status.textContent = "";
  }, 2200);
}

fields.save.addEventListener("click", () => void save());
void load();
