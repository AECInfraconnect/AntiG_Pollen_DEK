import { useEffect, useState } from "react";
import { Activity } from "lucide-react";
import { TelemetryApi } from "../../services/api";

export function AgentActivityTab({ agentId }: { agentId: string }) {
  const [events, setEvents] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    // Initial fetch of observations
    TelemetryApi.getObservations({ agentId })
      .then((res) => {
        const sorted = (res.items || []).sort((a: any, b: any) => 
          new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
        );
        setEvents(sorted.slice(0, 50));
      })
      .catch(console.error)
      .finally(() => setLoading(false));

    // SSE for live updates
    const source = new EventSource("/v1/telemetry/observations/stream");
    source.onmessage = (e) => {
      try {
        const data = JSON.parse(e.data);
        if (data.agent_id === agentId) {
          setEvents((prev) => [data, ...prev].slice(0, 50));
        }
      } catch (err) {
        console.error("Failed to parse SSE event", err);
      }
    };

    return () => {
      source.close();
    };
  }, [agentId]);

  if (loading) return <div className="p-4 text-sm text-muted-foreground">Loading activity...</div>;
  
  if (!events.length) {
    return (
      <div className="flex flex-col items-center justify-center p-8 text-center border border-dashed rounded-lg text-muted-foreground">
        <Activity className="h-8 w-8 mb-2 opacity-50" />
        <p className="text-sm">No activity recorded yet.</p>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {events.map((ev, i) => (
        <div key={i} className="flex flex-col p-4 bg-muted/30 border rounded-lg">
          <div className="flex justify-between items-center mb-2">
            <span className="text-sm font-semibold">{ev.action || "Observation"}</span>
            <span className="text-xs text-muted-foreground">
              {new Date(ev.timestamp).toLocaleTimeString()}
            </span>
          </div>
          <div className="text-xs font-mono bg-muted/50 p-2 rounded break-all">
            {JSON.stringify(ev.details || ev, null, 2)}
          </div>
        </div>
      ))}
    </div>
  );
}
