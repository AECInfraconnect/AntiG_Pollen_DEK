import { type ReactNode, useState } from "react";
import { cn } from "@/lib/utils";
import type { Entity360Response } from "../../features/entity-graph/types";
import { EntityPageHeader, type EntityPageHeaderProps } from "./EntityPageHeader";
import { RelatedList, type RelatedListItem } from "./RelatedList";
import { ActivityFeed } from "./ActivityFeed";
import { EntityRelationshipPanel } from "../relationship/EntityRelationshipPanel";
import { Activity, Network, FileText } from "lucide-react";

export interface RelatedSection {
  title: string;
  icon: any;
  iconColor?: string;
  items: RelatedListItem[];
  viewAllHref?: string;
}

interface Entity360PageProps {
  header: EntityPageHeaderProps;
  /** Left column: About section with entity-specific details */
  aboutSection: ReactNode;
  /** Related entity lists shown in the right sidebar */
  relatedSections: RelatedSection[];
  /** Entity 360 data for graph and activity */
  data?: Entity360Response | null;
  /** Additional tabs beyond the default ones */
  extraTabs?: Array<{ id: string; label: string; content: ReactNode }>;
}

type TabId = "feed" | "details" | "relationships" | string;

/**
 * Entity 360° Page — Salesforce Lightning-inspired layout.
 *
 * Structure:
 * ┌─────────────────────────────────────────────────────┐
 * │  Compact Header (icon + type + name + status + btns)│
 * ├──────────┬──────────────────────┬───────────────────┤
 * │  About   │  Feed / Details tabs │  Related Lists    │
 * │  (left)  │  (center)            │  (right sidebar)  │
 * └──────────┴──────────────────────┴───────────────────┘
 *
 * No KPI cards or big stat panels. Clean, data-focused.
 */
export function Entity360Page({
  header,
  aboutSection,
  relatedSections,
  data,
  extraTabs = [],
}: Entity360PageProps) {
  const [activeTab, setActiveTab] = useState<TabId>("feed");

  const defaultTabs: Array<{ id: TabId; label: string; icon: any }> = [
    { id: "feed", label: "Feed", icon: Activity },
    { id: "details", label: "Details", icon: FileText },
    { id: "relationships", label: "Relationships", icon: Network },
  ];

  const allTabs = [
    ...defaultTabs,
    ...extraTabs.map((t) => ({ id: t.id, label: t.label, icon: FileText })),
  ];

  return (
    <div className="space-y-4">
      {/* Compact Record Header */}
      <EntityPageHeader {...header} />

      {/* Three-column layout */}
      <div className="grid gap-4 lg:grid-cols-[280px_1fr_300px] md:grid-cols-[260px_1fr]">
        {/* ─── Left Column: About ─── */}
        <div className="space-y-3">
          <section className="rounded-lg border border-border bg-card/50 p-4">
            <h3 className="mb-3 flex items-center gap-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground">
              <div className="h-1 w-1 rounded-full bg-primary" />
              About
            </h3>
            {aboutSection}
          </section>
        </div>

        {/* ─── Center Column: Feed & Details (tabbed) ─── */}
        <div className="min-w-0">
          <div className="rounded-lg border border-border bg-card/50 overflow-hidden">
            {/* Tab bar */}
            <div className="border-b border-border/50 px-4">
              <nav className="flex gap-0.5" aria-label="Entity detail tabs">
                {allTabs.map((tab) => {
                  const TabIcon = tab.icon;
                  return (
                    <button
                      key={tab.id}
                      type="button"
                      onClick={() => setActiveTab(tab.id)}
                      className={cn(
                        "flex items-center gap-1.5 whitespace-nowrap border-b-2 px-3 py-2.5 text-xs font-medium transition-colors",
                        activeTab === tab.id
                          ? "border-primary text-primary"
                          : "border-transparent text-muted-foreground hover:text-foreground",
                      )}
                    >
                      <TabIcon className="h-3.5 w-3.5" />
                      {tab.label}
                    </button>
                  );
                })}
              </nav>
            </div>

            {/* Tab content */}
            <div className="p-4">
              {activeTab === "feed" && (
                <ActivityFeed items={data?.activity ?? []} maxVisible={20} />
              )}
              {activeTab === "details" && data?.graph && (
                <div className="space-y-4">
                  <EntityRelationshipPanel
                    graph={data.graph}
                    selectedNodeId={data.entity.id}
                    compact
                  />
                </div>
              )}
              {activeTab === "relationships" && data?.graph && (
                <EntityRelationshipPanel
                  graph={data.graph}
                  selectedNodeId={data.entity.id}
                />
              )}
              {extraTabs.map(
                (tab) =>
                  activeTab === tab.id && (
                    <div key={tab.id}>{tab.content}</div>
                  ),
              )}
            </div>
          </div>
        </div>

        {/* ─── Right Column: Related Lists ─── */}
        <div className="space-y-3">
          {relatedSections.map((section) => (
            <RelatedList
              key={section.title}
              title={section.title}
              icon={section.icon}
              iconColor={section.iconColor}
              items={section.items}
              viewAllHref={section.viewAllHref}
            />
          ))}
          {relatedSections.length === 0 && (
            <div className="rounded-lg border border-dashed border-border p-6 text-center text-xs text-muted-foreground">
              No related entities found yet.
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
