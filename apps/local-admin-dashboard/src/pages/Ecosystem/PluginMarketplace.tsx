import { useState } from "react";
import { Download, CheckCircle, ShieldAlert } from "lucide-react";

export function PluginMarketplace() {
  const [installing, setInstalling] = useState<string | null>(null);
  const [installed, setInstalled] = useState<Record<string, boolean>>({
    "pii-redactor": true,
  });

  const plugins = [
    {
      id: "pii-redactor",
      name: "PII Redactor Plugin",
      version: "1.0.0",
      vendor: "AEC Infraconnect",
      description: "Redacts PII such as email and phone numbers before egress.",
      verified: true,
    },
    {
      id: "malware-scanner",
      name: "Malware Scanner",
      version: "2.1.0",
      vendor: "Security Inc",
      description: "Scans files for malware signatures during filesystem access.",
      verified: true,
    },
    {
      id: "git-commit-analyzer",
      name: "Git Commit Analyzer",
      version: "0.9.0",
      vendor: "Community",
      description: "Ensures generated git commits meet conventional standards.",
      verified: false,
    },
  ];

  const handleInstall = (id: string) => {
    setInstalling(id);
    // Mock installation delay
    setTimeout(() => {
      setInstalled((prev) => ({ ...prev, [id]: true }));
      setInstalling(null);
    }, 1500);
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold tracking-tight">Plugin Marketplace</h2>
          <p className="text-muted-foreground">
            Discover and install verifiable Wasm plugins to extend the local DEK.
          </p>
        </div>
      </div>

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        {plugins.map((plugin) => (
          <div
            key={plugin.id}
            className="rounded-xl border bg-card text-card-foreground shadow flex flex-col"
          >
            <div className="p-6 flex-1 space-y-4">
              <div className="flex items-start justify-between">
                <div>
                  <h3 className="font-semibold leading-none tracking-tight">
                    {plugin.name}
                  </h3>
                  <p className="text-sm text-muted-foreground mt-1">
                    v{plugin.version} • {plugin.vendor}
                  </p>
                </div>
                {plugin.verified ? (
                  <div title="Verified Publisher"><CheckCircle className="h-5 w-5 text-green-500" /></div>
                ) : (
                  <div title="Community Plugin"><ShieldAlert className="h-5 w-5 text-yellow-500" /></div>
                )}
              </div>
              <p className="text-sm">{plugin.description}</p>
            </div>
            <div className="p-6 pt-0 mt-auto">
              {installed[plugin.id] ? (
                <button
                  disabled
                  className="w-full inline-flex items-center justify-center rounded-md text-sm font-medium border border-input bg-background hover:bg-accent hover:text-accent-foreground h-10 px-4 py-2 opacity-50"
                >
                  <CheckCircle className="mr-2 h-4 w-4" />
                  Installed
                </button>
              ) : (
                <button
                  onClick={() => handleInstall(plugin.id)}
                  disabled={installing === plugin.id}
                  className="w-full inline-flex items-center justify-center rounded-md text-sm font-medium bg-primary text-primary-foreground hover:bg-primary/90 h-10 px-4 py-2"
                >
                  {installing === plugin.id ? (
                    <Download className="mr-2 h-4 w-4 animate-spin" />
                  ) : (
                    <Download className="mr-2 h-4 w-4" />
                  )}
                  {installing === plugin.id ? "Installing..." : "Install"}
                </button>
              )}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
