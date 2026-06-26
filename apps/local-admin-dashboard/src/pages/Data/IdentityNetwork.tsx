import { useState, useEffect } from "react";
import { UserCircle, Network, Activity, Info } from "lucide-react";
import { useSearchParams } from "react-router-dom";
import { RegistryApi } from "../../services/api";
import type { Entity, Relationship } from "../../services/types";
import { MasterDetailLayout } from "../../components/master-detail/MasterDetailLayout";
import { EntityCard } from "../../components/master-detail/EntityCard";
import { DetailPane } from "../../components/master-detail/DetailPane";
import { EmptyState } from "../../components/master-detail/EmptyState";

export function IdentityNetwork() {
  const [entities, setEntities] = useState<Entity[]>([]);
  const [relationships, setRelationships] = useState<Relationship[]>([]);
  const [loading, setLoading] = useState(true);
  const [params, setParams] = useSearchParams();
  const selectedId = params.get("selected") ?? undefined;

  const loadData = () => {
    setLoading(true);
    Promise.all([RegistryApi.listEntities(), RegistryApi.listRelationships()])
      .then(([ents, rels]) => {
        setEntities(ents);
        setRelationships(rels);
      })
      .catch(console.error)
      .finally(() => setLoading(false));
  };

  useEffect(() => {
    loadData();

    const source = new EventSource("/v1/telemetry/observations/stream");
    source.onmessage = () => {
      // Refresh identity graph on telemetry update
      Promise.all([RegistryApi.listEntities(), RegistryApi.listRelationships()])
        .then(([ents, rels]) => {
          setEntities(ents);
          setRelationships(rels);
        })
        .catch(console.error);
    };

    return () => source.close();
  }, []);

  const select = (id: string) =>
    setParams((p) => {
      p.set("selected", id);
      return p;
    });

  return (
    <div className="p-6 md:p-8 space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold tracking-tight">
            Identity & Network
          </h2>
          <p className="text-sm text-muted-foreground">
            Manage your local identity graph: people, systems, and their
            relationships.
          </p>
        </div>
      </div>

      <MasterDetailLayout
        idSelector={(x: any) => x.identity_id || x.id}
        items={entities}
        loading={loading}
        selectedId={selectedId}
        onSelect={select}
        toolbar={
          <div className="flex items-center gap-2 mb-4">
            <input
              type="text"
              placeholder="Search identities..."
              className="px-3 py-1.5 text-sm rounded-md border bg-background"
            />
          </div>
        }
        emptyState={
          <EmptyState
            icon={Network}
            title="No identities found"
            description="The identity graph is currently empty."
          />
        }
        renderCard={(e, selected) => {
          const isGoverned = e.meta?.status === "active";
          return (
            <EntityCard
              title={e.display_name}
              subtitle={e.entity_type}
              icon={UserCircle}
              status={isGoverned ? "ok" : "idle"}
              statusLabel={isGoverned ? "Governed" : "Unmanaged"}
              meta={[{ label: "Roles", value: e.roles?.length || 0 }]}
              selected={selected}
            />
          );
        }}
        renderDetail={(e) => {
          const isGoverned = e.meta?.status === "active";
          const related = relationships.filter(
            (r) =>
              r.subject.object_id === e.entity_id ||
              r.object.object_id === e.entity_id,
          );

          return (
            <DetailPane
              title={e.display_name}
              subtitle={e.entity_type}
              status={isGoverned ? "ok" : "idle"}
              statusLabel={isGoverned ? "Governed" : "Unmanaged"}
              tabs={[
                {
                  id: "overview",
                  label: "Overview",
                  content: (
                    <div className="space-y-6">
                      <div className="grid grid-cols-2 gap-4 text-sm">
                        <div className="p-4 bg-muted/30 rounded-xl border">
                          <span className="text-muted-foreground block mb-1">
                            ID
                          </span>
                          <span className="font-mono text-xs">
                            {e.entity_id}
                          </span>
                        </div>
                        <div className="p-4 bg-muted/30 rounded-xl border">
                          <span className="text-muted-foreground block mb-1">
                            Roles
                          </span>
                          <span className="capitalize">
                            {e.roles?.join(", ") || "None"}
                          </span>
                        </div>
                      </div>

                      <div>
                        <h4 className="font-medium mb-2 flex items-center gap-2 text-sm">
                          <Info className="h-4 w-4" /> Raw Data
                        </h4>
                        <pre className="text-[10px] font-mono bg-muted/50 p-4 rounded-lg overflow-x-auto border">
                          {JSON.stringify(e, null, 2)}
                        </pre>
                      </div>
                    </div>
                  ),
                },
                {
                  id: "relationships",
                  label: "Relationships",
                  content: (
                    <div className="space-y-4">
                      {related.length === 0 ? (
                        <div className="flex flex-col items-center justify-center p-8 text-center border border-dashed rounded-lg text-muted-foreground">
                          <Network className="h-8 w-8 mb-2 opacity-50" />
                          <p className="text-sm">
                            No relationships defined for this identity.
                          </p>
                        </div>
                      ) : (
                        <div className="rounded-md border bg-card/30">
                          <table className="w-full text-sm text-left">
                            <thead className="bg-muted/50">
                              <tr>
                                <th className="px-4 py-2 font-medium">
                                  Relation
                                </th>
                                <th className="px-4 py-2 font-medium">
                                  Target
                                </th>
                              </tr>
                            </thead>
                            <tbody className="divide-y divide-border">
                              {related.map((r) => {
                                const isSubject =
                                  r.subject.object_id === e.entity_id;
                                const target = isSubject ? r.object : r.subject;
                                return (
                                  <tr key={r.relationship_id}>
                                    <td className="px-4 py-3 font-medium text-primary">
                                      {isSubject
                                        ? `${r.relation} ➔`
                                        : `⬅ ${r.relation}`}
                                    </td>
                                    <td className="px-4 py-3 font-mono text-xs">
                                      {target.object_type}:{target.object_id}
                                    </td>
                                  </tr>
                                );
                              })}
                            </tbody>
                          </table>
                        </div>
                      )}
                    </div>
                  ),
                },
                {
                  id: "activity",
                  label: "Activity",
                  content: (
                    <div className="flex flex-col items-center justify-center p-8 text-center border border-dashed rounded-lg text-muted-foreground">
                      <Activity className="h-8 w-8 mb-2 opacity-50" />
                      <p className="text-sm">No recent activity found.</p>
                    </div>
                  ),
                },
              ]}
            />
          );
        }}
      />
    </div>
  );
}
