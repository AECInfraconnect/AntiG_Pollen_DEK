# UX/UI Redesign: Entity-Centric 360° Navigation

## Design Philosophy

The Pollek Dashboard is being redesigned around a **Salesforce Lightning-inspired Entity-Centric** model where:

1. **Every entity page is a 360° view** — showing the entity's details, related entities, activity timeline, policies, and cost in one screen
2. **Navigation follows user workflow** — Observe → Understand → Govern → Verify
3. **Relationships are always visible** — from any entity, users can see connected entities via Related Lists and a mini relationship graph
4. **Mode-aware menus** — Simple shows fewer items, Advanced shows more, Enterprise shows all

## New Navigation Structure

### Side Menu (Consolidated from 7 groups → 4 groups)

| Group | Simple | Advanced | Enterprise | Purpose |
|-------|--------|----------|------------|---------|
| **Home** | ✓ | ✓ | ✓ | Overview dashboard |
| **Registry** | | | | |
| → Agents & Models | ✓ | ✓ | ✓ | All AI agents with 360° detail |
| → Tools & Resources | ✓ | ✓ | ✓ | Combined tools + data resources |
| → Identities | | ✓ | ✓ | SPIFFE/OAuth bindings |
| → Entity Graph | | ✓ | ✓ | Full relationship visualization |
| **Governance** | | | | |
| → Policies | ✓ | ✓ | ✓ | Active policies with impact view |
| → Policy Presets | | ✓ | ✓ | Pre-built templates |
| → Deployments | ✓ | ✓ | ✓ | Enforcement status |
| → Simulator | | ✓ | ✓ | What-if testing |
| **Observe** | | | | |
| → Activity | ✓ | ✓ | ✓ | Timeline with full context |
| → Alerts | ✓ | ✓ | ✓ | Shadow AI + security alerts |
| → Cost & Tokens | ✓ | ✓ | ✓ | Spending breakdown |
| → Health | | ✓ | ✓ | System diagnostics |
| **System** | | | | |
| → Scan & Discover | ✓ | ✓ | ✓ | Auto-discovery trigger |
| → Integrations | | ✓ | ✓ | External connections |
| → Bundles & Sync | | | ✓ | Cloud sync |
| → Settings | ✓ | ✓ | ✓ | Configuration |

## Entity 360° Page Layout (Salesforce Lightning-style)

```
┌─────────────────────────────────────────────────────────────────────┐
│ [Icon] Entity Type                                                   │
│ Entity Name                          [Follow] [Edit] [Actions ▾]    │
│ Status Badge  •  Mode Badge  •  Last Seen: 2 min ago                │
├────────────────────────┬────────────────────────────────────────────┤
│                        │                                            │
│  ┌─ About ──────────┐  │  ┌─ Activity Feed ──────────────────────┐  │
│  │ Property: Value   │  │  │ Filters: All time • All types       │  │
│  │ Property: Value   │  │  │                                     │  │
│  │ Property: Value   │  │  │ ▾ Today                             │  │
│  │ Property: Value   │  │  │ ● Agent called tool_x → Allow      │  │
│  └───────────────────┘  │  │   Policy: cost-guard • PEP: proxy   │  │
│                        │  │ ● Agent accessed resource_y → Deny   │  │
│  ┌─ Capabilities ───┐  │  │   Policy: data-guard • PEP: ebpf    │  │
│  │ • MCP Client      │  │  └─────────────────────────────────────┘  │
│  │ • File Access     │  │                                            │
│  │ • Network         │  │                                            │
│  └───────────────────┘  │                                            │
│                        │                                            │
├────────────────────────┴────────────────────────────────────────────┤
│                                                                     │
│  ┌─ Related: Policies (3) ──┐  ┌─ Related: Tools (5) ────────────┐ │
│  │ [🛡] cost-guard          │  │ [🔧] file_read                   │ │
│  │   Status: Enforcing      │  │   Last used: 5 min ago           │ │
│  │   Engine: Cedar           │  │   Decisions: 42 allow, 3 deny   │ │
│  │                          │  │                                   │ │
│  │ [🛡] data-access-policy  │  │ [🔧] web_search                  │ │
│  │   Status: Observe-only   │  │   Last used: 1 hour ago          │ │
│  │   Engine: OPA             │  │   Decisions: 18 allow            │ │
│  │                          │  │                                   │ │
│  │ View All →               │  │ View All →                        │ │
│  └──────────────────────────┘  └───────────────────────────────────┘ │
│                                                                     │
│  ┌─ Related: Resources (2) ─┐  ┌─ Cost Summary ──────────────────┐ │
│  │ [📁] project-docs        │  │ Today: $2.45 (1,240 tokens)     │ │
│  │   Type: Folder            │  │ This week: $18.90               │ │
│  │   Policy: data-access     │  │ Top model: gpt-4o (78%)         │ │
│  └──────────────────────────┘  └───────────────────────────────────┘ │
│                                                                     │
│  ┌─ Mini Relationship Graph ────────────────────────────────────────┐│
│  │         [Tool A] ──── [Agent] ──── [Policy X]                    ││
│  │                          │                                       ││
│  │                     [Resource B]                                  ││
│  └──────────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────┘
```

## Key UX Principles Applied

1. **No dead-end pages** — Every entity links to its related entities
2. **Context always visible** — Activity shows which policy, which agent, which tool
3. **Progressive disclosure** — Simple mode shows essentials, Advanced reveals depth
4. **Consistent card pattern** — All Related Lists use the same card component
5. **Bidirectional navigation** — Click a policy from Agent page → see that policy's 360° with this agent highlighted in Related
