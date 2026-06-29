const { DEFAULT_SETTINGS } = globalThis.PollekPromptGuardConstants;
const { storageGet, storageSet } = globalThis.PollekWebExt;

const fields = {
  enabled: document.getElementById("enabled"),
  observeEnabled: document.getElementById("observeEnabled"),
  endpoint: document.getElementById("endpoint"),
  observeEndpoint: document.getElementById("observeEndpoint"),
  apiToken: document.getElementById("apiToken"),
  captureMode: document.getElementById("captureMode"),
  responseMetadataEnabled: document.getElementById("responseMetadataEnabled"),
  attachmentMetadataEnabled: document.getElementById("attachmentMetadataEnabled"),
  save: document.getElementById("save"),
  status: document.getElementById("status"),
};

async function load() {
  const settings = await storageGet(DEFAULT_SETTINGS);
  fields.enabled.checked = Boolean(settings.enabled);
  fields.observeEnabled.checked = settings.observeEnabled !== false;
  fields.endpoint.value = settings.endpoint || DEFAULT_SETTINGS.endpoint;
  fields.observeEndpoint.value =
    settings.observeEndpoint || DEFAULT_SETTINGS.observeEndpoint;
  fields.apiToken.value = settings.apiToken || "";
  fields.captureMode.value = settings.captureMode || "observe";
  fields.responseMetadataEnabled.checked =
    settings.responseMetadataEnabled !== false;
  fields.attachmentMetadataEnabled.checked =
    settings.attachmentMetadataEnabled !== false;
}

async function save() {
  await storageSet({
    enabled: fields.enabled.checked,
    observeEnabled: fields.observeEnabled.checked,
    endpoint: fields.endpoint.value.trim() || DEFAULT_SETTINGS.endpoint,
    observeEndpoint:
      fields.observeEndpoint.value.trim() || DEFAULT_SETTINGS.observeEndpoint,
    apiToken: fields.apiToken.value.trim(),
    captureMode: fields.captureMode.value,
    responseMetadataEnabled: fields.responseMetadataEnabled.checked,
    attachmentMetadataEnabled: fields.attachmentMetadataEnabled.checked,
  });
  fields.status.textContent = "Saved.";
  window.setTimeout(() => {
    fields.status.textContent = "";
  }, 2200);
}

fields.save.addEventListener("click", () => void save());
void load();
