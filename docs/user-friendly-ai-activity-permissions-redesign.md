# User-Friendly AI Activity & Permissions Redesign

Status: first implementation pass for `apps/local-admin-dashboard`

## Product Direction

Pollek Simple Mode should behave like an AI activity and permission manager,
not like a policy console. The primary user questions are:

1. Which AI apps are on my computer?
2. What did each AI app read, write, open, connect to, run, or call?
3. Was the activity allowed, blocked, asked first, or only watched?
4. Can Pollek control this on my OS today?
5. If Pollek can only observe it, what should I change in the AI app settings?
6. What happened historically, grouped by AI app, data target, and result?

The redesign is intentionally Observe-first. Enforcement remains valuable, but
the default user value is broad, explainable visibility plus clear next steps.

## Research Basis

- Usable security research argues that permission prompts must be contextual and
  not too frequent. Wijesekera et al. found that users wanted to block some
  unexpected data accesses, but excessive prompting can cause habituation:
  https://arxiv.org/abs/1504.03747
- A 2025 user study of iOS App Privacy Report found that transparency is useful
  but users need clearer access purpose and domain descriptions:
  https://arxiv.org/abs/2511.00467
- NIST Privacy Framework 1.0 is an enterprise privacy risk management tool, and
  the 1.1 draft work emphasizes current privacy needs and usability:
  https://www.nist.gov/privacy-framework/privacy-framework
  https://www.nist.gov/privacy-framework/new-projects/privacy-framework-version-11
- NIST AI RMF frames AI risk management around trustworthiness across design,
  development, use, and evaluation. Pollek maps this to find AI apps, map
  access, measure activity/risk/cost, and manage rules:
  https://www.nist.gov/itl/ai-risk-management-framework
- OpenTelemetry defines telemetry signals as traces, metrics, logs, and baggage;
  this supports a normalized activity model that can accept local OS events,
  agent events, MCP events, and cloud events later:
  https://opentelemetry.io/docs/concepts/signals/
- W3C Trace Context standardizes trace correlation across systems. Pollek should
  keep stable trace IDs for advanced inspection while translating them into
  plain-language activity for Simple Mode:
  https://www.w3.org/TR/trace-context/
- MCP explicitly treats tools and data access as powerful operations requiring
  user consent, control, clear authorization UI, and privacy protection:
  https://modelcontextprotocol.io/specification/2025-06-18

## Cross-OS Observe/Control Layers

Pollek should represent capabilities as a matrix instead of promising uniform
enforcement on every computer:

1. Observe: collect metadata about files, folders, web domains, commands,
   apps/processes, AI model usage, MCP tools, email/calendar connectors, and
   cost without storing raw file content, email bodies, prompts, or responses.
2. Explain: map technical telemetry into `AI app -> action -> target -> result`
   with purpose hints and setup notes.
3. Warn/Ask: raise user attention for sensitive or unusual activity when the OS,
   connector, or agent integration supports it.
4. Block: apply enforcement only when a local capability reports it can do so
   reliably, or guide the user to configure the AI app directly.

Examples of host-dependent sources:

- Windows can use ETW for high-volume event tracing and WFP for network filtering
  or monitoring surfaces:
  https://learn.microsoft.com/en-us/windows/win32/etw/about-event-tracing
  https://learn.microsoft.com/en-us/windows/win32/fwp/windows-filtering-platform-start-page
- Linux can use fanotify for filesystem notifications and permission decisions:
  https://man7.org/linux/man-pages/man7/fanotify.7.html
- macOS support should stay capability-driven and permission-aware. Endpoint
  Security is the likely privileged system-event integration point, but the UI
  must explain setup requirements clearly:
  https://developer.apple.com/documentation/endpointsecurity

## Product Layers

1. Source adapters
   - OS file/process/network observers.
   - Browser, email, calendar, model, cost, and MCP observers.
   - Agent-specific connectors where available.
2. Event normalization
   - Convert source events into a stable user-facing schema.
   - Current implementation starts this in
     `src/features/user-activity/userActivityModel.ts`.
3. Capability matrix
   - Convert host capabilities into `Watch`, `Warn`, `Ask first`, and `Block`.
   - Simple Mode must say when Pollek can only observe and when the user should
     configure the AI app itself.
4. Rules and decisions
   - Simple presets are expressed as watch/allow/ask/block intents.
   - Advanced policy pages remain available for technical users.
5. UX surfaces
   - Simple Mode: My AI Apps, AI Activity, Data & Apps, Allowed & Blocked,
     Setup, History.
   - Advanced Mode: Agents, Tools & Resources, Policies, Deployments,
     Activity Timeline, Capabilities, Health.
6. Future extension boundary
   - Plugin work should wait for the user's additional guideline.
   - Future third-party extensions should declare source type, event schema,
     privacy class, setup actions, OS support, capability level, and data
     retention behavior before they can contribute activity.

## Implemented In This Pass

- Added a user-friendly activity schema and mapping layer.
- Replaced Simple Mode navigation with user language.
- Added Simple Mode pages for home, AI apps, activity, data/apps, rules,
  setup capability, and history reports.
- Preserved Advanced Mode technical pages and routes.
- Kept enforcement as capability-dependent and made AI app settings a first-class
  next step when Pollek can only observe.

## Next Development Pass

1. Wire backend endpoints to produce first-class `user-friendly-activity.v1`
   instead of mapping from the existing entity timeline only in the dashboard.
2. Add OS-specific observer status details and missing-permission setup flows.
3. Add retention settings and export/delete controls.
4. Add event examples and tests for file, folder, website, command, email,
   MCP tool, model, token/cost, allowed, blocked, asked, and watched-only cases.
5. Pause before implementing third-party plugin SDK/manifest work and ingest the
   user's additional plugin guideline.
