# ADR 0006 — Single-language locale configuration uses the `root` key

- **Status:** Accepted
- **Date:** 2026-11-07
- **Deciders:** Documentation Architect (autonomous run)

## Context

The brief's i18n snippet suggests `defaultLocale: 'en'` with `locales: { en: { label: 'English' } }`. With Starlight 0.42 that configuration declares **`en` as a prefixed locale directory**: every docs entry is then expected to live under `src/content/docs/en/…`, and sidebar `slug` lookups for unprefixed routes fail at build time with "The slug … does not exist" (verified empirically).

This site is English-only, with all content at the collection root, and must stay multilingual-ready (translations can be added later as prefixed directories such as `fr/`).

## Decision

```js
defaultLocale: 'en',
locales: {
  root: { label: 'English', lang: 'en' },
},
```

The `root` key maps the default language onto the unprefixed collection root — Starlight's documented pattern for "default language at the root, additional languages prefixed".

## Consequences

- Sidebar slugs resolve; the build passes with content at `src/content/docs/*.mdx`.
- Adding a translation later means creating `src/content/docs/fr/…` and adding `fr: { label: 'Français' }` to `locales`.
- UI strings remain translatable through `useTranslations()`; the locale switcher appears automatically once a second locale exists.
