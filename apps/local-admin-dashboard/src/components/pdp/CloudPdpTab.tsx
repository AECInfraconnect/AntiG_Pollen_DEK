import { useState, useEffect } from "react";
import { PdpCloudApi } from "../../services/api";
import type { CloudPdpProfile } from "../../services/api";

export function CloudPdpTab() {
  const [profile, setProfile] = useState<CloudPdpProfile | null>(null);
  const [loading, setLoading] = useState(true);

  const reload = async () => {
    setLoading(true);
    try {
      const data = await PdpCloudApi.get();
      setProfile(data);
    } catch (e) {
      console.error(e);
      setProfile(null);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    reload();
  }, []);

  const handleLogin = async () => {
    try {
      await PdpCloudApi.login();
      reload();
    } catch (e) {
      console.error(e);
    }
  };

  const handleDiscover = async () => {
    try {
      await PdpCloudApi.discover();
      reload();
    } catch (e) {
      console.error(e);
    }
  };

  const handleProbe = async () => {
    try {
      await PdpCloudApi.probe();
      reload();
    } catch (e) {
      console.error(e);
    }
  };

  if (loading) {
    return <div className="text-sm text-muted-foreground p-8 text-center">Loading Cloud PDP Profile...</div>;
  }

  if (!profile || profile.status === "disconnected") {
    return (
      <div className="py-8 text-center text-muted-foreground">
        <div className="inline-block p-4 bg-muted/50 rounded-full mb-4">
          <svg className="w-8 h-8 text-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 15a4 4 0 004 4h9a5 5 0 10-.1-9.999 5.002 5.002 0 10-9.78 2.096A4.001 4.001 0 003 15z" />
          </svg>
        </div>
        <h4 className="text-lg font-medium text-foreground mb-2">Connect to Pollen Cloud PDP</h4>
        <p className="max-w-md mx-auto text-sm">
          Connect this DEK to a fully managed Pollen Cloud PDP. Configuration and routing will be synced automatically.
        </p>
        <button
          onClick={handleLogin}
          className="mt-6 px-4 py-2 bg-primary text-primary-foreground rounded hover:opacity-90"
        >
          Login / Enroll Device
        </button>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between border-b pb-4">
        <div>
          <h3 className="font-medium">Pollen Cloud Connected</h3>
          <p className="text-sm text-muted-foreground">This DEK is enrolled with Pollen Cloud.</p>
        </div>
        <div className="flex gap-2">
          <button onClick={handleDiscover} className="px-3 py-1 bg-secondary text-secondary-foreground rounded text-xs hover:opacity-80">
            Refresh Contract
          </button>
          <button onClick={handleProbe} className="px-3 py-1 bg-secondary text-secondary-foreground rounded text-xs hover:opacity-80">
            Probe Decision
          </button>
        </div>
      </div>

      <div className="grid grid-cols-2 gap-4 text-sm">
        <div className="space-y-1">
          <span className="text-muted-foreground block text-xs">Tenant ID</span>
          <span className="font-medium font-mono">{profile.tenant_id ?? "unknown"}</span>
        </div>
        <div className="space-y-1">
          <span className="text-muted-foreground block text-xs">Device ID</span>
          <span className="font-medium font-mono">{profile.device_id ?? "unknown"}</span>
        </div>
        <div className="space-y-1">
          <span className="text-muted-foreground block text-xs">Contract Version</span>
          <span className="font-medium">{profile.contract_version ?? "unknown"}</span>
        </div>
        <div className="space-y-1">
          <span className="text-muted-foreground block text-xs">Auth Method</span>
          <span className="font-medium">{profile.auth_method ?? "unknown"}</span>
        </div>
        <div className="space-y-1 col-span-2 border-t pt-4">
          <span className="text-muted-foreground block text-xs">PDP Endpoint</span>
          <span className="font-medium font-mono bg-muted px-2 py-1 rounded inline-block mt-1">
            {profile.pdp_endpoint ?? "not discovered"}
          </span>
        </div>
      </div>
    </div>
  );
}
