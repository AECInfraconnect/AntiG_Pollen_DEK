import { useEffect, useMemo, useState } from "react";
import { Link } from "react-router-dom";
import {
  Activity,
  CheckCircle,
  Download,
  Power,
  ShieldAlert,
  ShieldCheck,
  Trash2,
  X,
} from "lucide-react";
import { toast } from "sonner";
import { PluginApi } from "../../services/api";
import type { InstalledPlugin, PluginMarketItem } from "../../services/types";
import { cn } from "@/lib/utils";

function capabilityText(item: PluginMarketItem) {
  return item.human_capabilities?.length
    ? item.human_capabilities
    : item.capabilities.map((capability) =>
        capability
          .replace(/^http_out:/, "Sends data to ")
          .replace(/^native:/, "Uses local system capability ")
          .replace(/:/g, " ")
          .replace(/_/g, " "),
      );
}

function isSensitive(item: PluginMarketItem) {
  return item.capabilities.some(
    (capability) =>
      capability.startsWith("http_out:") ||
      capability.startsWith("native:") ||
      capability.includes(":write"),
  );
}

function signatureLabel(item: PluginMarketItem) {
  if (item.signature_state === "valid") return "Signature valid";
  if (item.signature_state === "test_only") return "Developer preview";
  if (item.signature_state === "missing") return "Missing signature";
  if (item.signature_state === "invalid") return "Signature invalid";
  return "Signature unknown";
}

function MarketCard({
  item,
  installed,
  onInstall,
}: {
  item: PluginMarketItem;
  installed?: InstalledPlugin;
  onInstall: () => void;
}) {
  const sensitive = isSensitive(item);
  return (
    <article className="flex min-h-[310px] flex-col rounded-lg border bg-card/70 p-4">
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <h3 className="break-words text-sm font-semibold">{item.name}</h3>
          <p className="mt-1 text-xs text-muted-foreground">
            v{item.version} • {item.publisher}
          </p>
        </div>
        <div
          title={item.verified ? "Verified publisher" : "Unverified publisher"}
          className={cn(
            "rounded-lg p-2",
            item.verified
              ? "bg-emerald-500/10 text-emerald-700"
              : "bg-amber-500/10 text-amber-700",
          )}
        >
          {item.verified ? (
            <CheckCircle className="h-4 w-4" />
          ) : (
            <ShieldAlert className="h-4 w-4" />
          )}
        </div>
      </div>

      <p className="mt-3 text-sm leading-6 text-muted-foreground">
        {item.description_en}
      </p>

      <div className="mt-3 flex flex-wrap gap-1.5">
        <span className="rounded-full border bg-background px-2 py-0.5 text-[11px]">
          {item.kind}
        </span>
        <span
          className={cn(
            "rounded-full border px-2 py-0.5 text-[11px]",
            item.signature_ok
              ? "border-emerald-500/25 bg-emerald-500/10 text-emerald-700"
              : "border-amber-500/25 bg-amber-500/10 text-amber-700",
          )}
        >
          {signatureLabel(item)}
        </span>
        {sensitive && (
          <span className="rounded-full border border-amber-500/25 bg-amber-500/10 px-2 py-0.5 text-[11px] text-amber-800">
            Needs consent
          </span>
        )}
      </div>

      <div className="mt-3 space-y-1.5">
        {capabilityText(item)
          .slice(0, 3)
          .map((capability) => (
            <div
              key={capability}
              className="rounded-md border bg-background/60 px-3 py-2 text-xs text-muted-foreground"
            >
              {capability}
            </div>
          ))}
      </div>

      <p className="mt-3 text-xs leading-5 text-muted-foreground">
        {item.privacy_note}
      </p>

      <div className="mt-auto pt-4">
        {installed ? (
          <div className="flex items-center justify-between gap-2 rounded-md border bg-background px-3 py-2 text-sm">
            <span className="font-medium">
              {installed.enabled ? "Installed and enabled" : "Installed"}
            </span>
            <CheckCircle className="h-4 w-4 text-emerald-600" />
          </div>
        ) : (
          <button
            type="button"
            onClick={onInstall}
            className="inline-flex h-9 w-full items-center justify-center gap-2 rounded-md bg-primary px-3 text-sm font-medium text-primary-foreground hover:bg-primary/90"
          >
            <Download className="h-4 w-4" />
            Review and install
          </button>
        )}
      </div>
    </article>
  );
}

