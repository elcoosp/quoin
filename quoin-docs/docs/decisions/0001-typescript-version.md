# ADR 0001 — Pin TypeScript to 6.x for `astro check`

- **Status:** Accepted
- **Date:** 2026-11-07
- **Deciders:** Documentation Architect (autonomous run)

## Context

The documentation site runs `astro check` (Astro 7.4.7) as a quality gate. At execution time, npm resolves these versions:

- `typescript` `latest` dist-tag: **7.0.2** (7.1.0 in beta/dev)
- Latest stable 6.x release: **6.0.3**

As of Astro 7.4, `astro check` still does not support TypeScript 7. Running it with a TypeScript 7 install fails with:

> `astro check does not currently support TypeScript 7.0. To continue using astro check, install TypeScript 6 instead.`
>
> `astro check will be deprecated in a future Astro release. Experimental support for type-checking Astro files with TypeScript 7.1+ is available through @astrojs/ts-content-mapper. See its README for setup instructions: https://github.com/withastro/astro/tree/main/packages/language-tools/ts-content-mapper#usage`

(Quoted verbatim from Astro 7.4.7's check CLI, which hard-fails when `getPackageVersion("typescript")` starts with `7`.)

## Decision

Pin `typescript` to **`6.0.3`** (exact, not a caret range) in `devDependencies`. `pnpm check` therefore runs `astro check` against a supported TypeScript release.

## Consequences

- `pnpm check` is a reliable release blocker.
- Upgrading to TypeScript 7 requires revisiting this ADR once `astro check` announces support; the pin is exact so nothing drifts silently.
