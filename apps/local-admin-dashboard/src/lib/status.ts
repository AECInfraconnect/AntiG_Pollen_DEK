export type UiStatus = "ok" | "degraded" | "failed" | "info" | "idle";

export function statusToken(s: UiStatus) {
  return {
    ok: {
      dot: "bg-emerald-400",
      text: "text-emerald-400",
      ring: "ring-emerald-500/30",
      bg: "bg-emerald-500/10",
    },
    degraded: {
      dot: "bg-amber-400",
      text: "text-amber-400",
      ring: "ring-amber-500/30",
      bg: "bg-amber-500/10",
    },
    failed: {
      dot: "bg-rose-400",
      text: "text-rose-400",
      ring: "ring-rose-500/30",
      bg: "bg-rose-500/10",
    },
    info: {
      dot: "bg-sky-400",
      text: "text-sky-400",
      ring: "ring-sky-500/30",
      bg: "bg-sky-500/10",
    },
    idle: {
      dot: "bg-zinc-500",
      text: "text-zinc-400",
      ring: "ring-zinc-500/20",
      bg: "bg-zinc-500/10",
    },
  }[s];
}
