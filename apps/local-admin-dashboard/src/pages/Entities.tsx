import { useState, useEffect } from "react";
import { Plus } from "lucide-react";
import { MasterDetailLayout } from "../components/layout/MasterDetailLayout";
import { EntityCard } from "../components/shared/EntityCard";
import type { EntityCardProps } from "../components/shared/EntityCard";
import { RegistryApi } from "../services/api";

export function Entities({ hideHeader = false }: { hideHeader?: boolean }) {
  const [entities, setEntities] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedEntityId, setSelectedEntityId] = useState<string | null>(null);

  const fetchEntities = () => {
    setLoading(true);
    RegistryApi.listEntities()
      .then(setEntities)
      .catch(console.error)
      .finally(() => setLoading(false));
  };

  useEffect(() => {
    fetchEntities();
  }, []);

  // Map backend entities to EntityCardProps
  const mappedCards: EntityCardProps[] = entities.map((entity) => ({
    id: entity.entity_id,
    kind: "user", // fallback
    title: entity.display_name,
    subtitle: entity.entity_id,
    status: entity.meta?.status === "active" ? "active" : "unknown",
    statusLabel: entity.meta?.status === "active" ? "Active" : "Unknown",
    summary: `Type: ${entity.entity_type}. Roles: ${entity.roles?.join(", ") || "None"}`,
    chips: entity.roles?.map((r: string) => ({ label: r, tone: "neutral" as const })) || [],
    metrics: [],
    lastUpdatedAt: entity.meta?.updated_at,
  }));

  const selectedEntity = entities.find((e) => e.entity_id === selectedEntityId);

  const masterContent = (
    <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
      {loading ? (
        <div className="text-muted-foreground p-4">Loading entities...</div>
      ) : mappedCards.length === 0 ? (
        <div className="text-muted-foreground p-4">No entities found.</div>
      ) : (
        mappedCards.map((card) => (
          <EntityCard
            key={card.id}
            {...card}
            selected={selectedEntityId === card.id}
            onClick={() => setSelectedEntityId(card.id)}
          />
        ))
      )}
    </div>
  );

  const detailContent = selectedEntity ? (
    <div className="space-y-6">
      <div>
        <h3 className="text-xl font-bold">{selectedEntity.display_name}</h3>
        <p className="text-sm text-muted-foreground font-mono mt-1">
          {selectedEntity.entity_id}
        </p>
      </div>

      <div className="space-y-4">
        <div className="p-4 bg-muted/50 rounded-lg border">
          <h4 className="text-sm font-semibold mb-2">Details</h4>
          <dl className="grid grid-cols-2 gap-x-4 gap-y-2 text-sm">
            <dt className="text-muted-foreground">Type</dt>
            <dd>{selectedEntity.entity_type}</dd>
            <dt className="text-muted-foreground">Status</dt>
            <dd>{selectedEntity.meta?.status}</dd>
            <dt className="text-muted-foreground">Created</dt>
            <dd>{selectedEntity.meta?.created_at ? new Date(selectedEntity.meta.created_at).toLocaleString() : "N/A"}</dd>
          </dl>
        </div>

        <div className="p-4 bg-muted/50 rounded-lg border">
          <h4 className="text-sm font-semibold mb-2">Raw JSON</h4>
          <pre className="text-xs font-mono overflow-x-auto p-2 bg-background rounded border">
            {JSON.stringify(selectedEntity, null, 2)}
          </pre>
        </div>
      </div>
      
      <div className="flex gap-2 justify-end">
        <button 
          onClick={() => {
            if (confirm("Are you sure?")) {
              RegistryApi.deleteEntity(selectedEntity.entity_id).then(() => {
                setSelectedEntityId(null);
                fetchEntities();
              });
            }
          }}
          className="px-4 py-2 bg-red-500/10 text-red-500 border border-red-500/20 rounded-md text-sm font-medium hover:bg-red-500/20"
        >
          Delete Entity
        </button>
      </div>
    </div>
  ) : null;

  return (
    <MasterDetailLayout
      title={hideHeader ? "" : "Entities"}
      description={hideHeader ? undefined : "Manage humans, devices, and agents."}
      actions={
        <button className="flex items-center gap-2 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors shadow-[0_0_15px_rgba(124,58,237,0.3)]">
          <Plus className="h-4 w-4" />
          Register Entity
        </button>
      }
      masterContent={masterContent}
      detailContent={detailContent}
      onCloseDetail={() => setSelectedEntityId(null)}
    />
  );
}
