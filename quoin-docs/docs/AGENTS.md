# AGENTS.md — Working on the Quoin documentation

Instructions for humans and AI agents contributing to this Starlight documentation site for [elcoosp/quoin](https://github.com/elcoosp/quoin).

## Stack

- Astro **7.4.x** + Starlight **0.42.x** (verified 2026-11-07; Astro 7.4 is the floor).
- TypeScript pinned to **6.0.3** — `astro check` does not support TypeScript 7 (see `docs/decisions/0001-typescript-version.md`).
- Package manager of record: **pnpm**.

## Commands

```sh
pnpm install        # install dependencies
pnpm dev            # local dev server
pnpm build          # production build (fails on broken internal links)
pnpm check          # astro check (TypeScript 6.x)
pnpm lint           # markdownlint-cli2 over src/content/docs
```

`pnpm build` must pass before any change is merged. `starlight-links-validator` makes broken internal links a build failure.

## Information architecture

The site follows the Diátaxis framework — one type per page, never mixed:

| Type | Folder | Answers |
| --- | --- | --- |
| Tutorial | `src/content/docs/tutorials/` | "Walk me through it" (learning-oriented) |
| How-to | `src/content/docs/guides/` | "How do I…?" (task-oriented) |
| Reference | `src/content/docs/reference/` | "What is…?" (information-oriented) |
| Explanation | `src/content/docs/explanation/` | "Why…?" (understanding-oriented) |

Every page carries required frontmatter:

```yaml
---
title: "Clear, Searchable Title"
description: "One-sentence summary for SEO and AI agents."
type: tutorial | how-to | reference | explanation
---
```

The `type` field is enforced by the extended docs schema (`src/content.config.ts`) — the build fails without it.

## Writing standards

- Second person ("you"), present tense, active voice, imperative instructions.
- Never write "simply", "just", "easy", or "obviously".
- Every page: frontmatter, one-sentence summary, prerequisites (if any), main content, **Next steps** via `<LinkCard />`.
- Code blocks always declare a language; show expected output where relevant.
- Reference pages use tables: signature, type, default, description, example.

## Accuracy rules

- **Never invent features, flags, or options.** Document only what exists in the Quoin source. When unsure, read the crate source in the repository (doc comments are the source of truth).
- Quoin is a Rust workspace: user-facing commands use `cargo`/`just`, not JavaScript tooling.
- Version claims (Rust 1.88+, crate versions, feature lists) must match the repository manifests.

## Branding

- Accent palette is derived from the repository logo and documented in `docs/decisions/0003-branding-accent.md`; it passes WCAG AA in both themes. Do not reintroduce Starlight's default blue.
- The logo asset in `src/assets/` comes from the repository's `docs/logo.png`; do not substitute placeholder art.
- Dark mode is supported and must stay supported: check changes in both themes.

## AI discoverability

`starlight-llms-txt` emits `/llms.txt`, `/llms-small.txt`, and `/llms-full.txt` at build time. The `site` value in `astro.config.mjs` is a provisional placeholder (`quoin-docs.example.com`) — replace it with the real origin when deploying (see `docs/decisions/0004-site-url-and-llms-txt.md`).

## Decisions

Architectural decisions live in `docs/decisions/` as numbered ADRs. When you change tooling, branding, schema, or plugins, add a new ADR rather than silently diverging.
