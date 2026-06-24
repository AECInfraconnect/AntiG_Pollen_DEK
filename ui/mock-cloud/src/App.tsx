import { useEffect, useState } from "react";

function App() {
  const [data, setData] = useState<any>(null);

  useEffect(() => {
    // In a real scenario we'd fetch from /api/admin/dashboard
    fetch("/api/admin/dashboard/data")
      .then((r) => r.json())
      .then((d) => setData(d))
      .catch((e) => console.error(e));
  }, []);

  return (
    <div className="min-h-screen bg-background text-textMain p-8 flex flex-col items-center animate-fade-in">
      <div className="w-full max-w-6xl">
        <header className="flex justify-between items-center mb-8 glass-panel p-6">
          <h1 className="text-3xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-primary to-secondary">
            Mock Cloud Admin
          </h1>
          <div className="flex space-x-4">
            <button className="btn-primary">Settings</button>
            <button className="btn-danger">Logout</button>
          </div>
        </header>

        <main className="grid grid-cols-1 lg:grid-cols-3 gap-6 animate-slide-up">
          <section className="lg:col-span-2 glass-panel p-6">
            <h2 className="text-xl font-semibold mb-4">Enrolled Devices</h2>
            <div className="overflow-x-auto">
              <table className="w-full text-left">
                <thead>
                  <tr className="border-b border-white/10 text-textMuted">
                    <th className="pb-3 font-medium">Device ID</th>
                    <th className="pb-3 font-medium">Tenant</th>
                    <th className="pb-3 font-medium">Status</th>
                  </tr>
                </thead>
                <tbody>
                  {data?.devices?.map((d: any) => (
                    <tr
                      key={d.id}
                      className="border-b border-white/5 last:border-0 hover:bg-white/5 transition-colors"
                    >
                      <td className="py-3">{d.id}</td>
                      <td className="py-3 text-textMuted">{d.tenant_id}</td>
                      <td className="py-3">
                        <span
                          className={`px-2 py-1 rounded text-xs font-medium ${d.revoked ? "bg-accent/20 text-accent" : "bg-green-500/20 text-green-400"}`}
                        >
                          {d.revoked ? "Revoked" : "Active"}
                        </span>
                      </td>
                    </tr>
                  ))}
                  {!data?.devices?.length && (
                    <tr>
                      <td
                        colSpan={3}
                        className="py-4 text-center text-textMuted"
                      >
                        No devices enrolled
                      </td>
                    </tr>
                  )}
                </tbody>
              </table>
            </div>
          </section>

          <section className="glass-panel p-6 flex flex-col space-y-6">
            <div>
              <h2 className="text-xl font-semibold mb-2">Policy Status</h2>
              <div className="p-4 bg-surface rounded-xl border border-white/5">
                <p className="text-sm text-textMuted">Current Active Bundle</p>
                <p className="text-lg font-bold">
                  {data?.current_version || "Unknown"}
                </p>
              </div>
            </div>
            <div>
              <h2 className="text-xl font-semibold mb-2">Telemetry</h2>
              <div className="p-4 bg-surface rounded-xl border border-white/5">
                <p className="text-sm text-textMuted">Events Captured</p>
                <p className="text-2xl font-bold text-primary">
                  {data?.telemetry_count || 0}
                </p>
              </div>
            </div>
          </section>
        </main>
      </div>
    </div>
  );
}

export default App;
