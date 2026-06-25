import { useState } from "react";
import { MasterDetailLayout } from "../components/layout/MasterDetailLayout";
import { EntityCard } from "../components/shared/EntityCard";
import type { EntityCardProps } from "../components/shared/EntityCard";

export function LocalEvidence() {
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const mappedCards: EntityCardProps[] = [
    {
      id: "ev-1",
      kind: "evidence",
      title: "File Integrity Proof",
      status: "ready",
      statusLabel: "Verified",
      summary: "SHA256 signature match for Claude Desktop",
      chips: [{ label: "cryptographic", tone: "success" }],
    }
  ];

  return (
    <MasterDetailLayout
      title="Local Evidence"
      description="Cryptographic proofs and environment integrity data."
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
