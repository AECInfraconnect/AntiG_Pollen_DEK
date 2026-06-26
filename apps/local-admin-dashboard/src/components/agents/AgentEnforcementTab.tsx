import { useEffect, useState } from "react";
import { TelemetryApi } from "../../services/api";
import { ShieldCheck, AlertTriangle } from "lucide-react";
import { useTranslation } from "react-i18next";

function getFriendlyState(state: string, isTh: boolean) {
  const s = state?.toLowerCase() || "";
  if (s === "enforcing") return isTh ? "🛡️ กำลังบังคับใช้จริงบนเครื่องนี้" : "🛡️ Actively enforcing on this device";
  if (s === "observing") return isTh ? "👁️ กำลังสังเกตการณ์ (ยังไม่บล็อกจริง)" : "👁️ Observing (not blocking yet)";
  if (s === "degraded") return isTh ? "⚠️ บังคับใช้ได้บางส่วน" : "⚠️ Partially enforced";
  if (s === "failed") return isTh ? "❌ บังคับใช้ไม่สำเร็จ — ดูวิธีแก้" : "❌ Enforcement failed — see fix";
  return state;
}

export function AgentEnforcementTab({ agentId }: { agentId: string }) {
  const [data, setData] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const { i18n } = useTranslation();
  const th = i18n.language === "th";

  useEffect(() => {
    TelemetryApi.getEnforcementStatus(agentId)
      .then((res) => setData(res.items || []))
      .catch(console.error)
      .finally(() => setLoading(false));
  }, [agentId]);

  if (loading) return <div className="p-4 text-sm text-muted-foreground">{th ? "กำลังโหลด..." : "Loading enforcement status..."}</div>;
  if (!data.length) {
    return (
      <div className="flex flex-col items-center justify-center p-8 text-center border border-dashed rounded-lg text-muted-foreground">
        <ShieldCheck className="h-8 w-8 mb-2 opacity-50" />
        <p className="text-sm">{th ? "ไม่มีการบังคับใช้สำหรับ Agent นี้" : "No enforcement active or requested for this agent."}</p>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {data.map((item, i) => {
        const payload = item.payload || item;
        const msg = th ? payload.message_th : payload.message_en;
        const userAction = th ? payload.user_action_th : payload.user_action_en;
        const planeState = payload.plane_state || item.state;
        
        return (
          <div key={i} className="flex flex-col p-4 bg-muted/30 border rounded-lg">
            <div className="flex items-center justify-between mb-2">
              <div>
                <div className="text-sm font-semibold capitalize">{payload.domain || "System"}</div>
                <div className="text-xs text-muted-foreground mt-1">Method: {payload.control_method || "Unknown"}</div>
              </div>
              <div className={`px-3 py-1 rounded-full text-xs font-medium ${planeState?.toLowerCase() === 'enforcing' ? 'bg-green-500/10 text-green-500' : 'bg-amber-500/10 text-amber-500'}`}>
                {getFriendlyState(planeState, th)}
              </div>
            </div>
            {msg && <div className="text-sm mt-2">{msg}</div>}
            {userAction && <div className="text-sm mt-1 text-primary font-medium flex items-center gap-1"><AlertTriangle className="h-4 w-4"/> {userAction}</div>}
          </div>
        );
      })}
    </div>
  );
}
