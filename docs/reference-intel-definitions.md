# Reference Intel Definitions

Pollek can enrich entity details with compact, trusted external reference
metadata. This is intentionally separate from local evidence:

- **Local evidence** comes from registry, telemetry, OS probes, wrappers,
  proxies, and policy decisions.
- **Reference intel** comes from well-known external sources matched by
  observed keys such as agent name, vendor, runtime, host, URI, path, or tool
  protocol.

Reference intel must never be used as proof that access happened. It only
helps classify and explain an observed entity.

## Contract

All local and cloud components that exchange curated reference intel should use
the schema at:

`contracts/schemas/reference-intel-definition.v1.schema.json`

The current Local Dashboard seed file is:

`apps/local-admin-dashboard/src/data/entity-reference-intel.compact.json`

Keep this file compact. Add short summaries, source labels, source URLs,
review dates, and control notes. Do not embed long articles, scraped pages, or
large vendor datasets in this file.

Definitions may include `expectedCapabilities`. These are not evidence. They are
standard capability hints that dashboards can compare with locally observed
capabilities. A detected capability can be shown as confirmed only when local
evidence, registry data, telemetry, or wrapper/proxy observations match it.

Definitions may also include `visualIdentity`. This is a compact dashboard hint
for inline rendering: icon key, short mark, colors, and source label. Do not
embed large logo files or remote image URLs in the compact seed file. Official
logo asset packs can be layered later, but the seed file should remain small and
should always fall back to the v1 visual hint shape.

## Display Rules

Dashboards must label reference intel separately from local evidence. Recommended
detail labels are:

- `Source`: the external source label.
- `Reviewed`: the review date.
- `Control note`: a short operational interpretation for Pollek.
- `Known capability checklist`: expected capabilities, marked detected only when
  local evidence matches the definition keywords.
- `Visual identity`: a small inline icon or mark sourced from curated
  definition metadata. This is UI context, not proof of access and not a policy
  decision input.

Local Dashboard and Pollek Cloud may maintain richer catalogs later, but the
shared shape should remain compatible with the v1 schema unless Contract Hub is
updated.
