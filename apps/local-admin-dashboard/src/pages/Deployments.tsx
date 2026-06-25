import { useState } from "react";
import { MasterDetailLayout } from "../components/layout/MasterDetailLayout";
import { EntityCard } from "../components/shared/EntityCard";
import type { EntityCardProps } from "../components/shared/EntityCard";

export function Deployments() {
  const [selectedId, setSelectedId] = useState<string | null>(null);

  const mockDeployments = [
    {
      id: "dep-1",
      target: "claude_desktop",
      status: "active",
      level: "Enforce",
      date: new Date().toISOString(),
    },
    {
      id: "dep-2",
      target: "cursor",
      status: "needs_approval",
      level: "Approval",
      date: new Date().toISOString(),
    },
  ];

  const mappedCards: EntityCardProps[] = mockDeployments.map((d) => ({
    id: d.id,
    kind: "deployment",
    title: `Deployment to ${d.target}`,
    subtitle: d.id,
    status: d.status as any,
    statusLabel: d.status.replace("_", " "),
    summary: `Requested level: ${d.level}`,
    chips: [{ label: "User Policy", tone: "info" }],
    lastUpdatedAt: d.date,
  }));

  const selected = mockDeployments.find((d) => d.id === selectedId);

  const masterContent = (
    <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
      {mappedCards.map((card) => (
        <EntityCard
          key={card.id}
          {...card}
          selected={selectedId === card.id}
          onClick={() => setSelectedId(card.id)}
        />
      ))}
    </div>
  );

  const detailContent = selected ? (
    <div className="space-y-6">
      <div>
        <h3 className="text-xl font-bold">Deployment Details</h3>
        <p className="text-sm text-muted-foreground">{selected.id}</p>
      </div>
      <div className="p-4 bg-muted/50 rounded border">
        <p className="text-sm">
          Status: <strong>{selected.status}</strong>
        </p>
        <p className="text-sm">
          Target: <strong>{selected.target}</strong>
        </p>
      </div>
      <div className="flex gap-2 justify-end">
        {selected.status === "needs_approval" && (
          <button className="px-4 py-2 bg-primary text-primary-foreground rounded-md text-sm font-medium">
            Approve
          </button>
        )}
        <button className="px-4 py-2 bg-muted text-foreground border rounded-md text-sm font-medium">
          Rollback
        </button>
      </div>
    </div>
  ) : null;

  return (
    <MasterDetailLayout
      title="Deployments"
      description="Track active policy deployments across devices and agents."
      masterContent={masterContent}
      detailContent={detailContent}
      onCloseDetail={() => setSelectedId(null)}
    />
  );
}