function InstalledRow({
  plugin,
  onToggle,
  onUninstall,
}: {
  plugin: InstalledPlugin;
  onToggle: () => void;
  onUninstall: () => void;
}) {
  return (
    <article className="rounded-lg border bg-card/70 p-4">
      <div className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
        <div>
          <h3 className="text-sm font-semibold">{plugin.name ?? plugin.id}</h3>
          <p className="mt-1 text-xs text-muted-foreground">
            {plugin.kind ?? "plugin"} • {plugin.version ?? "unknown version"} •{" "}
            {plugin.health}
          </p>
          <div className="mt-3 flex flex-wrap gap-1.5">
            {(plugin.human_grants?.length
              ? plugin.human_grants
              : plugin.granted_caps
            ).map((grant) => (
              <span
                key={grant}
                className="rounded-full border bg-background px-2 py-0.5 text-[11px]"
              >
                {grant}
              </span>
            ))}
          </div>
        </div>
        <div className="flex flex-wrap gap-2">
          <button
            type="button"
            onClick={onToggle}
            className="inline-flex h-9 items-center gap-2 rounded-md border px-3 text-sm hover:bg-muted"
          >
            <Power className="h-4 w-4" />
            {plugin.enabled ? "Disable" : "Enable"}
          </button>
          <button
            type="button"
            onClick={onUninstall}
            className="inline-flex h-9 items-center gap-2 rounded-md border border-red-500/30 bg-red-500/10 px-3 text-sm font-medium text-red-700 hover:bg-red-500/15"
          >
            <Trash2 className="h-4 w-4" />
            Uninstall
          </button>
          <Link
            to={`/activity?category=plugins&q=${encodeURIComponent(plugin.id)}`}
            className="inline-flex h-9 items-center gap-2 rounded-md border px-3 text-sm hover:bg-muted"
          >
            <Activity className="h-4 w-4" />
            Activity
          </Link>
        </div>
      </div>
    </article>
  );
}

function ConsentDialog({
  item,
  onClose,
  onConfirm,
}: {
  item: PluginMarketItem;
  onClose: () => void;
  onConfirm: () => void;
}) {
  const sensitive = isSensitive(item);
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
      <button
        type="button"
        aria-label="Close plugin consent"
        className="absolute inset-0 bg-background/80 backdrop-blur-sm"
        onClick={onClose}
      />
      <section className="relative z-10 w-full max-w-2xl rounded-lg border bg-background p-5 shadow-xl">
        <div className="flex items-start justify-between gap-3">
          <div>
            <h2 className="text-lg font-semibold">Review plugin access</h2>
            <p className="mt-1 text-sm text-muted-foreground">
              Install {item.name} only if these capabilities match what you
              expect this plugin to do.
            </p>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="rounded-md p-2 text-muted-foreground hover:bg-muted hover:text-foreground"
          >
            <X className="h-4 w-4" />
          </button>
        </div>

        <div className="mt-4 grid gap-3 md:grid-cols-2">
          <div className="rounded-lg border bg-card/60 p-4">
            <div className="flex items-center gap-2 text-sm font-semibold">
              <ShieldCheck className="h-4 w-4 text-primary" />
              Trust
            </div>
            <dl className="mt-3 space-y-2 text-sm">
              <div className="flex justify-between gap-3">
                <dt className="text-muted-foreground">Publisher</dt>
                <dd className="font-medium">{item.publisher}</dd>
              </div>
              <div className="flex justify-between gap-3">
                <dt className="text-muted-foreground">Verified</dt>
                <dd className="font-medium">{item.verified ? "Yes" : "No"}</dd>
              </div>
              <div className="flex justify-between gap-3">
                <dt className="text-muted-foreground">Signature</dt>
                <dd className="font-medium">{signatureLabel(item)}</dd>
              </div>
            </dl>
          </div>

          <div
            className={cn(
              "rounded-lg border p-4",
              sensitive ? "border-amber-500/25 bg-amber-500/10" : "bg-card/60",
            )}
          >
            <div className="text-sm font-semibold">
              {sensitive ? "Sensitive access requested" : "Local access only"}
            </div>
            <p className="mt-2 text-sm leading-6 text-muted-foreground">
              {item.privacy_note ??
                "This plugin did not provide a separate privacy note."}
            </p>
          </div>
        </div>

        <div className="mt-4 rounded-lg border bg-card/60 p-4">
          <h3 className="text-sm font-semibold">Capabilities requested</h3>
          <div className="mt-3 space-y-2">
            {capabilityText(item).map((capability) => (
              <div
                key={capability}
                className="rounded-md border bg-background px-3 py-2 text-sm"
              >
                {capability}
              </div>
            ))}
          </div>
        </div>

        <div className="mt-5 flex flex-col-reverse gap-2 sm:flex-row sm:justify-end">
          <button
            type="button"
            onClick={onClose}
            className="inline-flex h-9 items-center justify-center rounded-md border px-3 text-sm hover:bg-muted"
          >
            Cancel
          </button>
          <button
            type="button"
            onClick={onConfirm}
            className="inline-flex h-9 items-center justify-center gap-2 rounded-md bg-primary px-3 text-sm font-medium text-primary-foreground hover:bg-primary/90"
          >
            <CheckCircle className="h-4 w-4" />
            Consent and install
          </button>
        </div>
      </section>
    </div>
  );
}

