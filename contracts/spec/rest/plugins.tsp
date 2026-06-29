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

union PluginLifecycleState {
  "available",
  "installed",
  "enabled",
  "disabled",
  "canary",
  "revoked",
  "update_available",
  "rollback_available",
  "failed",
}

union PluginTrustLevel {
  "verified",
  "reviewed_native",
  "local_only",
  "sends_data_out",
  "developer_preview",
  "private_registry",
  "unverified",
}

model PluginMarketItem {
  id: string;
  name: string;
  version: string;
  latest_version?: string;
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
  registry_ref?: string;
  release_notes?: string;
  update_available?: boolean;
  rollback_supported?: boolean;
  trust_labels?: PluginTrustLevel[];
  lifecycle_state?: PluginLifecycleState;
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
  latest_version?: string;
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
  previous_versions?: string[];
  rollback_version?: string;
  update_available?: boolean;
  rollback_available?: boolean;
  revoked?: boolean;
  rollout?: string;
  canary_percent?: int32;
  trust_labels?: PluginTrustLevel[];
  health_metrics?: Record<unknown>;
  lifecycle_state?: PluginLifecycleState;
}

model InstalledPluginListResponse {
  schema_version: "pollek.installed_plugins.v1";
  items: InstalledPlugin[];
}

model PluginInstallRequest {
  id: string;
  granted_caps: string[];
  accept_risk?: boolean;
  source?: PluginSource;
}

model PluginToggleRequest {
  enabled: boolean;
}

model PluginLifecycleRequest {
  target_version?: string;
  canary_percent?: int32;
  reason?: string;
  accept_risk?: boolean;
}

model PluginLifecycleResponse {
  schema_version: "pollek.plugin_lifecycle.v1";
  status: string;
  action: string;
  plugin: InstalledPlugin;
  audit_event_id?: string;
  message: string;
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

  @doc("Check plugin health and record a local audit event.")
  @route("{plugin_id}/health")
  @post checkPluginHealth(@path tenant_id: string, @path plugin_id: string, ...PollekHeaders): PluginLifecycleResponse | PollekError;

  @doc("Update an installed plugin to a locally available or private-registry version.")
  @route("{plugin_id}/update")
  @post updatePlugin(@path tenant_id: string, @path plugin_id: string, ...PollekHeaders, @body body: PluginLifecycleRequest): PluginLifecycleResponse | PollekError;

  @doc("Rollback an installed plugin to the previous local version.")
  @route("{plugin_id}/rollback")
  @post rollbackPlugin(@path tenant_id: string, @path plugin_id: string, ...PollekHeaders, @body body: PluginLifecycleRequest): PluginLifecycleResponse | PollekError;

  @doc("Move an installed plugin into a staged canary rollout.")
  @route("{plugin_id}/canary")
  @post canaryPlugin(@path tenant_id: string, @path plugin_id: string, ...PollekHeaders, @body body: PluginLifecycleRequest): PluginLifecycleResponse | PollekError;

  @doc("Revoke a plugin locally, disable it, and mark granted capabilities as unavailable.")
  @route("{plugin_id}/revoke")
  @post revokePlugin(@path tenant_id: string, @path plugin_id: string, ...PollekHeaders, @body body: PluginLifecycleRequest): PluginLifecycleResponse | PollekError;

  @doc("Uninstall a plugin and revoke local grants.")
  @route("{plugin_id}")
  @delete uninstallPlugin(@path tenant_id: string, @path plugin_id: string, ...PollekHeaders): PluginUninstallResponse | PollekError;
}
