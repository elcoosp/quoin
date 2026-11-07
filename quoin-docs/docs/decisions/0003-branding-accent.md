# ADR 0003 — Branding: logo and derived accent color

- **Status:** Accepted
- **Date:** 2026-11-07
- **Deciders:** Documentation Architect (autonomous run)

## Context

Starlight's default branding (blue accent, default logo) must not ship. The repository contains exactly one brand asset: `docs/logo.png` (1264×1264, RGBA, transparent background) — a 3D interlocking-ring sculpture in rust/burnt orange and dark slate.

## Decision

**Logo.** Use the real asset for both theme variants: the sculpture is mid-saturation mid-lightness, so it reads correctly on both Starlight's light and dark backgrounds, and it contains no wordmark (so `replacesTitle` stays unset and the accessible text title "Quoin" renders normally). A square 256×256 crop of the same PNG is generated as `public/favicon.png` and wired through the `head` config.

**Accent.** The dominant saturated hue of the logo was extracted programmatically (HLS bucketing of opaque, non-gray pixels): hue ≈ 10–20°, i.e. rust/burnt orange. The palette:

| Variable | Light (`:root`) | Dark (`[data-theme='dark']`) |
| --- | --- | --- |
| `--sl-color-accent-low` | `hsl(16, 65%, 96%)` | `hsl(14, 40%, 15%)` |
| `--sl-color-accent` | `hsl(14, 71%, 36%)` | `hsl(20, 88%, 60%)` |
| `--sl-color-accent-high` | `hsl(10, 71%, 28%)` | `hsl(28, 92%, 74%)` |

Contrast was verified numerically against WCAG relative luminance:

- light accent on white: **6.9:1**, light accent-high on white: **9.9:1**
- dark accent on Starlight's dark background (`hsl(219 28% 10%)`): **6.6:1**, dark accent-high: **10.5:1**

All text-bearing pairs exceed the 4.5:1 AA requirement.

**Theme toggle.** Starlight's built-in light/dark toggle is kept; per-theme values use the supported `data-theme` attribute pattern in `src/styles/custom.css`, registered via `customCss`.

## Consequences

- No invented branding: the only image asset ships from the repository itself.
- If the project later produces a dedicated wordmark or dark-mode-tuned logo, add `logo.light` / `logo.dark` variants and update this ADR.
