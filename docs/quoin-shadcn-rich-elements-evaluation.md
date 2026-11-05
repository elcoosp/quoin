# Leveraging shadcn/ui-Rust Ports for Unified Rich Elements in `quoin_render!`

**Evaluation report** · 2026-11-05 · prepared against quoin @ `cb622a3` (fresh clone of `elcoosp/quoin` main)

> **STATUS UPDATE — Phase 0 + Phase 1 are LANDED on this branch.**
> The canonical contract is checked-in Rust (`quoin-macros-core/src/emit/shadcn_spec.rs`),
> `floem-shadcn` is vendored under `third_party/floem-shadcn` with the full feature
> chain (`quoin/floem-shadcn`), the Tier-1 emitters lower for real (47/47 conformance
> tests run against the shadcn components), Leptos features broadened 8 → 28 (wasm32-verified),
> and the canonical `variant:`/`size:` grammar is wired into GPUI/Leptos/Dioxus/Floem.
> See §10 for the landed inventory and the verification matrix.

---

## 1. Executive summary

quoin already has the hard part in place: a framework-neutral render DSL (`quoin_render!`, ~45 known elements), per-framework emitters that branch on cargo features, a machine-checked parity table for Floem/Xilem, and **two working shadcn integrations** (`leptos-shadcn` → `harana-oss/leptos-shadcn-ui`, `dioxus-shadcn` → `MBeliou/shadcn-dioxus`). What is missing is (a) the two GPU-native backends (GPUI, Floem) having a *shadcn-backed* emit path, (b) wider component coverage behind the existing flags, and (c) a written-down **canonical component contract** so all five lowerings stay in lockstep.

The opportunity is real and unusually cheap on two of five backends because the ideal backing libraries already exist and are quality code:

