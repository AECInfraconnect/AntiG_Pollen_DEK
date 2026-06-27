# Contextual Help Catalog

Pollek uses a compact contextual help catalog to show short explanations inside
the Local Dashboard and, later, Pollek Cloud. The goal is to keep users in the
workflow while still giving them enough context to understand a function, form,
entity, or policy decision.

## Contract

Shared catalog entries should follow:

`contracts/schemas/contextual-help-topic.v1.schema.json`

The Local Dashboard seed file is:

`apps/local-admin-dashboard/src/data/contextual-help.compact.json`

Each topic has:

- `id`: stable topic key used by UI components and future cloud surfaces.
- `title`: short label shown in the help popover.
- `summary`: one compact explanation.
- `guidance`: short actionable bullets.
- `sourceDoc` and optional `sourceAnchor`: where the help text is derived from.
- `relatedTopicIds`: nearby concepts for future richer help drawers.

## Rules

- Keep popup content short enough to read without leaving the page.
- Link help text back to repo docs, but do not require users to open the docs for
  basic understanding.
- Treat help text as documentation, not evidence. It must not affect policy
  allow or deny decisions.
- Local Dashboard and Pollek Cloud should use the same topic IDs for the same
  concepts.

## Generated Docs

Run this script after editing the compact catalog:

`node scripts/docs/build-contextual-help-docs.mjs`

It regenerates:

`docs/generated/contextual-help-catalog.md`
