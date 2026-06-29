import "@typespec/http";
import "@typespec/rest";
using TypeSpec.Http;
using TypeSpec.Rest;

namespace Pollek.Contract;

union BrowserObserveEventType {
  "tab_loaded",
  "tab_visible",
  "prompt_submitted",
  "attachment_detected",
  "visible_response_metadata",
  "tool_surface_detected",
}

model BrowserExtensionObserveEvent {
  schema_version?: "pollek.browser_observe_event.v1";
  event_type: BrowserObserveEventType | string;
  extension_id?: string;
  extension_version?: string;
  browser_id?: string;
  browser_name?: string;
  provider_id?: string;
  provider_label?: string;
  tab_id?: int64;
  window_id?: int64;
  url?: string;
  title?: string;
  session_id?: string;
  occurred_at?: string;
  text_length?: int64;
  text_hash?: string;
  response_length?: int64;
  attachment_count?: int64;
  attachment_extensions?: string[];
  page_visibility?: string;
  capture_mode?: string;
  metadata?: Record<unknown>;
}

model BrowserExtensionStatusItem {
  schema_version?: "pollek.browser_extension.status_item.v1";
  extension_id?: string;
  extension_version?: string;
  browser_id?: string;
  browser_name?: string;
  last_provider_id?: string;
  last_provider_label?: string;
  last_event_type?: string;
  last_seen?: string;
  capture_mode?: string;
  raw_prompt_or_response_stored: boolean;
  capabilities: string[];
}

model BrowserExtensionStatusResponse {
  schema_version: "pollek.browser_extension.status.v1";
  tenant_id: string;
  items: BrowserExtensionStatusItem[];
  limitations: string[];
}

model BrowserExtensionIngestResponse {
  schema_version: "pollek.browser_extension.ingest.v1";
  status: string;
  event_id: string;
  telemetry_recorded: boolean;
  raw_prompt_or_response_stored: boolean;
  capture_quality: string;
}

@tag("Browser Extension")
@route("/v1/tenants/{tenant_id}/browser-extension")
interface BrowserExtensionApi {
  @doc("List installed browser observe connectors that have emitted metadata-only events.")
  @route("status")
  @get getBrowserExtensionStatus(@path tenant_id: string, ...PollekHeaders): BrowserExtensionStatusResponse | PollekError;

  @doc("Ingest a metadata-only browser observe event from the Prompt Guard browser extension.")
  @route("events")
  @post ingestBrowserExtensionEvent(@path tenant_id: string, ...PollekHeaders, @body body: BrowserExtensionObserveEvent): BrowserExtensionIngestResponse | PollekError;
}