| Backend | Backing library | Status | Verdict |
|---|---|---|---|
| **Floem** | [`elcoosp/floem-shadcn`](https://github.com/elcoosp/floem-shadcn) — 55 modules, 290 exported types, **100 % shadcn/ui v4 catalog**, OKLCH light/dark theme, builder API, headless test harness | your own project, tracks `floem` git main exactly like `quoin-floem` | **Integrate now** — highest ROI, a `floem-shadcn` emitter is mostly 1:1 mapping |
| **Leptos** | [`harana-oss/leptos-shadcn-ui`](https://github.com/harana-oss/leptos-shadcn-ui) — 63 component crates, v0.9.1 | **already integrated** (git rev pin, 8 components enabled) | **Expand** — enable more crates, extend the emitter set |
| **GPUI** | [`longbridge/gpui-kit`](https://github.com/longbridge/gpui-kit) `gpui-component` — 75+ components, the reference backend | already the reference lowering; harmonization gap list already filed upstream (discussion [#2311](https://github.com/longbridge/gpui-kit/discussions/2311)) | **Adopt as canonical anchor**, push #2311 items |
| **Dioxus** | `MBeliou/shadcn-dioxus` (28 modules, already integrated) + [`yuxuetr/dioxus-ui`](https://github.com/yuxuetr/dioxus-ui) `dioxus-shadcn` (83 components, `dxui` CLI) + first-party [`DioxusLabs/components`](https://github.com/DioxusLabs/components) unstyled primitives | partial integration | **Hybrid**: keep MBeliou, fill Table/Dialog gaps from `dioxus-shadcn` / `dioxus-primitives` |
| **Xilem** | nothing mature exists (no component ecosystem yet) | generic lowering only | **Defer** — keep the generic emitter; optionally hand-roll the top-10 widgets later |

**Recommendation.** Do it in three phases: (1) write the canonical element/prop/variant contract into quoin (a Rust module, not just a doc, so the parity test enforces it); (2) land `floem-shadcn` + broaden `leptos-shadcn`/`dioxus-shadcn` — this gives *five-framework rich elements* with per-framework native quality; (3) drive gpui-kit #2311 toward the canonical spec and treat Xilem as generic-only until an ecosystem appears. Estimated emitter work: **~2–4 weeks** for phase 2, the rest incremental.

---

## 2. Scope and method

- Re-cloned `elcoosp/quoin` (main @ `cb622a3`, includes the pushed muda-stub work) and `elcoosp/rs-query` (@ `de39a6a`, wasm32 fixes); `gpui-navi` remote unchanged, local state preserved.
- Cloned and read: `elcoosp/floem-shadcn`, `harana-oss/leptos-shadcn-ui`, `MBeliou/shadcn-dioxus` (component inventory + API shapes).
- Read quoin internals: `quoin-macros-core/src/render_ast.rs` (the DSL parser + `KNOWN_ELEMENTS`), `emit/parity.rs` (machine-checked Floem/Xilem parity), `emit/render_{leptos,dioxus,gpui,floem,xilem}/` (per-framework lowerings), `quoin-ui/src/node.rs` (the router-driven `Node` tree), `quoin-ui-gpui/src/node_renderer.rs` (gpui-component lowering), `docs/leptos-dioxus-emission-parity.md` (the existing gap table).
- Web research on the rest of the Rust shadcn ecosystem (sources listed in §9).

---

## 3. Where rich elements plug into quoin today

quoin has **three element channels**; a unified shadcn layer must serve all three:

1. **`quoin_render!` macro DSL** (component-level UI, e.g. the navi demo shell). Input is parsed into `RenderNode` (element / text / expr / `if[…]` / `for[…]`), then lowered by a per-framework emitter in `quoin-macros-core/src/emit/render_*/`. The parser owns a `KNOWN_ELEMENTS` table (currently `div, h1–h3, p, text, span, button, input, label, img, svg, a, ul, ol, li, hr, br, textarea, select, form, tabs, tab, data_table, column, virtual_list, dropdown_menu, rich_text, clipboard_button, item, badge, styled_text, icon, scroll_area, tab_bar, separator, skeleton*, progress, checkbox, switch, radio_group, radio, slider, tooltip`).
2. **`quoin_ui::Node`** (runtime tree used by navi routes via `define_route!(…, node: …)`). Node kinds: `Div`+`StyleCmd`, `Text`, `StyledText`, `Icon`, `Button` (variant/size), `Input`, `TabBar`, `Table`, `VirtualList`, `DropdownMenu` items, `Clipboard`, `ScrollRestoration`, `Custom`. Lowered by each adapter (`quoin-ui-gpui` → gpui-component, and equivalents).
3. **Host bootstrap** (`run_app!` for GPUI/Xilem/Floem; `LaunchBuilder` / `mount_to_body` for Dioxus/Leptos) — irrelevant to component choice but part of the "same app source" story proven by the navi demos.

**Existing shadcn plumbing** (already in main):

- `quoin` facade features: `leptos-shadcn` and `dioxus-shadcn` forward to `quoin-leptos?/shadcn` + `quoin-macros?/leptos-shadcn` (resp. dioxus). Exactly the "no second flag to remember" pattern you asked for with `gpui-rsx-classes`.
- `quoin-leptos` pins `leptos-shadcn-ui` at rev `b16956f…` with features `button, input, tabs, badge, table, tooltip, scroll-area, dropdown-menu`.
- `quoin-dioxus` pins `MBeliou/shadcn-dioxus` (`package = "ui"`).
- The Leptos emitter has 17 shadcn-capable element handlers (button, input, badge, checkbox, switch, radio, slider, progress, separator, skeleton*, scroll_area, tabs, tooltip, dropdown_menu, data_table, styled_text, virtual_list), each with a **plain-HTML/Tailwind fallback** when the feature is off (`#[cfg(not(feature = "leptos-shadcn"))] → emit_*_html`).
- The Dioxus emitter mirrors this for its subset (button, input, checkbox/switch via `form_controls`, tabs, tooltip, dropdown_menu, separator, data_table, virtual_list).
- The GPUI emitter lowers to **gpui-component** widgets directly (this *is* the shadcn-class layer for GPUI) — button variants, `Input`+`InputState`, `TabBar`, `DataTable`+`TableDelegate`, `PopupMenu`/`DropdownMenu`, `Clipboard`, `VirtualList`.
- Floem/Xilem emitters lower to native primitives (style/text/widgets), governed by the `parity.rs` contract: `Native` / `Partial(reason)` / `Unsupported(reason)` per element — **the exact mechanism a shadcn layer should slot into**.

---

## 4. The candidate libraries (studied at code level)

### 4.1 Floem — `elcoosp/floem-shadcn` ⭐ integrate

- **Catalog**: `src/components/` = 56 files / 55 modules, "290 exported types across 55 modules", claims **100 % shadcn/ui v4 coverage** — includes everything quoin's DSL lacks today: `card`, `dialog`, `alert_dialog`, `popover`, `combobox`, `command`, `menubar`, `context_menu`, `navigation_menu`, `sidebar`, `sheet`, `drawer`, `calendar`, `date_picker`, `carousel`, `chart`, `table`, `resizable`, `breadcrumb`, `pagination`, `avatar`, `hover_card`, `field`, `item`, `kbd`, `empty`, `spinner`, `toast`, …
- **API shape** (from `components/button.rs`): builder style — `Button::new(child).sm().lg().ghost().destructive().outline().secondary()`, icon variant, plus per-part components (`Card` + `CardHeader/Title/…`). This is *macro-emitter friendly*: arguments map 1:1 to builder calls, no `view!`-style prop splicing.
- **Theming**: `ShadcnTheme::light()/dark()` as `AlphaColor<Oklch>`; hover/active derived by lightness deltas; `ShadcnStyleExt` exposes ~40 token accessors. A quoin `ThemeToken` bridge is straightforward (quoin-ui already has `ThemeToken`/`TextColor`).
- **Styling**: builds on `floem-tailwind` (also yours) — typed `w_full()/gap_2()/text_sm()` equivalents of the shadcn class strings.
- **Testing**: uses floem's headless `floem-test` harness (128 + 46 doc tests, interaction-level: click dialog trigger → assert closed).
- **Compatibility**: depends on `floem` git main — the same pin style `quoin-floem` and `navi-demo-floem` already use, so no version skew. `unsafe` forbidden, clippy-pedantic clean, docs complete.

**Effort**: LOW–MEDIUM. Add `quoin-floem/shadcn` feature + `floem-shadcn` dep; write `render_floem/shadcn_*.rs` handlers per element (start with button/input/badge/checkbox/switch/separator/tabs/progress/slider/tooltip/card/dialog); extend `parity.rs` rows from `Native(generic)` to shadcn-backed `Native`. The builder API means no event-handler gymnastics beyond what `render_floem/handler.rs` already does.

### 4.2 Leptos — `harana-oss/leptos-shadcn-ui` ⭐ already integrated, expand

- 63 component crates under `packages/leptos/` (accordion … toast; also `error-boundary`, `form`, `native-select`, `textarea`, `sonner`, `tabs`, …), versioned `v0.9.1`, Leptos 0.9-CSR-compatible (works with quoin's wasm32 target).
- API: `view!`-prop style — `<Button variant=ButtonVariant::Outline size=ButtonSize::Sm on_click=Callback::new(…)>` — already what `render_leptos/button.rs` splices.
- Each component has `default.rs` / `new_york.rs` style variants and constant class strings (shadcn canon), plus responsive/touch variants.
- Repo also ships a Rust **shadcn CLI port** (`packages/shadcn`) and a registry (`packages/registry`) — useful later for "copy the component" workflows, not needed for quoin.

**Effort**: LOW. Quoin already enables 8 of 63. Phased enablement (see §6) + new emitter handlers for the canonical additions.

### 4.3 GPUI — `longbridge/gpui-kit` (`gpui-component`) ⭐ canonical anchor

- 75+ components, light/dark themes, production-proven (Longbridge Pro); already the reference backend in `quoin-ui-gpui` (0.8.1 pinned) and in navi's example-app.
- The gap analysis you filed in [gpui-kit discussion #2311](https://github.com/longbridge/gpui-kit/discussions/2311) is exactly the harmonization program the canonical contract needs: variant renames (`primary→default`, `danger→destructive`, `text→link`), canonical sub-components (`AccordionItem/Trigger/Content`), controlled/uncontrolled props (`on_change` + optional `value` signal), and missing components (`Card`, `Carousel`, `Combobox`, `Command`, `Empty`, `Menubar`, `NavigationMenu`, `ScrollArea`, `Separator`, `Sidebar`).
- Until upstream lands those, quoin's GPUI emitter can **shim** the canonical names onto existing widgets (e.g. `card` → styled `GroupBox`/div composition) — the same fallback pattern the Leptos emitter already uses.

**Effort**: MEDIUM–LONG (upstream-dependent for the missing components; shimmable today for variants/sub-component naming).

### 4.4 Dioxus — three sources, hybrid

| Library | Coverage | Notes |
|---|---|---|
| `MBeliou/shadcn-dioxus` (integrated) | 28 modules: alert, avatar, badge, button, card, checkbox, dialog, input, item, kbd, label, native_select, portal, progress, separator, sheet, skeleton, spinner, switch, textarea, toggle, … | Missing for quoin parity: data_table (emitted as HTML `<table>` today — fine), dropdown_menu (exists ✅), virtual_list (RecycleList ⚠) |
| [`yuxuetr/dioxus-ui`](https://github.com/yuxuetr/dioxus-ui) `dioxus-shadcn` | 83 components, one cargo feature per component, `dxui` CLI (init/add/diff), `dioxus-shadcn-primitives` unstyled layer, semver'd, doc-commented | Strongest breadth; alternative source for gaps (command palette, menubar, navigation-menu, pagination, breadcrumb, carousel, calendar, chart, sidebar …) |
| [`DioxusLabs/components`](https://github.com/DioxusLabs/components) | first-party **unstyled** accessible primitives (shadcn-style) | the Radix-equivalent foundation; ideal for behaviors where styled ports lag |

**Effort**: MEDIUM (mainly deciding the source-of-truth per missing component and keeping one emit path per element).

### 4.5 Xilem — nothing to leverage (yet)

Web/community research finds **no shadcn-class component library for Xilem**; the ecosystem conversation (2026 critical reviews) is still at the architecture stage, and linebender ships no styled component layer. quoin's Xilem emitter stays on the generic path, gated by `parity.rs` (`Partial`/`Unsupported` rows with honest reasons). Revisit when linebender's component story lands; the canonical contract costs nothing to keep Xilem-compatible (the contract is framework-neutral data).

### 4.6 Adjacent ecosystems (context, not targets)

Useful evidence that "shadcn canon as cross-framework contract" is the emerging pattern in Rust:

- **`FerrisMind/shadcn-rs`** — shadcn/Radix-inspired workspace for **iced + egui**, two API styles (props-first `*Props`, builder-first), shared `shadcn-common` theming on `twill`.
- **`iced-shadcn`** (lib.rs, Mar 2026) — "shared visual language and theme tokens that match the rest of the shadcn-rs ecosystem", explicitly under construction.
- **OUROBOROS-UI** (egui) — shadcn design language with token-governance tests.
- **Slint shadcn-style** port (reddit, 2026) and **shadcn for ratatui** — the design language is spreading beyond GUI toolkits.
- These confirm the *contract-first* approach: none of them can be a quoin backend today (quoin has no iced/egui adapter), but the variant/prop vocabulary they converge on should inform the canonical spec (§5).

---

## 5. The unified API: what "same interface, lowered per framework" means concretely

### 5.1 Canonical layer (new, in quoin)

A `quoin-macros-core` (or `quoin-shadcn-spec`) module defining, in code:

1. **Canonical element set** — extend `KNOWN_ELEMENTS` (~45 → ~60) with the shadcn canon that has ≥3-framework coverage: `card`(+`card_header/card_title/card_content/card_footer`), `dialog`(+trigger/content/title/description/close), `alert`, `alert_dialog`, `popover`, `accordion`(+item/trigger/content), `command`, `combobox`, `menubar`, `context_menu`, `navigation_menu`, `sheet`, `drawer`, `avatar`, `breadcrumb`, `pagination`, `calendar`, `date_picker`, `carousel`, `chart`, `sidebar`, `table` parts, `toast`/`sonner`, `kbd`, `avatar`, `empty`, `field`, `spinner`, `toggle`, `input_otp`, `input_group`, `collapsible`, `resizable`, `hover_card`, `aspect_ratio`, `native_select`, `textarea`.
2. **Canonical argument grammar** per element — the emitter contract, e.g.
   `button(variant: default|destructive|outline|secondary|ghost|link, size: sm|default|lg|icon, disabled, loading, on_click, class, tooltip, children)`.
   *Note*: today's DSL uses boolean args (`primary:`/`destructive:`/`ghost:` on button); the canonical form should be a single `variant:` arg with the booleans kept as deprecated sugar — one spelling, mappable everywhere.
3. **Canonical variant enums** as a const table: `ButtonVariant`, `BadgeVariant`, `AlertVariant`, `InputSize` … each row carrying the per-framework token (`leptos_shadcn_ui::ButtonVariant::Destructive`, `floem_shadcn::ButtonVariant::Destructive`, gpui-component's `ButtonVariant` post-#2311, `shadcn_dioxus::button::ButtonVariant`).
4. **Controlled/uncontrolled seam**: `value:` + `on_change:` args compile to controlled props where the backend supports them (Leptos/Dioxus/GPUI post-#2311), and to a fallback internal-state binding where not (Floem builder, Xilem generic).
5. **Parity enforcement**: every new element gets a `parity.rs` row; the existing parity-matrix test keeps Floem/Xilem claims honest, and the doc-checker keeps `docs/BACKEND-PARITY.md` in sync.

### 5.2 Per-framework lowering contract

| Canonical element | GPUI (gpui-component) | Floem (floem-shadcn) | Leptos (leptos-shadcn-ui) | Dioxus (shadcn-dioxus / dioxus-shadcn) | Xilem |
|---|---|---|---|---|---|
| `button(variant:, size:, disabled:, on_click:)` | `Button::new().variant/label` | `Button::new(child).ghost().sm()` | `<Button variant=… on_click=…>` | `Button { variant: …, on_click: … }` | generic |
| `input(value:, on_change:, placeholder:, disabled:)` | `Input`+`InputState` (already) | `Input` module | `<Input …>` | `Input { … }` | generic |
| `card` family | shim until #2311 lands (`Card`) | `Card`+`CardHeader/Title/Content/Footer/Action` ✅ | `leptos-shadcn-card` ✅ | `shadcn_dioxus::card` ✅ | ❌ |
| `dialog` family | declarative `Dialog` exists (#2311 notes) ✅ | `Dialog`+Trigger/Content/… ✅ | `leptos-shadcn-dialog` ✅ | `shadcn_dioxus::dialog` ✅ | ❌ |
| `accordion` | shim / upstream ask | ✅ full sub-parts | ✅ | dioxus-shadcn ✅ | ❌ |
| `command`, `combobox`, `menubar`, `sidebar`, `chart`, `calendar` | #2311 asks | ✅ all | ✅ (most) | dioxus-shadcn ✅ (verify each) | ❌ |
| `data_table`, `virtual_list`, `clipboard_button`, `styled_text`, `rich_text` | native gpui-component ✅ | floem-shadcn `table` ✅; list virtualization via floem | `leptos-shadcn-table` ✅ + web APIs ✅ | HTML `<table>` + web APIs ✅; `dioxus-primitives::RecycleList` ⚠ | partial |

This is deliberately the same table shape as the existing `docs/leptos-dioxus-emission-parity.md`, generalized to five backends and machine-checked where possible.

### 5.3 What stays framework-idiomatic (important non-goal)

The unified API covers *structure, variants, state seams and events*. Hosts still write framework-native code around it (`rsx!`, `view!`, GPUI entities, xilem views), and quoin's `rsx!`/`gpui-rsx-classes` escape hatch remains for GPUI-only subtrees. "Same API" = same `quoin_render!` source compiles everywhere with native visual quality — not a lowest-common-denominator widget set.

---

## 6. Gap analysis & effort estimate

| Work item | Backend(s) | Size | Notes |
|---|---|---|---|
| Canonical spec module + extend `KNOWN_ELEMENTS` | all | M (1 wk) | pure macro-core work; no framework deps; parity rows + docs |
| `variant:` arg + boolean-sugar deprecation | all | S | emitter `common.rs` helper |
| `floem-shadcn` integration: dep, theme bridge, 10–15 elements | floem | M (1–2 wk) | biggest win; builder API is emitter-friendly; extend parity to `Native` |
| Expand `leptos-shadcn` features 8 → ~25 + new handlers | leptos | S–M (3–5 d) | cargo features per crate already exist upstream |
| Dioxus gap fill (dialog/accordion/command/… source decision) | dioxus | M (3–5 d) | hybrid MBeliou + dioxus-shadcn |
| GPUI shims (card/dialog composition) + #2311 PRs | gpui | M–L | upstream collaboration; shims unblock parity meanwhile |
| Xilem | xilem | — | keep generic; document in parity table |

Total for the *quoin-side* work: **~2–4 focused weeks**. The upstream items (gpui-kit #2311, dioxus primitives) proceed independently and are not on the critical path.

---

## 7. Risks and mitigations

1. **API churn in the backing libraries** (all are 0.x, floem/xilem track git mains). *Mitigation*: pin revs (as `quoin-leptos` already does) behind the feature flags; the flags make swaps one-line; the parity test catches drift.
2. **Two Dioxus sources diverge** (MBeliou vs yuxuetr). *Mitigation*: choose per-element source-of-truth in the canonical table; wrap behind quoin's emitter so user code never imports the backing crate directly.
3. **Tailwind class strings are meaningless on GPU backends.** *Mitigation*: the canonical contract carries *semantic* args (`variant:`, `size:`), not classes; `class:` stays an escape hatch that Floem/GPUI interpret via `floem-tailwind` / `gpui-rsx-classes` respectively — both of which already exist in this stack.
4. **Theming mismatch** (OKLCH in floem-shadcn vs CSS tokens on web vs `ActiveTheme` in gpui-component). *Mitigation*: extend `quoin_ui::ThemeToken` to be the single token source; per-adapter token→native-theme bridges (a `ShadcnTheme::from_tokens` for Floem exists conceptually in `theme.rs`).
5. **Feature-flag explosion**. *Mitigation*: follow the established pattern — one `<fw>-shadcn` facade flag implying the adapter + macro features (`floem-shadcn = ["floem", "quoin-floem?/shadcn", "quoin-macros?/floem-shadcn"]`), exactly like `leptos-shadcn`/`dioxus-shadcn` today.
6. **Scope creep toward 60 elements at once.** *Mitigation*: phase by coverage class — Tier 1 (form + surface: button/input/card/badge/checkbox/switch/separator/tabs/tooltip/dialog) → Tier 2 (navigation: accordion/menubar/dropdown/breadcrumb/pagination) → Tier 3 (data: table/chart/calendar/combobox/command).

---

## 8. Recommended roadmap

- **Phase 0 (spec) — ✅ LANDED** (see §10): `emit::shadcn_spec` module (Tier-1 table + canonical
  `variant:`/`size:` grammar with boolean-sugar fallback), `KNOWN_ELEMENTS` +14 surface elements,
  parity-matrix carve-out for the shadcn layer, gpui/leptos/dioxus/floem emitters wired.
- **Phase 1 (fast wins) — ✅ LANDED (Floem + Leptos; Dioxus partial)** (see §10):
  `floem-shadcn` vendored backend + 13 Tier-1 lowerings, Leptos 8 → 28 component features
  (wasm32 green), Dioxus canonical variant classes. Tier-2 elements (dialog/accordion/…)
  and Dioxus source-of-truth decisions remain open as planned.
- **Phase 2 (gpui convergence) — open**: implement the shimmable parts of #2311 in quoin's
  gpui emitter; contribute the upstream PRs (canonical sub-components, controlled props,
  missing components). Landed groundwork: gpui emitter now maps `destructive` → `danger`
  (0.8.1 naming) and imports `ButtonVariants`/`Sizable` into generated render bodies.
- **Phase 3 (frontier)** — Tier-2/3 elements across backends; revisit Xilem when linebender has a component story; optionally evaluate `shadcn-rs` (iced/egui) if quoin ever grows those adapters.

---

## 9. Sources

Code studied (cloned this session):

- quoin @ `cb622a3` — `render_ast.rs`, `emit/parity.rs`, `emit/render_*/`, `quoin-ui/src/node.rs`, `quoin-ui-gpui/src/node_renderer.rs`, `quoin-leptos/Cargo.toml`, `quoin-dioxus/Cargo.toml`, `docs/leptos-dioxus-emission-parity.md`
- `elcoosp/floem-shadcn` — README, `COMPONENTS.md`, `src/components/button.rs`, `src/theme.rs`, `Cargo.toml`
- `harana-oss/leptos-shadcn-ui` — README, `packages/leptos/*` (63 crates), `packages/shadcn` (CLI port), button `default.rs`
- `MBeliou/shadcn-dioxus` — `packages/ui/src` (28 modules)

Web (accessed 2026-11-05):

- gpui-kit discussion #2311, "Harmonizing `gpui-component` API with the ShadCN canonical spec (to support `quoin` and cross-framework UI)" — https://github.com/longbridge/gpui-kit/discussions/2311
- GPUI Kit site & repo — https://gpui-kit.com · https://github.com/longbridge/gpui-kit
- `dioxus-shadcn` (yuxuetr) README — https://github.com/yuxuetr/dioxus-ui (docs.rs: `dioxus-shadcn-primitives`)
- shadcn-dioxus — https://shadcn-dioxus.com · DioxusLabs first-party primitives — https://github.com/DioxusLabs/components · catalog — https://dioxuslabs.com/components
- `FerrisMind/shadcn-rs` (iced/egui) — https://github.com/FerrisMind/shadcn-rs
- `iced-shadcn` — https://lib.rs/crates/iced-shadcn
- Xilem status — https://xilem.dev · https://github.com/linebender/xilem/blob/main/xilem/ARCHITECTURE.md · 2026 critical review (reddit r/rust)
- shadcn ecosystem spread (Slint, ratatui, egui OUROBOROS-UI threads, r/rust 2026)

---

## 10. Phase 0/1 landed inventory & verification matrix (2026-11-05)

### 10.1 What landed

**Phase 0 — the canonical contract (checked-in Rust, not just docs):**

| Artifact | Where | Notes |
|---|---|---|
| `emit::shadcn_spec` module | `quoin-macros-core/src/emit/shadcn_spec.rs` | `CANONICAL_TIER1` table (14 elements × per-framework backing map), canonical variant/size constants, `canonical_choice` reader ("string arg wins, sugar breaks ties", unknown values = precise `compile_error!`), 13 unit tests |
| `KNOWN_ELEMENTS` +14 | `render_ast.rs`, `render_ast_diag.rs` | `card`, `card_header`, `card_title`, `card_description`, `card_content`, `card_footer`, `card_action`, `alert`, `alert_title`, `alert_description`, `spinner`, `kbd`, `avatar`, `avatar_fallback` — both parser tables kept in lockstep |
| Canonical `variant:`/`size:` grammar | gpui/leptos/dioxus/floem button+badge+alert emitters | `button(variant: "destructive", size: "lg")` compiles on 4 backends from one source; boolean sugar (`primary:`/`destructive:`/`ghost:`/…) still accepted |
| Parity-matrix carve-out | `parity_matrix.rs` cfg | The matrix documents the **generic** contract; the shadcn layer steps aside with an honest comment (its own coverage is the quoin-floem conformance suite) |
| trybuild snapshot refresh | `render_unknown_element_fail.stderr` | Machine-verified match via the dioxus trybuild suite (all green) |

**Phase 1 — the fast wins:**

| Artifact | Where | Notes |
|---|---|---|
| `floem-shadcn` vendored | `third_party/floem-shadcn` | Follows the `gpui-stub`/`muda-stub` pattern. **Fixes an upstream blocker**: `Cargo.toml` declared `floem-tailwind = { path = "https://github.com/elcoosp/floem-tailwind" }` — a URL is not a valid path dep; vendored copy uses `git =`. Upstream one-line fix recommended. |
| Floem adapter feature | `quoin-floem/Cargo.toml` `shadcn` | `dep:floem-shadcn` + dev-dep `quoin-macros/floem-shadcn` so conformance tests compile *and run* against the shadcn lowerings |
| Floem Tier-1 emitters | `render_floem/{button,input,widgets}.rs` | button (variant/size), input (controlled `.value()`/`.on_update()`), badge, checkbox/switch (same guarded two-way bridge, real shadcn chrome), separator, slider, progress, skeleton, alert, card + 6 card parts, spinner, kbd, avatar — every one cfg-switched with the generic fallback |
| Feature chain | `quoin`/`quoin-macros`/`quoin-macros-core` Cargo.tomls | `floem-shadcn = ["floem", "quoin-floem?/shadcn", "quoin-macros?/floem-shadcn"]` — the same one-flag pattern as `leptos-shadcn`/`dioxus-shadcn` |
| Leptos broadening | `quoin-leptos/Cargo.toml` | 8 → 28 component features (adds card, alert, alert-dialog, accordion, dialog, popover, collapsible, separator, checkbox, switch, radio-group, slider, progress, skeleton, toggle, spinner, kbd, avatar, label, textarea) |
| Dioxus canonical variants | `render_dioxus/button.rs` | canonical `variant:` → shadcn Tailwind class strings (class-based path) |
| gpui groundwork | `emit/gpui.rs`, `render_gpui/button.rs` | generated render bodies import `ButtonVariants`/`Sizable` (variant args previously could not resolve — latent bug), `destructive` maps to 0.8.1's `danger` |
| gpui expansion regression test | `quoin-macros-tests/tests/button_variant_expansion_gpui.rs` | all 6 canonical variants + 3 sizes + sugar — typechecks against real gpui-component 0.8.1 in the normal test target (no trybuild scratch build needed) |
| Parser legalisation | `KNOWN_ARGS` +`variant`,`description`,`fallback`,`small`,`xsmall` | `small:`/`xsmall:` were documented by the gpui emitter but rejected by the parser — now actually usable |

### 10.2 Verification matrix (all commands green this session)

| Gate | Command | Result |
|---|---|---|
| Contract unit tests | `cargo nextest run -p quoin-macros-core --features floem,xilem` | 13/13 ✅ |
| Generic parity matrix | `cargo nextest run -p quoin-macros-core --features floem,xilem` | 39/39 ✅ (parity_matrix 5/5) |
| shadcn-combo emitters | `cargo nextest run -p quoin-macros-core --features floem,floem-shadcn,xilem` | 34/34 ✅ |
| **Floem shadcn conformance** | `cargo nextest run -p quoin-floem --features shadcn` | **47/47 ✅ — real expansions against vendored floem-shadcn** |
| Dioxus shadcn | `cargo nextest run -p quoin-dioxus --features shadcn` | 32/32 ✅ |
| Xilem | `cargo nextest run -p quoin-xilem` | 39/39 ✅ |
| Core crates | `cargo nextest run -p quoin-core -p quoin-style -p quoin-ui` | 121/121 ✅ |
| Testkit/devtools/conformance/macros-tests | `cargo nextest run -p quoin-macros-tests -p quoin-testkit -p quoin-devtools -p quoin-conformance` | 56/56 ✅ |
| trybuild (dioxus suite incl. refreshed snapshot) | `cargo test -p quoin-macros-tests --features dioxus` | ✅ all fixtures |
| Facade flag | `cargo check -p quoin --features floem,floem-shadcn` | ✅ |
| GPUI adapter | `cargo check -p quoin-gpui` | ✅ |
| Canonical variant expansion (gpui-component 0.8.1) | `cargo check -p quoin-macros-tests --features gpui --test button_variant_expansion_gpui` | ✅ |
| Leptos 28-feature shadcn | `cargo check -p quoin-leptos --features shadcn --target wasm32-unknown-unknown` | ✅ |
| Standalone expand/grsx suites | `cargo test` in `quoin-expand-test`, `quoin-grsx-compiletest` | ✅ (compile-time assertions) |
| navi against new quoin | `git reset --hard origin/main` (36 commits) + `cargo check -p example-app` + `cargo nextest run -p navi-core -p navi-router -p navi-router-core` | ✅ 155/155 |
| navi example-app live | headless Xvfb + lavapipe run | ✅ renders full nav tree + devtools, "Route tree validation complete", stable |

**Known environment limits (documented honestly):** the gpui trybuild suite needs a second
full gpui scratch workspace (~2.7 GB) which does not fit this 10 GB sandbox alongside the
main target; equivalent coverage was established via the dioxus trybuild suite (shared
fail-fixtures + snapshot), the gpui expansion integration test, and the
gpui-component API probe. Upstream CI runs the gpui suite unchanged.

### 10.3 Follow-ups (Phase 2/3, unchanged priorities)

1. Push the `floem-tailwind` path-dep fix upstream so `floem-shadcn` can be consumed as a git dep and the vendored copy retired.
2. Tier-2 elements (dialog/accordion/popover/command) once overlay seams exist on GPU backends; each gets parity rows then.
3. Dioxus source-of-truth decisions (MBeliou vs yuxuetr `dioxus-shadcn`) per missing component.
4. gpui-kit #2311: canonical renames (`danger`→`destructive`, sizes), sub-component splits, controlled props.
5. Xilem stays generic until linebender ships a component story; the contract costs nothing there.
