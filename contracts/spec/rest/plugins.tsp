import "@typespec/http";
import "@typespec/rest";
using TypeSpec.Http;
using TypeSpec.Rest;

namespace Pollek.Contract;

union PluginKind {
  "discovery.scanner",
  "discovery.signature",
  "observe.collector",
  "enforce.method",
  "policy.evaluator",
  "policy.preset",
  "telemetry.exporter",
  "telemetry.transform",
  "resource.classifier",
  "risk.scorer",
  "definition.feed",
  "notify.channel",
  "compliance.profile",
  "unknown",
}

union PluginSource {
  "local_catalog",
  "marketplace",
  "sideload",
  "private_registry",
}

union PluginSignatureState {
  "valid",
  "invalid",
  "missing",
  "test_only",
  "unknown",
}

model PluginMarketItem {
  id: string;
  name: string;
  version: string;
  kind: PluginKind;
  publisher: string;
  verified: boolean;
  rating: float32;
  installs: int64;
  capabilities: string[];
  human_capabilities?: string[];
  os: string[];
  min_engine_version: string;
  signature_ok: boolean;
  signature_state: PluginSignatureState;
  description_en: string;
  description_th?: string;
  privacy_note?: string;
  source: PluginSource;
}

model PluginMarketplaceListResponse {
  schema_version: "pollek.marketplace.v1";
  items: PluginMarketItem[];
}

model PluginMarketplaceItemResponse {
  schema_version: "pollek.marketplace.item.v1";
  item?: PluginMarketItem;
}

model InstalledPlugin {
  schema_version?: "pollek.installed_plugin.v1";
  id: string;
  name?: string;
  version?: string;
  kind?: PluginKind;
  enabled: boolean;
  granted_caps: string[];
  human_grants?: string[];
  health: string;
  source: PluginSource;
  signature_state?: PluginSignatureState;
  privacy_note?: string;
  installed_at?: string;
  last_seen?: string;
}

model InstalledPluginListResponse {
  schema_version: "pollek.installed_plugins.v1";
  items: InstalledPlugin[];
}

model PluginInstallRequest {
  id: string;
  granted_caps: string[];
}

model PluginToggleRequest {
  enabled: boolean;
}

model PluginUninstallResponse {
  status: string;
  id: string;
  revoked_caps: boolean;
  cleared_plugin_namespace: boolean;
}

@tag("Plugins")
@route("/v1/tenants/{tenant_id}/marketplace")
interface PluginMarketplaceApi {
  @doc("List locally available plugin marketplace items.")
  @route("items")
  @get listMarketplaceItems(@path tenant_id: string, ...PollekHeaders): PluginMarketplaceListResponse | PollekError;

  @doc("Get one plugin marketplace item.")
  @route("items/{plugin_id}")
  @get getMarketplaceItem(@path tenant_id: string, @path plugin_id: string, ...PollekHeaders): PluginMarketplaceItemResponse | PollekError;
}

@tag("Plugins")
@route("/v1/tenants/{tenant_id}/plugins")
interface InstalledPluginsApi {
  @doc("List installed local plugins.")
  @get listInstalledPlugins(@path tenant_id: string, ...PollekHeaders): InstalledPluginListResponse | PollekError;

  @doc("Install a plugin after consent.")
  @route("install")
  @post installPlugin(@path tenant_id: string, ...PollekHeaders, @body body: PluginInstallRequest): InstalledPlugin | PollekError;

  @doc("Toggle an installed plugin.")
  @route("{plugin_id}/toggle")
  @post togglePlugin(@path tenant_id: string, @path plugin_id: string, ...PollekHeaders, @body body: PluginToggleRequest): InstalledPlugin | PollekError;

  @doc("Uninstall a plugin and revoke local grants.")
  @route("{plugin_id}")
  @delete uninstallPlugin(@path tenant_id: string, @path plugin_id: string, ...PollekHeaders): PluginUninstallResponse | PollekError;
}
