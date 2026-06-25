import { useState } from "react";
import { MasterDetailLayout } from "../components/layout/MasterDetailLayout";
import { EntityCard } from "../components/shared/EntityCard";
import type { EntityCardProps } from "../components/shared/EntityCard";

export function ControlMethods() {
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const mappedCards: EntityCardProps[] = [
    {
      id: "cm-1",
      kind: "control_method",
      title: "AgentToolControl",
      status: "active",
      statusLabel: "Ready",
      summary: "Intercepts tool execution via STDIO wrapper.",
      chips: [{ label: "Application Layer", tone: "info" }],
    }
  ];

  return (
    <MasterDetailLayout
      title="Control Methods"
      description="Capabilities available on this machine."
      masterContent={
        <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
          {mappedCards.map((card) => (
            <EntityCard key={card.id} {...card} selected={selectedId === card.id} onClick={() => setSelectedId(card.id)} />
          ))}
        </div>
      }
      detailContent={selectedId ? <div className="p-4">Details for {selectedId}</div> : null}
      onCloseDetail={() => setSelectedId(null)}
    />
  );
}
