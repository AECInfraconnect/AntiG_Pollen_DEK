export function Settings() {
  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold tracking-tight">Settings</h2>
          <p className="text-muted-foreground">
            Configure local control plane settings and synchronization profiles.
          </p>
        </div>
      </div>

      <div className="glass p-6 rounded-xl space-y-6">
        <h3 className="text-lg font-medium">Control Plane Profile</h3>
        
        <div className="space-y-4 max-w-md">
          <div className="grid gap-2">
            <label className="text-sm font-medium">API Endpoint</label>
            <input 
              type="text" 
              className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
              defaultValue="http://localhost:43890"
              disabled
            />
          </div>
          <div className="grid gap-2">
            <label className="text-sm font-medium">Mock Role</label>
            <input 
              type="text" 
              className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
              defaultValue="admin"
              disabled
            />
          </div>
        </div>
      </div>
    </div>
  );
}
