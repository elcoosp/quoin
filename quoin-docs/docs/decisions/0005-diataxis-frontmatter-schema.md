# ADR 0005 — Enforce the Diátaxis `type` in frontmatter schema

- **Status:** Accepted
- **Date:** 2026-11-07
- **Deciders:** Documentation Architect (autonomous run)

## Context

Every documentation page must declare its Diátaxis type — `tutorial`, `how-to`, `reference`, or `explanation` — and one type per page, never mixed. A convention enforced only by review decays; the build can enforce it.

## Decision

Extend Starlight's docs schema in `src/content.config.ts`:

```ts
schema: docsSchema({
  extend: z.object({
    type: z.enum(['tutorial', 'how-to', 'reference', 'explanation']),
  }),
}),
```

The field is **required** (not optional), so `pnpm build` fails on any page missing the field or using an unknown value.

## Consequences

- The information architecture is machine-checked on every build.
- The landing page (`index.mdx`) carries `type: explanation` — it is conceptual orientation, not task or API content.
- New Diátaxis types (if ever adopted) require a schema change and an ADR update.
