# ADR 0004 — Provisional `site` URL for llms.txt generation

- **Status:** Accepted
- **Date:** 2026-11-07
- **Deciders:** Documentation Architect (autonomous run)

## Context

Two requirements interact:

1. The brief says not to set `site`/`base` unless the deployment requires it.
2. The AI-agent discoverability requirement (generate `llms.txt` / `llms-full.txt` at the site root) is satisfied with the `starlight-llms-txt` plugin — and that plugin **throws at setup unless `site` is configured**, because llms.txt files must contain absolute URLs.

## Decision

Set `site: 'https://quoin-docs.example.com'` and no `base`. The `.example.com` suffix is the reserved-for-documentation domain, chosen precisely so it cannot be mistaken for a real deployment. The value exists to make `pnpm build` (and therefore llms.txt emission) possible.

**Before deploying**, replace `site` in `astro.config.mjs` with the real production origin. No other change is needed: Starlight resolves all internal links relative to `site`/`base` at build time.

## Consequences

- `pnpm build` emits `/llms.txt`, `/llms-small.txt`, and `/llms-full.txt` with absolute URLs.
- The generated files are only fully meaningful once the real origin is configured; until then their URLs point at the provisional domain.
- `base` remains unset; if the real host serves the site from a subpath (e.g. GitHub Pages project pages), set `base` at the same time.
