import { useState, useEffect } from "react";
import { switchProfile } from "../services/api";

export function Settings() {
  const [profile, setProfile] = useState<'local' | 'mock-cloud'>('local');

  useEffect(() => {
    const p = localStorage.getItem('dek_admin_profile');
    if (p === 'mock-cloud') setProfile('mock-cloud');
  }, []);

  const handleProfileChange = (newProfile: 'local' | 'mock-cloud') => {
    setProfile(newProfile);
    switchProfile(newProfile); // This will reload the page
  };

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
            <label className="text-sm font-medium">Active Profile</label>
            <select 
              value={profile}
              onChange={(e) => handleProfileChange(e.target.value as any)}
              className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
            >
              <option value="local">Local Control Plane (127.0.0.1:43890)</option>
              <option value="mock-cloud">Mock Pollen Cloud (127.0.0.1:43891)</option>
            </select>
          </div>
          <div className="grid gap-2">
            <label className="text-sm font-medium">API Endpoint</label>
            <input 
              type="text" 
              className="flex h-10 w-full rounded-md border border-input bg-muted/50 px-3 py-2 text-sm text-muted-foreground"
              value={profile === 'mock-cloud' ? 'http://localhost:43891' : 'http://localhost:43890'}
              disabled
            />
          </div>
          <div className="grid gap-2">
            <label className="text-sm font-medium">Mock Role</label>
            <input 
              type="text" 
              className="flex h-10 w-full rounded-md border border-input bg-muted/50 px-3 py-2 text-sm text-muted-foreground"
              value={profile === 'mock-cloud' ? 'admin' : ''}
              disabled
            />
          </div>
        </div>
      </div>
    </div>
  );
}