export function PluginMarketplace() {
  const [items, setItems] = useState<PluginMarketItem[]>([]);
  const [installed, setInstalled] = useState<InstalledPlugin[]>([]);
  const [loading, setLoading] = useState(true);
  const [consentItem, setConsentItem] = useState<PluginMarketItem | null>(null);

  const installedById = useMemo(
    () => new Map(installed.map((plugin) => [plugin.id, plugin])),
    [installed],
  );

  const load = async () => {
    setLoading(true);
    try {
      const [marketItems, installedItems] = await Promise.all([
        PluginApi.marketplaceItems(),
        PluginApi.installed(),
      ]);
      setItems(marketItems);
      setInstalled(installedItems);
    } catch (error) {
      console.error(error);
      toast.error("Failed to load plugin marketplace");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void load();
  }, []);

  const confirmInstall = async () => {
    if (!consentItem) return;
    try {
      const plugin = await PluginApi.install({
        id: consentItem.id,
        granted_caps: consentItem.capabilities,
      });
      setInstalled((current) => [
        ...current.filter((item) => item.id !== plugin.id),
        {
          ...plugin,
          name: plugin.name ?? consentItem.name,
          version: plugin.version ?? consentItem.version,
          kind: plugin.kind ?? consentItem.kind,
          source: plugin.source ?? consentItem.source,
          granted_caps: plugin.granted_caps ?? consentItem.capabilities,
        },
      ]);
      setConsentItem(null);
      toast.success(`${consentItem.name} installed`);
    } catch (error) {
      const message =
        error instanceof Error ? error.message : "Failed to install plugin";
      toast.error(message);
    }
  };

  const toggleInstalled = async (plugin: InstalledPlugin) => {
    try {
      const updated = await PluginApi.toggle(plugin.id, !plugin.enabled);
      setInstalled((current) =>
        current.map((item) =>
          item.id === plugin.id
            ? { ...item, ...updated, enabled: !plugin.enabled }
            : item,
        ),
      );
    } catch (error) {
      const message =
        error instanceof Error ? error.message : "Failed to update plugin";
      toast.error(message);
    }
  };

  const uninstall = async (plugin: InstalledPlugin) => {
    try {
      await PluginApi.uninstall(plugin.id);
      setInstalled((current) =>
        current.filter((item) => item.id !== plugin.id),
      );
      toast.success(`${plugin.name ?? plugin.id} uninstalled`);
    } catch (error) {
      const message =
        error instanceof Error ? error.message : "Failed to uninstall plugin";
      toast.error(message);
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
        <div>
          <h2 className="text-2xl font-bold tracking-tight">
            Plugin Marketplace
          </h2>
          <p className="text-sm text-muted-foreground">
            Install trusted extensions for discovery, observe, telemetry,
            definitions, and future control methods. Every sensitive capability
            requires consent before activation.
          </p>
        </div>
      </div>

      <section className="rounded-lg border bg-card/60 p-4">
        <h3 className="text-sm font-semibold">Installed Plugins</h3>
        <div className="mt-3 space-y-3">
          {installed.length ? (
            installed.map((plugin) => (
              <InstalledRow
                key={plugin.id}
                plugin={plugin}
                onToggle={() => void toggleInstalled(plugin)}
                onUninstall={() => void uninstall(plugin)}
              />
            ))
          ) : (
            <p className="rounded-md border border-dashed p-4 text-sm text-muted-foreground">
              No plugins are installed yet.
            </p>
          )}
        </div>
      </section>

      <section>
        <div className="mb-3 flex items-center justify-between">
          <h3 className="text-sm font-semibold">Available Plugins</h3>
          {loading && (
            <span className="text-xs text-muted-foreground">Loading...</span>
          )}
        </div>
        <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
          {items.map((item) => (
            <MarketCard
              key={item.id}
              item={item}
              installed={installedById.get(item.id)}
              onInstall={() => setConsentItem(item)}
            />
          ))}
          {!loading && !items.length && (
            <p className="rounded-md border border-dashed p-4 text-sm text-muted-foreground md:col-span-2 xl:col-span-3">
              No marketplace plugins are available from this local catalog yet.
            </p>
          )}
        </div>
      </section>

      {consentItem && (
        <ConsentDialog
          item={consentItem}
          onClose={() => setConsentItem(null)}
          onConfirm={() => void confirmInstall()}
        />
      )}
    </div>
  );
}
